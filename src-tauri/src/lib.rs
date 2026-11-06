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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
                position_spotlight(&window);

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

                // Check if a path argument was passed
                let target_path = args.iter().skip(1).find(|arg| !arg.starts_with('-')).cloned();
                if let Some(folder_path) = target_path {
                    let _ = window.emit("open-folder-filter", folder_path);
                } else {
                    let _ = window.emit("focus-search", ());
                    let _ = window.emit("window-shown", ());
                }
            }
        }))
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
            commands::open_item_with,
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
            let show_item = MenuItem::with_id(app, "show", "Buka Nexora Finder (Ctrl+Alt+S)", true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, "hide", "Sembunyikan ke Tray", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Keluar", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .tooltip("Nexora Finder (Ctrl+Alt+S)")
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

            // Register Explorer right-click context menu "Open with N-Finder"
            register_explorer_context_menu();

            // Check if app was started directly with a directory path
            let cli_args: Vec<String> = std::env::args().collect();
            let initial_folder = cli_args.iter().skip(1).find(|arg| !arg.starts_with('-')).cloned();
            if let (Some(folder_path), Some(window)) = (initial_folder, app.get_webview_window("main")) {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
                position_spotlight(&window);
                let w = window.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let _ = w.emit("open-folder-filter", folder_path);
                });
            }

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
        let _ = window.emit("window-shown", ());
    }
}

fn register_explorer_context_menu() {
    #[cfg(target_os = "windows")]
    {
        if let Ok(exe_path) = std::env::current_exe() {
            let exe_str = exe_path.to_string_lossy().to_string();
            let icon_str = format!("{},0", exe_str);
            let command_str = format!("\"{}\" \"%1\"", exe_str);
            let bg_command_str = format!("\"{}\"", exe_str);

            let targets = [
                ("Software\\Classes\\Directory\\shell\\NexoraFinder", &command_str),
                ("Software\\Classes\\Directory\\Background\\shell\\NexoraFinder", &bg_command_str),
                ("Software\\Classes\\Drive\\shell\\NexoraFinder", &command_str),
            ];

            for (sub_key, cmd) in targets {
                if let Ok(key) = windows_registry::CURRENT_USER.create(sub_key) {
                    let _ = key.set_string("", "Open with N-Finder");
                    let _ = key.set_string("Icon", &icon_str);
                }
                let cmd_key_path = format!("{}\\command", sub_key);
                if let Ok(cmd_key) = windows_registry::CURRENT_USER.create(&cmd_key_path) {
                    let _ = cmd_key.set_string("", cmd);
                }
            }
        }
    }
}

