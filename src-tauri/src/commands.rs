use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use crate::actions::{open_file, open_file_with, reveal_in_explorer};
use crate::db::Database;
use crate::drive::get_available_drives;
use crate::indexer::Indexer;
use crate::search::SearchEngine;
use crate::thumbnail::{ThumbnailCache, get_file_thumbnail};
use crate::types::{DriveInfo, FileItem, IndexStats, IndexingStatus, SearchQuery, SearchResult};

pub struct AppState {
    pub db: Database,
    pub search: SearchEngine,
    pub indexer: Arc<Indexer>,
    pub thumbnails: ThumbnailCache,
    pub initial_folder: Arc<parking_lot::Mutex<Option<String>>>,
}

fn is_category_match(category: &str, ext: &str) -> bool {
    let ext_lower = ext.to_lowercase();
    match category.to_lowercase().as_str() {
        "document" => matches!(ext_lower.as_str(), "pdf" | "doc" | "docx" | "txt" | "xlsx" | "xls" | "pptx" | "ppt" | "csv" | "md"),
        "image" => matches!(ext_lower.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg" | "ico"),
        "video" => matches!(ext_lower.as_str(), "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "flv"),
        "audio" => matches!(ext_lower.as_str(), "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a"),
        "app" => matches!(ext_lower.as_str(), "exe" | "msi" | "bat" | "cmd" | "ps1" | "lnk"),
        "archive" => matches!(ext_lower.as_str(), "zip" | "rar" | "7z" | "tar" | "gz" | "iso"),
        "code" => matches!(ext_lower.as_str(), "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "html" | "css" | "json" | "c" | "cpp"),
        _ => true,
    }
}

#[tauri::command]
pub async fn search_files(
    query: SearchQuery,
    state: State<'_, AppState>,
) -> Result<SearchResult, String> {
    let start_time = std::time::Instant::now();

    // If folder scope is locked, prioritize direct folder contents or folder-scoped search
    if let Some(folder_path) = &query.folder {
        let mut db_items = Vec::new();
        let path_obj = std::path::Path::new(folder_path);

        // When query is empty, show the direct contents of the chosen folder (directories first, then files)
        if query.query.trim().is_empty() {
            if path_obj.is_dir() {
                if let Ok(entries) = std::fs::read_dir(path_obj) {
                    for entry in entries.flatten() {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        let p = entry.path().to_string_lossy().to_string();
                        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                        let ext = if is_dir {
                            "".to_string()
                        } else {
                            entry.path().extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
                        };

                        if let Some(cat) = &query.category {
                            if cat == "folder" && !is_dir {
                                continue;
                            }
                            if cat != "all" && cat != "folder" && !is_category_match(cat, &ext) {
                                continue;
                            }
                        }

                        let meta = entry.metadata().ok();
                        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                        let mod_time = meta
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        let drive = folder_path.chars().next().unwrap_or('C').to_string();

                        db_items.push(FileItem {
                            id: None,
                            path: p,
                            filename: fname,
                            extension: ext,
                            size_bytes: size,
                            modified_at: mod_time,
                            is_directory: is_dir,
                            drive,
                        });
                    }
                }
            }

            // Fallback to database if read_dir returned empty
            if db_items.is_empty() {
                db_items = state.db.search(&query).unwrap_or_default();
            } else {
                // Sort direct folder items: directories first, then alphabetical by name
                db_items.sort_by(|a, b| {
                    b.is_directory.cmp(&a.is_directory).then_with(|| a.filename.to_lowercase().cmp(&b.filename.to_lowercase()))
                });
            }
        } else {
            // When query is not empty, search inside this folder using SQLite
            db_items = state.db.search(&query).unwrap_or_default();

            // If database search has no items, walk the directory directly
            if db_items.is_empty() && path_obj.is_dir() {
                let kw = query.query.trim().to_lowercase();
                let limit = query.limit.unwrap_or(150);

                let walker = jwalk::WalkDirGeneric::<((), ())>::new(path_obj)
                    .skip_hidden(false)
                    .max_depth(5);

                for entry_res in walker {
                    if let Ok(entry) = entry_res {
                        if entry.depth == 0 {
                            continue;
                        }
                        let fname = entry.file_name.to_string_lossy().to_string();
                        if !fname.to_lowercase().contains(&kw) {
                            continue;
                        }

                        let p = entry.path().to_string_lossy().to_string();
                        let is_dir = entry.file_type.is_dir();
                        let ext = if is_dir {
                            "".to_string()
                        } else {
                            entry.path().extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
                        };

                        if let Some(cat) = &query.category {
                            if cat == "folder" && !is_dir {
                                continue;
                            }
                            if cat != "all" && cat != "folder" && !is_category_match(cat, &ext) {
                                continue;
                            }
                        }

                        let meta = entry.metadata().ok();
                        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                        let mod_time = meta
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        let drive = folder_path.chars().next().unwrap_or('C').to_string();

                        db_items.push(FileItem {
                            id: None,
                            path: p,
                            filename: fname,
                            extension: ext,
                            size_bytes: size,
                            modified_at: mod_time,
                            is_directory: is_dir,
                            drive,
                        });

                        if db_items.len() >= limit {
                            break;
                        }
                    }
                }
            }
        }

        let total = db_items.len();
        return Ok(SearchResult {
            items: db_items,
            total_matched: total,
            elapsed_ms: start_time.elapsed().as_secs_f64() * 1000.0,
        });
    }

    let mut tantivy_res = state.search.search(&query).unwrap_or_else(|_| SearchResult {
        items: Vec::new(),
        total_matched: 0,
        elapsed_ms: 0.0,
    });

    // Filter by enabled drives if user configured specific drive scope
    if let Ok(Some(s)) = state.db.get_metadata("enabled_drives") {
        let enabled_set: std::collections::HashSet<String> = s
            .split(',')
            .map(|x| x.trim().to_uppercase())
            .filter(|x| !x.is_empty())
            .collect();
        if !enabled_set.is_empty() && !enabled_set.contains("ALL") {
            tantivy_res.items.retain(|item| enabled_set.contains(&item.drive.to_uppercase()));
            tantivy_res.total_matched = tantivy_res.items.len();
        }
    }

    tantivy_res.elapsed_ms = start_time.elapsed().as_secs_f64() * 1000.0;
    Ok(tantivy_res)
}

#[tauri::command]
pub async fn get_indexing_status(state: State<'_, AppState>) -> Result<IndexingStatus, String> {
    Ok(state.indexer.get_status())
}

#[tauri::command]
pub async fn start_indexing(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.indexer.clone().start_indexing(app);
    Ok(())
}

#[tauri::command]
pub async fn open_item(path: String) -> Result<(), String> {
    open_file(&path)
}

#[tauri::command]
pub async fn open_item_with(path: String) -> Result<(), String> {
    open_file_with(&path)
}

#[tauri::command]
pub async fn reveal_item(path: String) -> Result<(), String> {
    reveal_in_explorer(&path)
}

#[tauri::command]
pub async fn get_thumbnail(
    path: String,
    size: Option<u32>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if let Some(cached) = state.thumbnails.get_cached(&path) {
        return Ok(cached);
    }

    let target_sz = size.unwrap_or(128).clamp(32, 512);
    let path_clone = path.clone();
    let res = tokio::task::spawn_blocking(move || {
        get_file_thumbnail(&path_clone, target_sz)
    }).await.map_err(|e| e.to_string())??;

    state.thumbnails.insert(path, res.clone());
    Ok(res)
}

#[tauri::command]
pub async fn get_recent_files(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<FileItem>, String> {
    let lim = limit.unwrap_or(20);
    let mut files = state.db.query_recent(lim).map_err(|e| e.to_string())?;
    if let Ok(Some(s)) = state.db.get_metadata("enabled_drives") {
        let enabled_set: std::collections::HashSet<String> = s
            .split(',')
            .map(|x| x.trim().to_uppercase())
            .filter(|x| !x.is_empty())
            .collect();
        if !enabled_set.is_empty() && !enabled_set.contains("ALL") {
            files.retain(|item| enabled_set.contains(&item.drive.to_uppercase()));
        }
    }
    Ok(files)
}

#[tauri::command]
pub async fn get_drives() -> Result<Vec<DriveInfo>, String> {
    Ok(get_available_drives())
}

#[tauri::command]
pub async fn get_stats(state: State<'_, AppState>) -> Result<IndexStats, String> {
    let drives = get_available_drives();
    state.db.get_stats(drives).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn check_usn_status(drive: String) -> Result<bool, String> {
    Ok(crate::usn::UsnJournal::open(&drive).is_ok())
}

#[tauri::command]
pub async fn set_window_compact(compact: bool, window: tauri::WebviewWindow) -> Result<(), String> {
    let target_height = if compact { 58.0 } else { 580.0 };
    let target_width = 800.0;
    let scale = window.scale_factor().unwrap_or(1.0);

    // Resize via Tauri API so WebView2 synchronizes its controller viewport bounds
    let _ = window.set_size(tauri::LogicalSize::new(target_width, target_height));

    // Center window on current monitor
    if let Ok(Some(monitor)) = window.current_monitor() {
        let mon_pos = monitor.position();
        let mon_size = monitor.size();
        let mon_w = mon_size.width as f64 / scale;
        let mon_h = mon_size.height as f64 / scale;
        let x = (mon_pos.x as f64 / scale) + (mon_w - target_width) / 2.0;
        let y = (mon_pos.y as f64 / scale) + (mon_h - target_height) / 2.0;
        let _ = window.set_position(tauri::LogicalPosition::new(x, y));
    }

    Ok(())
}

#[tauri::command]
pub async fn get_indexed_drives(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    if let Ok(Some(s)) = state.db.get_metadata("enabled_drives") {
        let drives: Vec<String> = s
            .split(',')
            .map(|x| x.trim().to_uppercase())
            .filter(|x| !x.is_empty())
            .collect();
        if drives.is_empty() {
            Ok(vec!["ALL".to_string()])
        } else {
            Ok(drives)
        }
    } else {
        Ok(vec!["ALL".to_string()])
    }
}

#[tauri::command]
pub async fn set_indexed_drives(drives: Vec<String>, state: State<'_, AppState>) -> Result<(), String> {
    let s = drives.join(",");
    state.db.set_metadata("enabled_drives", &s).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_initial_folder(state: State<'_, AppState>) -> Option<String> {
    state.initial_folder.lock().take()
}
