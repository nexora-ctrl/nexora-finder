use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use crate::actions::{open_file, reveal_in_explorer};
use crate::db::Database;
use crate::drive::get_available_drives;
use crate::indexer::Indexer;
use crate::search::SearchEngine;
use crate::types::{DriveInfo, FileItem, IndexStats, IndexingStatus, SearchQuery, SearchResult};

pub struct AppState {
    pub db: Database,
    pub search: SearchEngine,
    pub indexer: Arc<Indexer>,
}

#[tauri::command]
pub async fn search_files(
    query: SearchQuery,
    state: State<'_, AppState>,
) -> Result<SearchResult, String> {
    let start_time = std::time::Instant::now();
    let limit = query.limit.unwrap_or(100).min(500);

    let mut tantivy_res = state.search.search(&query).unwrap_or_else(|_| SearchResult {
        items: Vec::new(),
        total_matched: 0,
        elapsed_ms: 0.0,
    });

    // If Tantivy returned fewer results than limit and query isn't empty, supplement with SQLite substring match
    if tantivy_res.items.len() < limit && !query.query.trim().is_empty() {
        if let Ok(sql_items) = state.db.search(&query) {
            let mut seen_paths = std::collections::HashSet::new();
            for item in &tantivy_res.items {
                seen_paths.insert(item.path.clone());
            }

            for item in sql_items {
                if !seen_paths.contains(&item.path) {
                    seen_paths.insert(item.path.clone());
                    tantivy_res.items.push(item);
                    if tantivy_res.items.len() >= limit {
                        break;
                    }
                }
            }
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
pub async fn reveal_item(path: String) -> Result<(), String> {
    reveal_in_explorer(&path)
}

#[tauri::command]
pub async fn get_recent_files(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<FileItem>, String> {
    let lim = limit.unwrap_or(20);
    state.db.query_recent(lim).map_err(|e| e.to_string())
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
