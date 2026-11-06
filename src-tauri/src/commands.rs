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
}

#[tauri::command]
pub async fn search_files(
    query: SearchQuery,
    state: State<'_, AppState>,
) -> Result<SearchResult, String> {
    let start_time = std::time::Instant::now();

    // If folder scope is locked, prioritize SQLite direct search or filesystem reading for 100% accuracy
    if let Some(folder_path) = &query.folder {
        let mut db_items = state.db.search(&query).unwrap_or_default();

        // If database search has no items, read directly from disk folder
        if db_items.is_empty() {
            let path_obj = std::path::Path::new(folder_path);
            if path_obj.is_dir() {
                if let Ok(entries) = std::fs::read_dir(path_obj) {
                    let kw = query.query.trim().to_lowercase();
                    for entry in entries.flatten() {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        if !kw.is_empty() && !fname.to_lowercase().contains(&kw) {
                            continue;
                        }
                        let p = entry.path().to_string_lossy().to_string();
                        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                        let ext = if is_dir {
                            "".to_string()
                        } else {
                            entry.path().extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
                        };
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
    let scale = window.scale_factor().unwrap_or(1.0);
    let target_h = if compact { 58.0 } else { 580.0 };
    let target_w = 800.0;

    let (target_x, target_y) = if let Ok(Some(monitor)) = window.current_monitor() {
        let mon_pos = monitor.position();
        let mon_size = monitor.size();
        let mon_w = mon_size.width as f64 / scale;
        let mon_h = mon_size.height as f64 / scale;
        let x = (mon_pos.x as f64 / scale) + (mon_w - target_w) / 2.0;
        let y = (mon_pos.y as f64 / scale) + (mon_h - target_h) / 2.0;
        (x, y)
    } else {
        (0.0, 0.0)
    };

    let cur_outer = window.outer_position().unwrap_or_default();
    let cur_inner = window.inner_size().unwrap_or_default();

    let start_x = cur_outer.x as f64;
    let start_y = cur_outer.y as f64;
    let start_w = cur_inner.width as f64;
    let start_h = cur_inner.height as f64;

    let end_x = target_x * scale;
    let end_y = target_y * scale;
    let end_w = target_w * scale;
    let end_h = target_h * scale;

    if (start_h - end_h).abs() > 3.0 || (start_y - end_y).abs() > 3.0 {
        let hwnd_opt: Option<isize> = {
            #[cfg(target_os = "windows")]
            {
                window.hwnd().ok().map(|h| h.0 as isize)
            }
            #[cfg(not(target_os = "windows"))]
            {
                None
            }
        };

        if let Some(hwnd_val) = hwnd_opt {
            let steps = 8;
            for i in 1..=steps {
                let t = i as f64 / steps as f64;
                // Cubic ease-out
                let ease = 1.0 - (1.0 - t).powi(3);
                let cur_x_val = (start_x + (end_x - start_x) * ease).round() as i32;
                let cur_y_val = (start_y + (end_y - start_y) * ease).round() as i32;
                let cur_w_val = (start_w + (end_w - start_w) * ease).round() as i32;
                let cur_h_val = (start_h + (end_h - start_h) * ease).round() as i32;

                #[cfg(target_os = "windows")]
                {
                    use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};
                    use windows::Win32::Foundation::HWND;
                    unsafe {
                        let _ = SetWindowPos(
                            HWND(hwnd_val as _),
                            None,
                            cur_x_val,
                            cur_y_val,
                            cur_w_val,
                            cur_h_val,
                            SWP_NOACTIVATE | SWP_NOZORDER,
                        );
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(12)).await;
            }
        }
    }

    let _ = window.set_size(tauri::LogicalSize::new(target_w, target_h));
    let _ = window.set_position(tauri::LogicalPosition::new(target_x, target_y));

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
