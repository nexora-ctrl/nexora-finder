use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub id: Option<i64>,
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size_bytes: u64,
    pub modified_at: i64,
    pub is_directory: bool,
    pub drive: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub category: Option<String>,
    pub drive: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub items: Vec<FileItem>,
    pub total_matched: usize,
    pub elapsed_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStatus {
    pub is_indexing: bool,
    pub total_indexed: u64,
    pub current_drive: String,
    pub current_directory: String,
    pub progress_percent: f32,
    pub last_completed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub letter: String,
    pub label: String,
    pub drive_type: String,
    pub total_space_bytes: u64,
    pub free_space_bytes: u64,
    pub is_ntfs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_files: u64,
    pub total_directories: u64,
    pub total_size_bytes: u64,
    pub count_by_category: std::collections::HashMap<String, u64>,
    pub drives: Vec<DriveInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedQuery {
    pub keywords: Vec<String>,
    pub extension: Option<String>,
    pub drive: Option<String>,
}

pub fn parse_search_query_text(input: &str) -> ParsedQuery {
    let mut parsed = ParsedQuery::default();
    let tokens = input.split_whitespace();

    for token in tokens {
        let token_lower = token.to_lowercase();
        if let Some(ext) = token_lower.strip_prefix("ext:") {
            let clean = ext.trim_start_matches('.');
            if !clean.is_empty() {
                parsed.extension = Some(clean.to_string());
            }
        } else if token_lower.starts_with('.') && token_lower.len() > 1 && !token_lower.contains('/') && !token_lower.contains('\\') {
            parsed.extension = Some(token_lower.trim_start_matches('.').to_string());
        } else if token_lower.len() == 2 && token_lower.ends_with(':') && token_lower.chars().next().unwrap().is_ascii_alphabetic() {
            parsed.drive = Some(token.to_uppercase());
        } else if token_lower.contains('.') && !token_lower.starts_with('.') && !token_lower.ends_with('.') && !token_lower.contains('/') && !token_lower.contains('\\') {
            let parts: Vec<&str> = token_lower.split('.').collect();
            if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                parsed.keywords.push(parts[0].to_string());
                if parsed.extension.is_none() {
                    parsed.extension = Some(parts[1].to_string());
                }
            } else {
                for p in parts {
                    if !p.is_empty() {
                        parsed.keywords.push(p.to_string());
                    }
                }
            }
        } else {
            parsed.keywords.push(token.to_string());
        }
    }

    parsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_query() {
        let p1 = parse_search_query_text("Surat .doc");
        assert_eq!(p1.keywords, vec!["Surat"]);
        assert_eq!(p1.extension, Some("doc".to_string()));

        let p2 = parse_search_query_text("Surat Perjanjian .docx");
        assert_eq!(p2.keywords, vec!["Surat", "Perjanjian"]);
        assert_eq!(p2.extension, Some("docx".to_string()));

        let p3 = parse_search_query_text(".pdf");
        assert!(p3.keywords.is_empty());
        assert_eq!(p3.extension, Some("pdf".to_string()));

        let p4 = parse_search_query_text("D: skripsi .pdf");
        assert_eq!(p4.drive, Some("D:".to_string()));
        assert_eq!(p4.keywords, vec!["skripsi"]);
        assert_eq!(p4.extension, Some("pdf".to_string()));
    }
}

