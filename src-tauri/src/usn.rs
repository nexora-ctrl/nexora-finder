use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use std::ffi::OsString;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
    OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{
    FSCTL_QUERY_USN_JOURNAL, FSCTL_READ_USN_JOURNAL,
    USN_JOURNAL_DATA_V0, READ_USN_JOURNAL_DATA_V0,
};
use windows::Win32::System::IO::DeviceIoControl;

#[derive(Debug, Clone)]
pub struct UsnChange {
    pub usn: i64,
    pub filename: String,
    pub reason: u32,
    pub file_reference_number: u64,
    pub parent_file_reference_number: u64,
}

#[derive(Debug, Clone)]
pub struct UsnJournalInfo {
    pub journal_id: u64,
    pub first_usn: i64,
    pub next_usn: i64,
    pub lowest_valid_usn: i64,
    pub max_usn: i64,
    pub maximum_size: u64,
    pub allocation_delta: u64,
}

pub struct UsnJournal {
    handle: HANDLE,
    pub drive_letter: String,
}

impl UsnJournal {
    pub fn open(drive_letter: &str) -> Result<Self, String> {
        let clean_drive = drive_letter.trim_end_matches(['\\', ':']);
        let volume_path = format!("\\\\.\\{}:", clean_drive);
        let wide_path: Vec<u16> = volume_path.encode_utf16().chain(std::iter::once(0)).collect();

        let handle = unsafe {
            CreateFileW(
                PCWSTR(wide_path.as_ptr()),
                0x80000000 | 0x40000000, // GENERIC_READ | GENERIC_WRITE
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                HANDLE(std::ptr::null_mut()),
            )
        };

        if let Ok(h) = handle {
            if h == INVALID_HANDLE_VALUE {
                return Err(format!("Cannot open volume {}: handle is invalid", drive_letter));
            }
            Ok(Self {
                handle: h,
                drive_letter: clean_drive.to_string(),
            })
        } else {
            Err(format!(
                "USN Journal for drive {}: requires Administrator privileges or NTFS volume",
                drive_letter
            ))
        }
    }

    pub fn query_info(&self) -> Result<UsnJournalInfo, String> {
        let mut journal_data = USN_JOURNAL_DATA_V0::default();
        let mut bytes_returned = 0u32;

        let success = unsafe {
            DeviceIoControl(
                self.handle,
                FSCTL_QUERY_USN_JOURNAL,
                None,
                0,
                Some(&mut journal_data as *mut _ as *mut std::ffi::c_void),
                size_of::<USN_JOURNAL_DATA_V0>() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        if success.is_ok() {
            Ok(UsnJournalInfo {
                journal_id: journal_data.UsnJournalID,
                first_usn: journal_data.FirstUsn,
                next_usn: journal_data.NextUsn,
                lowest_valid_usn: journal_data.LowestValidUsn,
                max_usn: journal_data.MaxUsn,
                maximum_size: journal_data.MaximumSize,
                allocation_delta: journal_data.AllocationDelta,
            })
        } else {
            Err("Failed to query USN journal information via DeviceIoControl".to_string())
        }
    }

    pub fn read_changes(&self, start_usn: i64) -> Result<(Vec<UsnChange>, i64), String> {
        let mut read_data = READ_USN_JOURNAL_DATA_V0 {
            StartUsn: start_usn,
            ReasonMask: 0xFFFFFFFF,
            ReturnOnlyOnClose: 0,
            Timeout: 0,
            BytesToWaitFor: 0,
            UsnJournalID: 0,
        };

        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_returned = 0u32;

        let success = unsafe {
            DeviceIoControl(
                self.handle,
                FSCTL_READ_USN_JOURNAL,
                Some(&mut read_data as *mut _ as *mut std::ffi::c_void),
                size_of::<READ_USN_JOURNAL_DATA_V0>() as u32,
                Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
                buffer.len() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        if success.is_err() || bytes_returned < 8 {
            return Ok((Vec::new(), start_usn));
        }

        // First 8 bytes of output buffer contains next USN
        let next_usn = i64::from_le_bytes(buffer[0..8].try_into().unwrap_or([0; 8]));
        let mut changes = Vec::new();
        let mut offset = 8usize;

        while offset + 60 <= bytes_returned as usize {
            let record_length = u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            if record_length == 0 {
                break;
            }

            let file_ref = u64::from_le_bytes(buffer[offset + 8..offset + 16].try_into().unwrap_or([0; 8]));
            let parent_ref = u64::from_le_bytes(buffer[offset + 16..offset + 24].try_into().unwrap_or([0; 8]));
            let usn = i64::from_le_bytes(buffer[offset + 24..offset + 32].try_into().unwrap_or([0; 8]));
            let reason = u32::from_le_bytes(buffer[offset + 40..offset + 44].try_into().unwrap_or([0; 4]));
            let filename_len = u16::from_le_bytes(buffer[offset + 56..offset + 58].try_into().unwrap_or([0; 2])) as usize;
            let filename_offset = u16::from_le_bytes(buffer[offset + 58..offset + 60].try_into().unwrap_or([0; 2])) as usize;

            if offset + filename_offset + filename_len <= bytes_returned as usize {
                let name_bytes = &buffer[offset + filename_offset..offset + filename_offset + filename_len];
                let u16_chars: Vec<u16> = name_bytes
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                let filename = OsString::from_wide(&u16_chars).to_string_lossy().to_string();

                changes.push(UsnChange {
                    usn,
                    filename,
                    reason,
                    file_reference_number: file_ref,
                    parent_file_reference_number: parent_ref,
                });
            }

            offset += record_length;
        }

        Ok((changes, next_usn))
    }
}

impl Drop for UsnJournal {
    fn drop(&mut self) {
        if self.handle != INVALID_HANDLE_VALUE {
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

