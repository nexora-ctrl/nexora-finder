use std::process::Command;
use windows::core::PCWSTR;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;

pub fn open_file(path: &str) -> Result<(), String> {
    let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_op: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();

    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(wide_op.as_ptr()),
            PCWSTR(wide_path.as_ptr()),
            None,
            None,
            SW_SHOW,
        )
    };

    if result.0 as usize > 32 {
        Ok(())
    } else {
        // Fallback using std::process::Command
        Command::new("cmd")
            .args(["/c", "start", "", path])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Failed to open file: {}", e))
    }
}

pub fn open_file_with(path: &str) -> Result<(), String> {
    let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_op: Vec<u16> = "openas".encode_utf16().chain(std::iter::once(0)).collect();

    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(wide_op.as_ptr()),
            PCWSTR(wide_path.as_ptr()),
            None,
            None,
            SW_SHOW,
        )
    };

    if result.0 as usize > 32 {
        Ok(())
    } else {
        // Fallback using rundll32 OpenAs_RunDLL
        Command::new("rundll32.exe")
            .args(["shell32.dll,OpenAs_RunDLL", path])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Failed to open with dialog: {}", e))
    }
}

pub fn reveal_in_explorer(path: &str) -> Result<(), String> {
    let formatted_arg = format!("/select,\"{}\"", path);
    Command::new("explorer.exe")
        .arg(formatted_arg)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to reveal in explorer: {}", e))
}

