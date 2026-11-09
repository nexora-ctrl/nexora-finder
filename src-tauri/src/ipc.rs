use std::sync::Arc;
use tauri::{Emitter, Manager};

const PIPE_NAME: &str = r"\\.\pipe\nexora_finder_single_instance";

/// Clean and normalize folder paths from CLI args or Explorer context menu
pub fn clean_folder_path(raw: &str) -> String {
    let mut p = raw.trim().trim_matches('"').trim_matches('\'').trim().to_string();
    p = p.trim_matches('"').trim_matches('\'').trim().to_string();

    // If it's a drive like "C:" or "c:", normalize to "C:\"
    if p.len() == 2 && p.ends_with(':') && p.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
        return format!("{}:\\", p.chars().next().unwrap().to_ascii_uppercase());
    }

    // If it's a drive with slash like "c:\" or "c:/", normalize to "C:\"
    if p.len() == 3 && (p.ends_with('\\') || p.ends_with('/')) && p.chars().nth(1) == Some(':') {
        return format!("{}:\\", p.chars().next().unwrap().to_ascii_uppercase());
    }

    // Remove trailing backslash for regular folders (e.g. "D:\Rust\" -> "D:\Rust")
    if p.len() > 3 && (p.ends_with('\\') || p.ends_with('/')) {
        p.pop();
    }

    p
}

/// Try to forward command line arguments to an existing running instance.
/// Returns true if an existing instance was contacted, false otherwise.
pub fn try_forward_to_existing_instance() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::io::Write;
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Storage::FileSystem::FlushFileBuffers;

        if let Ok(mut file) = std::fs::OpenOptions::new().read(true).write(true).open(PIPE_NAME) {
            let cli_args: Vec<String> = std::env::args().collect();
            let target_path = cli_args
                .iter()
                .skip(1)
                .find(|arg| !arg.starts_with('-'))
                .map(|arg| clean_folder_path(arg))
                .unwrap_or_default();

            // Always write at least a newline so the server knows an instance connected
            let payload = format!("{}\n", target_path);
            let _ = file.write_all(payload.as_bytes());

            // Crucial: FlushFileBuffers ensures the pipe data is completely read by the server
            // before this client process exits! Without this, the process terminates and the pipe
            // is broken before the server can read the data.
            unsafe {
                let _ = FlushFileBuffers(HANDLE(file.as_raw_handle() as _));
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
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
                            let raw_str = String::from_utf8_lossy(&buf[..n]);
                            let folder_path = clean_folder_path(raw_str.trim());
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
                    let _ = server.disconnect();
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
