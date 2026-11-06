use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use rusqlite::{params, Connection, Result};
use crate::types::{DriveInfo, FileItem, IndexStats, SearchQuery};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    pub db_path: PathBuf,
}

impl Database {
    pub fn new() -> Result<Self> {
        let app_dir = directories::ProjectDirs::from("com", "nexora", "NexoraFinder")
            .map(|dirs| dirs.data_local_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./data"));

        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("nexora.db");

        let conn = Connection::open(&db_path)?;

        // Optimize SQLite for high-throughput reads & writes
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -64000;
             PRAGMA foreign_keys = OFF;",
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                filename TEXT NOT NULL,
                extension TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                modified_at INTEGER NOT NULL,
                is_directory INTEGER NOT NULL,
                drive TEXT NOT NULL
            );",
            [],
        )?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_filename ON files(filename);", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_ext ON files(extension);", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_drive ON files(drive);", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_modified ON files(modified_at DESC);", [])?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    pub fn upsert_batch(&self, items: &[FileItem]) -> Result<usize> {
        if items.is_empty() {
            return Ok(0);
        }

        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO files (path, filename, extension, size_bytes, modified_at, is_directory, drive)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(path) DO UPDATE SET
                    filename = excluded.filename,
                    extension = excluded.extension,
                    size_bytes = excluded.size_bytes,
                    modified_at = excluded.modified_at,
                    is_directory = excluded.is_directory,
                    drive = excluded.drive;",
            )?;

            for item in items {
                stmt.execute(params![
                    item.path,
                    item.filename,
                    item.extension,
                    item.size_bytes as i64,
                    item.modified_at,
                    if item.is_directory { 1 } else { 0 },
                    item.drive
                ])?;
            }
        }

        tx.commit()?;
        Ok(items.len())
    }

    pub fn delete_by_path(&self, path: &str) -> Result<bool> {
        let conn = self.conn.lock();
        let affected = conn.execute("DELETE FROM files WHERE path = ?1;", params![path])?;
        Ok(affected > 0)
    }

    pub fn get_total_count(&self) -> Result<u64> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM files;", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    pub fn query_recent(&self, limit: usize) -> Result<Vec<FileItem>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, filename, extension, size_bytes, modified_at, is_directory, drive
             FROM files
             ORDER BY modified_at DESC
             LIMIT ?1;",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let is_dir_int: i32 = row.get(6)?;
            let size_int: i64 = row.get(4)?;
            Ok(FileItem {
                id: Some(row.get(0)?),
                path: row.get(1)?,
                filename: row.get(2)?,
                extension: row.get(3)?,
                size_bytes: size_int.max(0) as u64,
                modified_at: row.get(5)?,
                is_directory: is_dir_int != 0,
                drive: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn search(&self, q: &SearchQuery) -> Result<Vec<FileItem>> {
        let conn = self.conn.lock();
        let query_str = q.query.trim();
        let limit = q.limit.unwrap_or(100).min(500);

        let mut sql = String::from(
            "SELECT id, path, filename, extension, size_bytes, modified_at, is_directory, drive
             FROM files
             WHERE 1=1"
        );

        let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

        let parsed = crate::types::parse_search_query_text(query_str);

        // 1. Match all text keywords (AND logic for multi-word search)
        for kw in &parsed.keywords {
            let pattern = format!("%{}%", kw);
            sql.push_str(" AND (filename LIKE ? OR path LIKE ?)");
            params_vec.push(rusqlite::types::Value::Text(pattern.clone()));
            params_vec.push(rusqlite::types::Value::Text(pattern));
        }

        // 2. Match extension from query (e.g. ".doc" -> doc, docx)
        if let Some(ext) = &parsed.extension {
            let ext_pattern = format!("{}%", ext);
            let fn_pattern = format!("%.{}%", ext);
            sql.push_str(" AND (extension LIKE ? OR filename LIKE ?)");
            params_vec.push(rusqlite::types::Value::Text(ext_pattern));
            params_vec.push(rusqlite::types::Value::Text(fn_pattern));
        }

        // 3. Match Drive filter (either from parsed query "d:" or UI filter)
        let effective_drive = parsed.drive.as_ref().or(q.drive.as_ref());
        if let Some(drive) = effective_drive {
            if !drive.is_empty() && drive != "ALL" {
                sql.push_str(" AND drive = ?");
                params_vec.push(rusqlite::types::Value::Text(drive.clone()));
            }
        }

        // 4. Match locked folder scope
        if let Some(folder) = &q.folder {
            let norm_folder = folder.replace('/', "\\");
            let pattern = if norm_folder.ends_with('\\') {
                format!("{}%", norm_folder)
            } else {
                format!("{}\\%", norm_folder)
            };
            sql.push_str(" AND path LIKE ?");
            params_vec.push(rusqlite::types::Value::Text(pattern));
        }

        if let Some(cat) = &q.category {
            let cat_lower = cat.to_lowercase();
            if cat_lower == "folder" {
                sql.push_str(" AND is_directory = 1");
            } else {
                let extensions: &[&str] = match cat_lower.as_str() {
                    "document" => &["pdf", "doc", "docx", "txt", "xlsx", "xls", "pptx", "ppt", "csv", "md"],
                    "image" => &["png", "jpg", "jpeg", "webp", "gif", "bmp", "svg", "ico"],
                    "video" => &["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv"],
                    "audio" => &["mp3", "wav", "flac", "aac", "ogg", "m4a"],
                    "app" => &["exe", "msi", "bat", "cmd", "ps1", "lnk"],
                    "archive" => &["zip", "rar", "7z", "tar", "gz", "iso"],
                    "code" => &["rs", "ts", "tsx", "js", "jsx", "py", "html", "css", "json", "c", "cpp"],
                    _ => &[],
                };

                if !extensions.is_empty() {
                    let placeholders = extensions.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                    sql.push_str(&format!(" AND extension IN ({})", placeholders));
                    for ext in extensions {
                        params_vec.push(rusqlite::types::Value::Text(ext.to_string()));
                    }
                }
            }
        }

        sql.push_str(" ORDER BY is_directory DESC, filename ASC LIMIT ?");
        params_vec.push(rusqlite::types::Value::Integer(limit as i64));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), |row| {
            let is_dir_int: i32 = row.get(6)?;
            let size_int: i64 = row.get(4)?;
            Ok(FileItem {
                id: Some(row.get(0)?),
                path: row.get(1)?,
                filename: row.get(2)?,
                extension: row.get(3)?,
                size_bytes: size_int.max(0) as u64,
                modified_at: row.get(5)?,
                is_directory: is_dir_int != 0,
                drive: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT value FROM metadata WHERE key = ?1;")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn set_metadata(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO metadata (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value;",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_stats(&self, drives: Vec<DriveInfo>) -> Result<IndexStats> {
        let conn = self.conn.lock();
        let total_files: i64 = conn.query_row(
            "SELECT COUNT(*) FROM files WHERE is_directory = 0;",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let total_directories: i64 = conn.query_row(
            "SELECT COUNT(*) FROM files WHERE is_directory = 1;",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let total_size_bytes: i64 = conn.query_row(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM files WHERE is_directory = 0;",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let mut count_by_category = std::collections::HashMap::new();

        // Categorize by extensions
        let mut stmt = conn.prepare("SELECT extension, COUNT(*) FROM files GROUP BY extension;")?;
        let rows = stmt.query_map([], |row| {
            let ext: String = row.get(0)?;
            let cnt: i64 = row.get(1)?;
            Ok((ext, cnt))
        })?;

        for r in rows {
            if let Ok((ext, cnt)) = r {
                let cat = match ext.as_str() {
                    "pdf" | "doc" | "docx" | "txt" | "xlsx" | "xls" | "pptx" | "ppt" | "csv" | "md" => "document",
                    "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg" | "ico" => "image",
                    "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "flv" => "video",
                    "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => "audio",
                    "exe" | "msi" | "bat" | "cmd" | "ps1" | "lnk" => "app",
                    "zip" | "rar" | "7z" | "tar" | "gz" | "iso" => "archive",
                    "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "html" | "css" | "json" | "c" | "cpp" => "code",
                    _ => "other",
                };
                *count_by_category.entry(cat.to_string()).or_insert(0) += cnt as u64;
            }
        }

        Ok(IndexStats {
            total_files: total_files.max(0) as u64,
            total_directories: total_directories.max(0) as u64,
            total_size_bytes: total_size_bytes.max(0) as u64,
            count_by_category,
            drives,
        })
    }
}

