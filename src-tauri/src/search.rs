use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, Query, TermQuery, RegexQuery};
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, Term};
use crate::types::{FileItem, SearchQuery, SearchResult};

#[derive(Clone)]
pub struct SearchEngine {
    index: Arc<Index>,
    reader: Arc<IndexReader>,
    writer: Arc<RwLock<IndexWriter>>,
    pub schema: Schema,
    pub f_id: Field,
    pub f_path: Field,
    pub f_filename: Field,
    pub f_extension: Field,
    pub f_drive: Field,
    pub f_size_bytes: Field,
    pub f_modified_at: Field,
    pub f_is_directory: Field,
}

impl SearchEngine {
    pub fn new() -> anyhow::Result<Self> {
        let app_dir = directories::ProjectDirs::from("com", "nexora", "NexoraFinder")
            .map(|dirs| dirs.data_local_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./data"));

        let index_path = app_dir.join("tantivy_index");
        std::fs::create_dir_all(&index_path)?;

        let mut schema_builder = Schema::builder();
        let f_id = schema_builder.add_u64_field("id", STORED | INDEXED);
        let f_path = schema_builder.add_text_field("path", STRING | STORED);
        let f_filename = schema_builder.add_text_field("filename", TEXT | STORED);
        let f_extension = schema_builder.add_text_field("extension", STRING | STORED);
        let f_drive = schema_builder.add_text_field("drive", STRING | STORED);
        let f_size_bytes = schema_builder.add_u64_field("size_bytes", STORED | FAST);
        let f_modified_at = schema_builder.add_i64_field("modified_at", STORED | FAST);
        let f_is_directory = schema_builder.add_u64_field("is_directory", STORED);

        let schema = schema_builder.build();

        let index = if index_path.join("meta.json").exists() {
            Index::open_in_dir(&index_path)?
        } else {
            Index::create_in_dir(&index_path, schema.clone())?
        };

        let writer = index.writer(50_000_000)?; // 50MB buffer
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index: Arc::new(index),
            reader: Arc::new(reader),
            writer: Arc::new(RwLock::new(writer)),
            schema,
            f_id,
            f_path,
            f_filename,
            f_extension,
            f_drive,
            f_size_bytes,
            f_modified_at,
            f_is_directory,
        })
    }

    pub fn add_batch(&self, items: &[FileItem]) -> anyhow::Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        let writer = self.writer.write();
        for item in items {
            let id_val = item.id.unwrap_or(0) as u64;
            writer.add_document(doc!(
                self.f_id => id_val,
                self.f_path => item.path.clone(),
                self.f_filename => item.filename.clone(),
                self.f_extension => item.extension.to_lowercase(),
                self.f_drive => item.drive.clone(),
                self.f_size_bytes => item.size_bytes,
                self.f_modified_at => item.modified_at,
                self.f_is_directory => if item.is_directory { 1u64 } else { 0u64 },
            ))?;
        }
        Ok(())
    }

    pub fn commit(&self) -> anyhow::Result<()> {
        let mut writer = self.writer.write();
        writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    pub fn delete_by_path(&self, path: &str) -> anyhow::Result<()> {
        let term = Term::from_field_text(self.f_path, path);
        let writer = self.writer.write();
        writer.delete_term(term);
        Ok(())
    }

    pub fn search(&self, q: &SearchQuery) -> anyhow::Result<SearchResult> {
        let start_time = std::time::Instant::now();
        let searcher = self.reader.searcher();
        let limit = q.limit.unwrap_or(100).min(500);

        let query_str = q.query.trim();

        let mut subqueries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        let parsed = crate::types::parse_search_query_text(query_str);

        if !parsed.keywords.is_empty() {
            for kw in &parsed.keywords {
                let clean_kw = kw.to_lowercase();
                let escaped_re = escape_regex(&clean_kw);
                let pattern = format!(".*{}.*", escaped_re);

                let parsed_q: Box<dyn Query> = match RegexQuery::from_pattern(&pattern, self.f_filename) {
                    Ok(rq) => Box::new(rq),
                    Err(_) => {
                        let term = Term::from_field_text(self.f_filename, &clean_kw);
                        Box::new(TermQuery::new(term, IndexRecordOption::Basic))
                    }
                };

                subqueries.push((Occur::Must, parsed_q));
            }
        }

        // Apply Extension filter extracted from query (e.g. ".doc" -> doc, docx)
        if let Some(ext) = &parsed.extension {
            let clean_ext = ext.to_lowercase();
            let mut ext_queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

            let primary_term = Term::from_field_text(self.f_extension, &clean_ext);
            ext_queries.push((Occur::Should, Box::new(TermQuery::new(primary_term, IndexRecordOption::Basic))));

            // Common multi-letter extension variants
            match clean_ext.as_str() {
                "doc" => {
                    let t = Term::from_field_text(self.f_extension, "docx");
                    ext_queries.push((Occur::Should, Box::new(TermQuery::new(t, IndexRecordOption::Basic))));
                }
                "docx" => {
                    let t = Term::from_field_text(self.f_extension, "doc");
                    ext_queries.push((Occur::Should, Box::new(TermQuery::new(t, IndexRecordOption::Basic))));
                }
                "xls" => {
                    let t = Term::from_field_text(self.f_extension, "xlsx");
                    ext_queries.push((Occur::Should, Box::new(TermQuery::new(t, IndexRecordOption::Basic))));
                }
                "xlsx" => {
                    let t = Term::from_field_text(self.f_extension, "xls");
                    ext_queries.push((Occur::Should, Box::new(TermQuery::new(t, IndexRecordOption::Basic))));
                }
                "ppt" => {
                    let t = Term::from_field_text(self.f_extension, "pptx");
                    ext_queries.push((Occur::Should, Box::new(TermQuery::new(t, IndexRecordOption::Basic))));
                }
                "pptx" => {
                    let t = Term::from_field_text(self.f_extension, "ppt");
                    ext_queries.push((Occur::Should, Box::new(TermQuery::new(t, IndexRecordOption::Basic))));
                }
                "jpg" => {
                    let t = Term::from_field_text(self.f_extension, "jpeg");
                    ext_queries.push((Occur::Should, Box::new(TermQuery::new(t, IndexRecordOption::Basic))));
                }
                "jpeg" => {
                    let t = Term::from_field_text(self.f_extension, "jpg");
                    ext_queries.push((Occur::Should, Box::new(TermQuery::new(t, IndexRecordOption::Basic))));
                }
                _ => {}
            }

            subqueries.push((Occur::Must, Box::new(BooleanQuery::new(ext_queries))));
        }

        // Apply Drive filter (either from parsed query "d:" or UI filter)
        let effective_drive = parsed.drive.as_ref().or(q.drive.as_ref());
        if let Some(drive) = effective_drive {
            if !drive.is_empty() && drive != "ALL" {
                let term = Term::from_field_text(self.f_drive, drive);
                subqueries.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
            }
        }

        // Apply Category extension filter
        if let Some(cat) = &q.category {
            let extensions = match cat.to_lowercase().as_str() {
                "document" => vec!["pdf", "doc", "docx", "txt", "xlsx", "xls", "pptx", "ppt", "csv", "md"],
                "image" => vec!["png", "jpg", "jpeg", "webp", "gif", "bmp", "svg", "ico"],
                "video" => vec!["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv"],
                "audio" => vec!["mp3", "wav", "flac", "aac", "ogg", "m4a"],
                "app" => vec!["exe", "msi", "bat", "cmd", "ps1", "lnk"],
                "archive" => vec!["zip", "rar", "7z", "tar", "gz", "iso"],
                "code" => vec!["rs", "ts", "tsx", "js", "jsx", "py", "html", "css", "json", "c", "cpp"],
                "folder" => vec![],
                _ => vec![],
            };

            if cat.to_lowercase() == "folder" {
                // Filter is_directory == 1
                let term = Term::from_field_u64(self.f_is_directory, 1);
                subqueries.push((Occur::Must, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
            } else if !extensions.is_empty() {
                let mut ext_queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
                for ext in extensions {
                    let term = Term::from_field_text(self.f_extension, ext);
                    ext_queries.push((Occur::Should, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
                }
                subqueries.push((Occur::Must, Box::new(BooleanQuery::new(ext_queries))));
            }
        }

        let final_query: Box<dyn Query> = if subqueries.is_empty() {
            // Return all or recent
            Box::new(tantivy::query::AllQuery)
        } else {
            Box::new(BooleanQuery::new(subqueries))
        };

        let top_docs = searcher.search(&final_query, &TopDocs::with_limit(limit))?;
        let total_matched = top_docs.len();

        let mut items = Vec::with_capacity(top_docs.len());
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let id_val = retrieved_doc.get_first(self.f_id).and_then(|v| v.as_u64()).map(|v| v as i64);
            let path_val = retrieved_doc.get_first(self.f_path).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let filename_val = retrieved_doc.get_first(self.f_filename).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ext_val = retrieved_doc.get_first(self.f_extension).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let drive_val = retrieved_doc.get_first(self.f_drive).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let size_val = retrieved_doc.get_first(self.f_size_bytes).and_then(|v| v.as_u64()).unwrap_or(0);
            let modified_val = retrieved_doc.get_first(self.f_modified_at).and_then(|v| v.as_i64()).unwrap_or(0);
            let is_dir_val = retrieved_doc.get_first(self.f_is_directory).and_then(|v| v.as_u64()).unwrap_or(0) == 1;

            items.push(FileItem {
                id: id_val,
                path: path_val,
                filename: filename_val,
                extension: ext_val,
                size_bytes: size_val,
                modified_at: modified_val,
                is_directory: is_dir_val,
                drive: drive_val,
            });
        }

        let elapsed = start_time.elapsed().as_secs_f64() * 1000.0;

        Ok(SearchResult {
            items,
            total_matched,
            elapsed_ms: elapsed,
        })
    }
}

fn escape_regex(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len() * 2);
    for c in input.chars() {
        match c {
            '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$' => {
                escaped.push('\\');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    escaped
}

