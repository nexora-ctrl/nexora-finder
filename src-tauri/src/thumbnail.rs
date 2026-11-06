use std::io::Cursor;
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

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
            map.clear();
        }
        map.insert(path, data_uri);
    }
}

pub fn get_file_thumbnail(path: &str, target_size: u32) -> Result<String, String> {
    // Read file bytes with a reasonable limit for image decode
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "bmp" | "webp" | "gif" | "ico" => {
            extract_image_thumbnail(path, target_size)
        }
        _ => Err("Unsupported thumbnail file type".into()),
    }
}

fn extract_image_thumbnail(path: &str, target_size: u32) -> Result<String, String> {
    // Open image directly
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
    let thumb = img.thumbnail_exact(target_size, target_size);

    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    thumb.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to write thumbnail: {}", e))?;

    let b64 = BASE64.encode(&png_bytes);
    Ok(format!("data:image/png;base64,{}", b64))
}
