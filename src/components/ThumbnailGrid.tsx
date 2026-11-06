import React, { useEffect, useRef } from "react";
import { FileItem } from "../types";
import { ThumbnailCard } from "./ThumbnailCard";
import { Clock, Search, FolderSearch } from "lucide-react";

interface ThumbnailGridProps {
  items: FileItem[];
  selectedIndex: number;
  onSelect: (index: number) => void;
  onOpen: (item: FileItem) => void;
  onContextMenu?: (item: FileItem, e: React.MouseEvent) => void;
  query: string;
  isLoading: boolean;
}

export const ThumbnailGrid: React.FC<ThumbnailGridProps> = ({
  items,
  selectedIndex,
  onSelect,
  onOpen,
  onContextMenu,
  query,
  isLoading,
}) => {
  const gridRef = useRef<HTMLDivElement>(null);

  // Auto-scroll selected card into view
  useEffect(() => {
    if (gridRef.current && selectedIndex >= 0) {
      const activeElement = gridRef.current.children[selectedIndex] as HTMLElement;
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
        <p className="empty-subtitle">Memindai berkas dengan tampilan visual thumbnail</p>
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
          Ketik nama file atau gambar untuk melihat thumbnail secara cepat.
        </p>
      </div>
    );
  }

  return (
    <div className="thumbnail-grid-container" ref={gridRef}>
      {items.map((item, index) => (
        <ThumbnailCard
          key={item.path + index}
          item={item}
          isSelected={index === selectedIndex}
          onSelect={() => onSelect(index)}
          onOpen={() => onOpen(item)}
          onContextMenu={(e) => onContextMenu?.(item, e)}
        />
      ))}
    </div>
  );
};

