use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
};
use crate::types::DriveInfo;

pub fn get_available_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    unsafe {
        let bitmask = GetLogicalDrives();
        for i in 0..26 {
            if (bitmask & (1 << i)) != 0 {
                let drive_letter = (b'A' + i as u8) as char;
                let root_path_str = format!("{}:\\", drive_letter);
                let wide_path: Vec<u16> = root_path_str.encode_utf16().chain(std::iter::once(0)).collect();

                let drive_type_raw = GetDriveTypeW(PCWSTR(wide_path.as_ptr()));
                let drive_type = match drive_type_raw {
                    2 => "Removable",
                    3 => "Fixed",
                    4 => "Network",
                    5 => "CD-ROM",
                    6 => "RAM Disk",
                    _ => "Unknown",
                };

                // Only inspect Fixed (3) or Removable (2) drives
                if drive_type_raw != 3 && drive_type_raw != 2 {
                    continue;
                }

                let mut vol_name = [0u16; 261];
                let mut fs_name = [0u16; 261];
                let mut serial = 0u32;
                let mut max_component = 0u32;
                let mut flags = 0u32;

                let success = GetVolumeInformationW(
                    PCWSTR(wide_path.as_ptr()),
                    Some(&mut vol_name),
                    Some(&mut serial),
                    Some(&mut max_component),
                    Some(&mut flags),
                    Some(&mut fs_name),
                ).is_ok();

                let label = if success {
                    let len = vol_name.iter().position(|&c| c == 0).unwrap_or(vol_name.len());
                    OsString::from_wide(&vol_name[..len]).to_string_lossy().to_string()
                } else {
                    String::new()
                };

                let fs_str = if success {
                    let len = fs_name.iter().position(|&c| c == 0).unwrap_or(fs_name.len());
                    OsString::from_wide(&fs_name[..len]).to_string_lossy().to_string()
                } else {
                    String::new()
                };

                let is_ntfs = fs_str.eq_ignore_ascii_case("NTFS");

                let mut free_bytes_avail = 0u64;
                let mut total_bytes = 0u64;
                let mut total_free = 0u64;

                let space_success = GetDiskFreeSpaceExW(
                    PCWSTR(wide_path.as_ptr()),
                    Some(&mut free_bytes_avail),
                    Some(&mut total_bytes),
                    Some(&mut total_free),
                ).is_ok();

                let (total, free) = if space_success {
                    (total_bytes, total_free)
                } else {
                    (0, 0)
                };

                drives.push(DriveInfo {
                    letter: format!("{}:", drive_letter),
                    label: if label.is_empty() { format!("Local Disk ({}:)", drive_letter) } else { label },
                    drive_type: drive_type.to_string(),
                    total_space_bytes: total,
                    free_space_bytes: free,
                    is_ntfs,
                });
            }
        }
    }

    // Sort drives: index user data drives (D:, E:, etc.) first before system drive C:
    drives.sort_by(|a, b| {
        let a_is_c = a.letter.eq_ignore_ascii_case("C:");
        let b_is_c = b.letter.eq_ignore_ascii_case("C:");
        match (a_is_c, b_is_c) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a.letter.cmp(&b.letter),
        }
    });

    drives
}
