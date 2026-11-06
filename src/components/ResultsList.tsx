import React, { useEffect, useRef } from "react";
import { FileItem } from "../types";
import { FileIcon } from "./FileIcon";
import { formatBytes, formatDate } from "../utils/format";
import { Clock, Search, FolderSearch } from "lucide-react";

interface ResultsListProps {
  items: FileItem[];
  selectedIndex: number;
  onSelect: (index: number) => void;
  onOpen: (item: FileItem) => void;
  onContextMenu?: (item: FileItem, e: React.MouseEvent) => void;
  query: string;
  isLoading: boolean;
}

export const ResultsList: React.FC<ResultsListProps> = ({
  items,
  selectedIndex,
  onSelect,
  onOpen,
  onContextMenu,
  query,
  isLoading,
}) => {
  const listRef = useRef<HTMLDivElement>(null);

  // Auto-scroll selected item into view
  useEffect(() => {
    if (listRef.current && selectedIndex >= 0) {
      const activeElement = listRef.current.children[selectedIndex] as HTMLElement;
      if (activeElement) {
        activeElement.scrollIntoView({
          block: "nearest",
          behavior: "smooth",
        });
      }
    }
  }, [selectedIndex]);

  if (isLoading && items.length === 0) {
    return (
      <div className="results-empty-state">
        <div className="empty-icon-pulse">
          <Search size={32} className="text-cyan-400 animate-pulse" />
        </div>
        <p className="empty-title">Mencari di seluruh drive...</p>
        <p className="empty-subtitle">Memindai indeks Tantivy berkecepatan tinggi</p>
      </div>
    );
  }

  if (items.length === 0 && query.trim().length > 0) {
    return (
      <div className="results-empty-state">
        <FolderSearch size={36} className="text-slate-500 mb-2" />
        <p className="empty-title">Tidak ada berkas ditemukan</p>
        <p className="empty-subtitle">
          Tidak ada berkas atau folder yang cocok dengan "{query}". Coba kata kunci lain atau periksa filter.
        </p>
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="results-empty-state">
        <Clock size={36} className="text-slate-600 mb-2" />
        <p className="empty-title">Nexora Finder Spotlight Edition</p>
        <p className="empty-subtitle">
          Ketik nama file, ekstensi, atau kata kunci untuk mencari secara instan.
        </p>
      </div>
    );
  }

  return (
    <div className="results-scroll-container" ref={listRef}>
      {items.map((item, index) => {
        const isSelected = index === selectedIndex;
        return (
          <div
            key={item.path + index}
            className={`result-row ${isSelected ? "selected" : ""}`}
            onClick={() => onSelect(index)}
            onDoubleClick={() => onOpen(item)}
            onContextMenu={(e) => {
              e.preventDefault();
              onSelect(index);
              onContextMenu?.(item, e);
            }}
          >
            <div className="result-icon-cell">
              <FileIcon
                extension={item.extension}
                isDirectory={item.is_directory}
                size={16}
              />
            </div>

            <div className="result-details-cell">
              <div className="result-title-row">
                <span className="result-filename">{item.filename}</span>
                {item.extension && !item.is_directory && (
                  <span className="result-ext-badge">{item.extension.toUpperCase()}</span>
                )}
              </div>
              <div className="result-path-row" title={item.path}>
                {item.path}
              </div>
            </div>

            <div className="result-meta-cell">
              <span className="result-size">
                {item.is_directory ? "Folder" : formatBytes(item.size_bytes)}
              </span>
              <span className="result-date">{formatDate(item.modified_at)}</span>
            </div>
          </div>
        );
      })}
    </div>
  );
};

