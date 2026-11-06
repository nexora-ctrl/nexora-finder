mod actions;
mod commands;
mod db;
mod drive;
mod indexer;
mod search;
mod thumbnail;
mod types;
mod usn;
mod watcher;

use std::sync::Arc;
use tauri::{menu::{Menu, MenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use commands::AppState;
use db::Database;
use indexer::Indexer;
use search::SearchEngine;
use watcher::FileWatcher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = Database::new().expect("Failed to initialize SQLite database");
    let search = SearchEngine::new().expect("Failed to initialize Tantivy search engine");
    let indexer = Arc::new(Indexer::new(db.clone(), search.clone()));

    // Start background filesystem watcher
    let _ = FileWatcher::start(db.clone(), search.clone());

    let state = AppState {
        db: db.clone(),
        search: search.clone(),
        indexer: indexer.clone(),
        thumbnails: thumbnail::ThumbnailCache::new(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            toggle_window(&window);
                        }
                    }
                })
                .build(),
        )
        .manage(state)
        // Auto-hide the spotlight window when the user clicks outside it (window loses focus)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                if window.label() == "main" {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_files,
            commands::get_indexing_status,
            commands::start_indexing,
            commands::open_item,
            commands::reveal_item,
            commands::get_thumbnail,
            commands::get_recent_files,
            commands::get_drives,
            commands::get_stats,
            commands::hide_window,
            commands::check_usn_status,
            commands::set_window_compact,
            commands::get_indexed_drives,
            commands::set_indexed_drives,
        ])
        .setup(move |app| {
            // Register primary shortcut: Ctrl+Alt+S (requested) and Alt+Space
            for s in ["Ctrl+Alt+S", "Alt+Space", "Ctrl+Space"] {
                match s.parse::<Shortcut>() {
                    Ok(shortcut) => {
                        match app.global_shortcut().register(shortcut) {
                            Ok(_) => println!(">>> Registered shortcut: {}", s),
                            Err(e) => eprintln!(">>> Failed to register {}: {:?}", s, e),
                        }
                    }
                    Err(e) => eprintln!(">>> Failed to parse {}: {:?}", s, e),
                }
            }

            // Create System Tray
            let show_item = MenuItem::with_id(app, "show", "Buka Nexora Search (Ctrl+Alt+S)", true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, "hide", "Sembunyikan ke Tray", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Keluar", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .tooltip("Nexora Search (Ctrl+Alt+S)")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                toggle_window(&window);
                            }
                        }
                        "hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            toggle_window(&window);
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _ = tray_builder.build(app)?;

            // If indexing never completed or index is empty, initiate background indexing
            let completed = db.get_metadata("indexing_completed").unwrap_or(None);
            let count = db.get_total_count().unwrap_or(0);
            if completed.as_deref() != Some("true") || count == 0 {
                let app_handle = app.handle().clone();
                indexer.start_indexing(app_handle);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn position_spotlight(window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let scale = window.scale_factor().unwrap_or(1.0);
        let mon_pos = monitor.position();
        let mon_size = monitor.size();
        let mon_w = mon_size.width as f64 / scale;
        let mon_h = mon_size.height as f64 / scale;
        let cur_h = if let Ok(sz) = window.inner_size() {
            sz.to_logical::<f64>(scale).height
        } else {
            58.0
        };
        let x = (mon_pos.x as f64 / scale) + (mon_w - 800.0) / 2.0;
        let y = (mon_pos.y as f64 / scale) + (mon_h - cur_h) / 2.0;
        let _ = window.set_position(tauri::LogicalPosition::new(x, y));
    } else {
        let _ = window.center();
    }
}

fn toggle_window(window: &tauri::WebviewWindow) {
    let is_visible = window.is_visible().unwrap_or(false);
    let is_focused = window.is_focused().unwrap_or(false);

    if is_visible && is_focused {
        let _ = window.hide();
    } else {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        position_spotlight(window);

        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::{SetForegroundWindow, BringWindowToTop, ShowWindow, SW_RESTORE};
            use windows::Win32::Foundation::HWND;
            if let Ok(hwnd) = window.hwnd() {
                unsafe {
                    let _ = ShowWindow(HWND(hwnd.0 as _), SW_RESTORE);
                    let _ = SetForegroundWindow(HWND(hwnd.0 as _));
                    let _ = BringWindowToTop(HWND(hwnd.0 as _));
                }
            }
        }

        let _ = window.emit("focus-search", ());
    }
}
