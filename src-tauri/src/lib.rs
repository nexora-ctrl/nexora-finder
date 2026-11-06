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
    let last_shown = Arc::new(parking_lot::Mutex::new(std::time::Instant::now()));
    let last_shown_single = last_shown.clone();
    let last_shown_evt = last_shown.clone();
    let last_shown_toggle = last_shown.clone();
    let last_shown_tray = last_shown.clone();
    let last_shown_setup = last_shown.clone();

    tauri::Builder::default()
        // 1. Single Instance plugin MUST be registered FIRST so secondary instances forward args & exit before touching Tantivy
        .plugin(tauri_plugin_single_instance::init(move |app, args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                *last_shown_single.lock() = std::time::Instant::now();
                show_and_focus(&window);

                // Find directory path passed from Windows Explorer context menu
                let target_path = args.iter().skip(1).find(|arg| !arg.starts_with('-')).map(|arg| {
                    arg.trim().trim_matches('"').trim().to_string()
                });

                if let Some(folder_path) = target_path {
                    if !folder_path.is_empty() {
                        let _ = window.emit("open-folder-filter", folder_path);
                    } else {
                        let _ = window.emit("focus-search", ());
                        let _ = window.emit("window-shown", ());
                    }
                } else {
                    let _ = window.emit("focus-search", ());
                    let _ = window.emit("window-shown", ());
                }
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            toggle_window(&window, &last_shown_toggle);
                        }
                    }
                })
                .build(),
        )
        // Auto-hide the spotlight window when the user clicks outside it (window loses focus)
        // Add a 400ms grace period after showing so window handover doesn't instantly close it
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                if window.label() == "main" {
                    if last_shown_evt.lock().elapsed() > std::time::Duration::from_millis(400) {
                        let _ = window.hide();
                    }
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
            // Initialize Database, Tantivy, Indexer and Watcher here (ONLY executed by primary instance!)
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
            app.manage(state);

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

            let last_shown_tray_clone = last_shown_tray.clone();
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .tooltip("Nexora Finder (Ctrl+Alt+S)")
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                toggle_window(&window, &last_shown_tray_clone);
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
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            toggle_window(&window, &last_shown_tray);
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
            let initial_folder = cli_args.iter().skip(1).find(|arg| !arg.starts_with('-')).map(|arg| {
                arg.trim().trim_matches('"').trim().to_string()
            });

            if let (Some(folder_path), Some(window)) = (initial_folder, app.get_webview_window("main")) {
                if !folder_path.is_empty() {
                    *last_shown_setup.lock() = std::time::Instant::now();
                    show_and_focus(&window);
                    let w = window.clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
                        let _ = w.emit("open-folder-filter", folder_path);
                    });
                }
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

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn AttachThreadInput(id_attach: u32, id_attach_to: u32, f_attach: i32) -> i32;
}

fn show_and_focus(window: &tauri::WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    position_spotlight(window);
    let _ = window.set_focus();

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            SetForegroundWindow, BringWindowToTop, ShowWindow, SW_RESTORE,
            GetForegroundWindow, GetWindowThreadProcessId,
        };
        use windows::Win32::System::Threading::GetCurrentThreadId;
        use windows::Win32::Foundation::HWND;

        if let Ok(hwnd) = window.hwnd() {
            unsafe {
                let win_hwnd = HWND(hwnd.0 as _);
                let fg_hwnd = GetForegroundWindow();
                let mut fg_pid = 0;
                let fg_thread = GetWindowThreadProcessId(fg_hwnd, Some(&mut fg_pid));
                let cur_thread = GetCurrentThreadId();

                if fg_thread != 0 && fg_thread != cur_thread {
                    let _ = AttachThreadInput(cur_thread, fg_thread, 1);
                    let _ = ShowWindow(win_hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(win_hwnd);
                    let _ = BringWindowToTop(win_hwnd);
                    let _ = AttachThreadInput(cur_thread, fg_thread, 0);
                } else {
                    let _ = ShowWindow(win_hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(win_hwnd);
                    let _ = BringWindowToTop(win_hwnd);
                }
            }
        }
    }
}

fn toggle_window(window: &tauri::WebviewWindow, last_shown: &Arc<parking_lot::Mutex<std::time::Instant>>) {
    let is_visible = window.is_visible().unwrap_or(false);
    let is_focused = window.is_focused().unwrap_or(false);

    if is_visible && is_focused {
        let _ = window.hide();
    } else {
        *last_shown.lock() = std::time::Instant::now();
        show_and_focus(window);
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
            let bg_command_str = format!("\"{}\" \"%V\"", exe_str);

            let targets = [
                ("Software\\Classes\\Directory\\shell\\NexoraFinder", &command_str),
                ("Software\\Classes\\Directory\\Background\\shell\\NexoraFinder", &bg_command_str),
                ("Software\\Classes\\Drive\\shell\\NexoraFinder", &command_str),
                ("Software\\Classes\\Folder\\shell\\NexoraFinder", &command_str),
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
