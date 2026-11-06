use std::io::Cursor;
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

#[cfg(target_os = "windows")]
use windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::SIZE;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::HBITMAP;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::{
    SHCreateItemFromParsingName, IShellItemImageFactory, SIIGBF_RESIZETOFIT, SIIGBF_BIGGERSIZEOK
};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    GetObjectW, GetDIBits, GetDC, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, DeleteObject
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

#[derive(Clone)]
pub struct ThumbnailCache {
    cache: Arc<Mutex<HashMap<String, String>>>,
}

impl ThumbnailCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::with_capacity(500))),
        }
    }

    pub fn get_cached(&self, path: &str) -> Option<String> {
        self.cache.lock().get(path).cloned()
    }

    pub fn insert(&self, path: String, data_uri: String) {
        let mut map = self.cache.lock();
        if map.len() > 600 {
            map.clear(); // Simple bounded cache reset
        }
        map.insert(path, data_uri);
    }
}

pub fn get_file_thumbnail(path: &str, target_size: u32) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Try Windows Shell Image Factory first (supports cached system thumbnails for images, videos, pdfs, apps)
        match extract_windows_thumbnail(path, target_size) {
            Ok(base64_str) => return Ok(base64_str),
            Err(_) => {}
        }
    }

    // Fallback: If image file, try decoding using `image` crate directly
    extract_image_thumbnail_fallback(path, target_size)
}

#[cfg(target_os = "windows")]
fn extract_windows_thumbnail(path: &str, target_size: u32) -> Result<String, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let item_factory: IShellItemImageFactory = SHCreateItemFromParsingName(
            PCWSTR(wide_path.as_ptr()),
            None,
        ).map_err(|e| format!("Failed to create shell item: {}", e))?;

        let size = SIZE {
            cx: target_size as i32,
            cy: target_size as i32,
        };

        // SIIGBF_RESIZETOFIT | SIIGBF_BIGGERSIZEOK
        let flags = SIIGBF_RESIZETOFIT | SIIGBF_BIGGERSIZEOK;
        let hbitmap: HBITMAP = item_factory
            .GetImage(size, flags)
            .map_err(|e| format!("Failed to get image: {}", e))?;

        let res = hbitmap_to_png_base64(hbitmap);
        let _ = DeleteObject(hbitmap);
        let _ = CoUninitialize();
        res
    }
}

#[cfg(target_os = "windows")]
unsafe fn hbitmap_to_png_base64(hbitmap: HBITMAP) -> Result<String, String> {
    let mut bm: BITMAP = std::mem::zeroed();
    let bm_size = std::mem::size_of::<BITMAP>() as i32;
    if GetObjectW(hbitmap, bm_size, Some(&mut bm as *mut _ as *mut _)) == 0 {
        return Err("GetObjectW failed".into());
    }

    let width = bm.bmWidth;
    let height = bm.bmHeight;
    if width <= 0 || height <= 0 {
        return Err("Invalid bitmap dimensions".into());
    }

    let hdc = GetDC(None);
    if hdc.0.is_null() {
        return Err("GetDC failed".into());
    }

    let mut bmi: BITMAPINFO = std::mem::zeroed();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = -height; // top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB.0;

    let buf_size = (width * height * 4) as usize;
    let mut pixels = vec![0u8; buf_size];

    let lines = GetDIBits(
        hdc,
        hbitmap,
        0,
        height as u32,
        Some(pixels.as_mut_ptr() as *mut _),
        &mut bmi,
        DIB_RGB_COLORS,
    );

    let _ = ReleaseDC(None, hdc);

    if lines == 0 {
        return Err("GetDIBits failed".into());
    }

    // Convert BGRA to RGBA
    for chunk in pixels.chunks_exact_mut(4) {
        let b = chunk[0];
        let r = chunk[2];
        chunk[0] = r;
        chunk[2] = b;
    }

    // Encode to PNG using image crate
    let img = image::RgbaImage::from_raw(width as u32, height as u32, pixels)
        .ok_or_else(|| "Failed to construct RgbaImage".to_string())?;

    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode png: {}", e))?;

    let b64 = BASE64.encode(&png_bytes);
    Ok(format!("data:image/png;base64,{}", b64))
}

fn extract_image_thumbnail_fallback(path: &str, target_size: u32) -> Result<String, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open image file: {}", e))?;
    let thumbnail = img.thumbnail(target_size, target_size);

    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    thumbnail.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode fallback thumbnail: {}", e))?;

    let b64 = BASE64.encode(&png_bytes);
    Ok(format!("data:image/png;base64,{}", b64))
}
