use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;
use tauri::{AppHandle, Emitter};
use crate::db::Database;
use crate::drive::get_available_drives;
use crate::search::SearchEngine;
use crate::types::{FileItem, IndexingStatus};

pub struct Indexer {
    pub db: Database,
    pub search: SearchEngine,
    pub is_indexing: Arc<AtomicBool>,
    pub total_indexed: Arc<AtomicU64>,
    pub current_drive: Arc<RwLock<String>>,
    pub current_directory: Arc<RwLock<String>>,
    pub last_completed: Arc<RwLock<Option<String>>>,
}

impl Indexer {
    pub fn new(db: Database, search: SearchEngine) -> Self {
        let initial_count = db.get_total_count().unwrap_or(0);
        let last_time = db.get_metadata("last_indexed_at").unwrap_or(None);

        Self {
            db,
            search,
            is_indexing: Arc::new(AtomicBool::new(false)),
            total_indexed: Arc::new(AtomicU64::new(initial_count)),
            current_drive: Arc::new(RwLock::new(String::new())),
            current_directory: Arc::new(RwLock::new(String::new())),
            last_completed: Arc::new(RwLock::new(last_time)),
        }
    }

    pub fn get_status(&self) -> IndexingStatus {
        IndexingStatus {
            is_indexing: self.is_indexing.load(Ordering::Relaxed),
            total_indexed: self.total_indexed.load(Ordering::Relaxed),
            current_drive: self.current_drive.read().clone(),
            current_directory: self.current_directory.read().clone(),
            progress_percent: 0.0,
            last_completed: self.last_completed.read().clone(),
        }
    }

    pub fn start_indexing(self: Arc<Self>, app_handle: AppHandle) {
        if self.is_indexing.swap(true, Ordering::SeqCst) {
            // Already indexing
            return;
        }

        let indexer = self.clone();
        let _ = std::thread::Builder::new()
            .name("nexora-indexer".into())
            .spawn(move || {
            let start_time = Instant::now();
            let drives = get_available_drives();

            let mut batch = Vec::with_capacity(2500);
            let mut count_since_commit = 0;

            for drive in drives {
                if !indexer.is_indexing.load(Ordering::Relaxed) {
                    break;
                }
                *indexer.current_drive.write() = drive.letter.clone();
                let root_path = format!("{}\\", drive.letter);

                let walker = jwalk::WalkDirGeneric::<((), ())>::new(&root_path)
                    .skip_hidden(false)
                    .follow_links(false)
                    .process_read_dir(|_depth, path, _state, children| {
                        // Filter out forbidden or system directories
                        let path_str = path.to_string_lossy();
                        if is_ignored_path(&path_str) {
                            children.clear();
                            return;
                        }

                        children.retain(|child_res| {
                            if let Ok(entry) = child_res {
                                let name = entry.file_name.to_string_lossy();
                                !is_ignored_name(&name)
                            } else {
                                false
                            }
                        });
                    });

                for entry_result in walker {
                    if !indexer.is_indexing.load(Ordering::Relaxed) {
                        break;
                    }

                    if let Ok(entry) = entry_result {
                        let path = entry.path();
                        let path_str = path.to_string_lossy().to_string();

                        if let Ok(meta) = entry.metadata() {
                            let is_dir = meta.is_dir();
                            let filename = path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| path_str.clone());

                            let ext = path.extension()
                                .map(|e| e.to_string_lossy().to_lowercase())
                                .unwrap_or_default();

                            let size_bytes = if is_dir { 0 } else { meta.len() };
                            let modified_at = meta.modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs() as i64)
                                .unwrap_or(0);

                            batch.push(FileItem {
                                id: None,
                                path: path_str,
                                filename,
                                extension: ext,
                                size_bytes,
                                modified_at,
                                is_directory: is_dir,
                                drive: drive.letter.clone(),
                            });

                            if batch.len() >= 2500 {
                                let len = batch.len() as u64;
                                let _ = indexer.db.upsert_batch(&batch);
                                let _ = indexer.search.add_batch(&batch);
                                batch.clear();

                                indexer.total_indexed.fetch_add(len, Ordering::Relaxed);
                                count_since_commit += len;

                                if count_since_commit >= 10000 {
                                    let _ = indexer.search.commit();
                                    count_since_commit = 0;
                                }

                                if let Some(parent) = path.parent() {
                                    *indexer.current_directory.write() = parent.to_string_lossy().to_string();
                                }

                                // Emit progress update to UI
                                let _ = app_handle.emit("indexing-status", indexer.get_status());
                            }
                        }
                    }
                }
            }

            // Flush remaining
            if !batch.is_empty() {
                let len = batch.len() as u64;
                let _ = indexer.db.upsert_batch(&batch);
                let _ = indexer.search.add_batch(&batch);
                indexer.total_indexed.fetch_add(len, Ordering::Relaxed);
                batch.clear();
            }

            let _ = indexer.search.commit();

            let elapsed_secs = start_time.elapsed().as_secs();
            let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let _ = indexer.db.set_metadata("last_indexed_at", &now_str);
            let _ = indexer.db.set_metadata("indexing_completed", "true");
            *indexer.last_completed.write() = Some(now_str);
            indexer.is_indexing.store(false, Ordering::SeqCst);

            *indexer.current_drive.write() = String::new();
            *indexer.current_directory.write() = format!("Selesai dalam {}s", elapsed_secs);

            let _ = app_handle.emit("indexing-status", indexer.get_status());
        });
    }
}

fn is_ignored_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("$recycle.bin")
        || lower.contains("system volume information")
        || lower.contains("\\windows\\winsxs")
        || lower.contains("\\windows\\servicing")
        || lower.contains("\\node_modules")
        || lower.contains("\\.git")
        || lower.contains("\\.nexora-target")
        || lower.contains("\\target\\")
        || lower.contains("\\appdata\\local\\temp")
}

fn is_ignored_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower == "pagefile.sys"
        || lower == "hiberfil.sys"
        || lower == "swapfile.sys"
        || lower == "dumpstack.log.tmp"
        || lower == "node_modules"
        || lower == ".git"
        || lower == "target"
        || lower == ".nexora-target"
        || lower.starts_with("$")
}
