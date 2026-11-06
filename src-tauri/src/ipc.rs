use std::sync::Arc;
use tauri::{Emitter, Manager};

const PIPE_NAME: &str = r"\\.\pipe\nexora_finder_single_instance";

/// Try to forward command line arguments to an existing running instance.
/// Returns true if an existing instance was contacted, false otherwise.
pub fn try_forward_to_existing_instance() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(PIPE_NAME) {
            let cli_args: Vec<String> = std::env::args().collect();
            let target_path = cli_args
                .iter()
                .skip(1)
                .find(|arg| !arg.starts_with('-'))
                .map(|arg| arg.trim().trim_matches('"').trim().to_string())
                .unwrap_or_default();

            let _ = file.write_all(target_path.as_bytes());
            let _ = file.flush();
            return true;
        }
    }
    false
}

/// Start background Named Pipe server in the primary instance to listen for incoming directory requests.
pub fn start_ipc_server(
    app_handle: tauri::AppHandle,
    last_shown: Arc<parking_lot::Mutex<std::time::Instant>>,
) {
    #[cfg(target_os = "windows")]
    {
        tauri::async_runtime::spawn(async move {
            use tokio::net::windows::named_pipe::ServerOptions;
            use tokio::io::AsyncReadExt;

            let mut server = match ServerOptions::new()
                .first_pipe_instance(true)
                .create(PIPE_NAME)
            {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to create first named pipe server: {:?}", e);
                    return;
                }
            };

            loop {
                if server.connect().await.is_ok() {
                    let mut buf = vec![0u8; 4096];
                    if let Ok(n) = server.read(&mut buf).await {
                        if n > 0 {
                            let folder_path = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                            if let Some(window) = app_handle.get_webview_window("main") {
                                *last_shown.lock() = std::time::Instant::now();
                                crate::show_and_focus(&window);

                                if !folder_path.is_empty() {
                                    let _ = window.emit("open-folder-filter", folder_path);
                                } else {
                                    let _ = window.emit("focus-search", ());
                                    let _ = window.emit("window-shown", ());
                                }
                            }
                        }
                    }
                }

                // Prepare next pipe instance
                server = match ServerOptions::new().create(PIPE_NAME) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to create subsequent named pipe: {:?}", e);
                        break;
                    }
                };
            }
        });
    }
}

