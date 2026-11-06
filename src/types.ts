export interface FileItem {
  id: number | null;
  path: string;
  filename: string;
  extension: string;
  size_bytes: number;
  modified_at: number;
  is_directory: boolean;
  drive: string;
}

export interface SearchQuery {
  query: string;
  category?: string | null;
  drive?: string | null;
  min_size?: number | null;
  max_size?: number | null;
  limit?: number | null;
}

export interface SearchResult {
  items: FileItem[];
  total_matched: number;
  elapsed_ms: number;
}

export interface IndexingStatus {
  is_indexing: boolean;
  total_indexed: number;
  current_drive: string;
  current_directory: string;
  progress_percent: number;
  last_completed: string | null;
}

export interface DriveInfo {
  letter: string;
  label: string;
  drive_type: string;
  total_space_bytes: number;
  free_space_bytes: number;
  is_ntfs: boolean;
}

export interface IndexStats {
  total_files: number;
  total_directories: number;
  total_size_bytes: number;
  count_by_category: Record<string, number>;
  drives: DriveInfo[];
}

export type CategoryType =
  | "all"
  | "folder"
  | "document"
  | "image"
  | "video"
  | "audio"
  | "app"
  | "archive"
  | "code";

