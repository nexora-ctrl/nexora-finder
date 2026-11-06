use std::time::Duration;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use crate::db::Database;
use crate::search::SearchEngine;
use crate::types::FileItem;

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
}

impl FileWatcher {
    pub fn start(db: Database, search: SearchEngine) -> anyhow::Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        // Collect watch directories: User Desktop, Documents, Downloads, etc.
        let mut watch_paths = Vec::new();
        if let Some(user_dirs) = directories::UserDirs::new() {
            watch_paths.push(user_dirs.home_dir().to_path_buf());
            if let Some(desktop) = user_dirs.desktop_dir() {
                watch_paths.push(desktop.to_path_buf());
            }
            if let Some(docs) = user_dirs.document_dir() {
                watch_paths.push(docs.to_path_buf());
            }
            if let Some(downloads) = user_dirs.download_dir() {
                watch_paths.push(downloads.to_path_buf());
            }
        }

        for path in &watch_paths {
            if path.exists() {
                let _ = watcher.watch(path, RecursiveMode::Recursive);
            }
        }

        // Spawn background handler
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                handle_fs_event(&db, &search, event);
            }
        });

        Ok(Self {
            _watcher: watcher,
        })
    }
}

fn handle_fs_event(db: &Database, search: &SearchEngine, event: Event) {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {
            for path in event.paths {
                if let Ok(meta) = path.metadata() {
                    let path_str = path.to_string_lossy().to_string();
                    let is_dir = meta.is_dir();
                    let filename = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path_str.clone());

                    let ext = path.extension()
                        .map(|e| e.to_string_lossy().to_lowercase())
                        .unwrap_or_default();

                    let drive = path_str.chars().take(2).collect::<String>();
                    let size_bytes = if is_dir { 0 } else { meta.len() };
                    let modified_at = meta.modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);

                    let item = FileItem {
                        id: None,
                        path: path_str,
                        filename,
                        extension: ext,
                        size_bytes,
                        modified_at,
                        is_directory: is_dir,
                        drive,
                    };

                    let _ = db.upsert_batch(&[item.clone()]);
                    let _ = search.add_batch(&[item]);
                    let _ = search.commit();
                }
            }
        }
        EventKind::Remove(_) => {
            for path in event.paths {
                let path_str = path.to_string_lossy().to_string();
                let _ = db.delete_by_path(&path_str);
                let _ = search.delete_by_path(&path_str);
                let _ = search.commit();
            }
        }
        _ => {}
    }
}
