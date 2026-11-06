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

extern "system" {
    fn LoadLibraryW(lpLibFileName: *const u16) -> isize;
    fn GetProcAddress(hModule: isize, lpProcName: *const u8) -> usize;
}

type OpenAsFn = unsafe extern "system" fn(hwnd: isize, hinst: isize, lpsz_cmd_line: *const u16, n_cmd_show: i32);

pub fn open_file_with(path: &str) -> Result<(), String> {
    let wide_dll: Vec<u16> = "shell32.dll".encode_utf16().chain(std::iter::once(0)).collect();
    let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let hmodule = LoadLibraryW(wide_dll.as_ptr());
        if hmodule != 0 {
            let proc = GetProcAddress(hmodule, b"OpenAs_RunDLLW\0".as_ptr());
            if proc != 0 {
                let open_as: OpenAsFn = std::mem::transmute(proc);
                // Call on a background thread so it never blocks the async runtime
                std::thread::spawn(move || {
                    open_as(0, 0, wide_path.as_ptr(), 1);
                });
                return Ok(());
            }
        }
    }

    // Fallback: rundll32
    Command::new("rundll32.exe")
        .args(["shell32.dll,OpenAs_RunDLL", path])
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to open with dialog: {}", e))
}

pub fn reveal_in_explorer(path: &str) -> Result<(), String> {
    let formatted_arg = format!("/select,\"{}\"", path);
    Command::new("explorer.exe")
        .arg(formatted_arg)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to reveal in explorer: {}", e))
}

