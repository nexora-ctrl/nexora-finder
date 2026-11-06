import React from "react";
import { CategoryType, DriveInfo } from "../types";
import { HardDrive, FolderSearch, X } from "lucide-react";

interface CategoryFilterProps {
  activeCategory: CategoryType;
  onSelectCategory: (cat: CategoryType) => void;
  drives: DriveInfo[];
  activeDrive: string;
  onSelectDrive: (drive: string) => void;
  folderScope?: string | null;
  onClearFolderScope?: () => void;
}

const CATEGORIES: { id: CategoryType; label: string }[] = [
  { id: "all", label: "Semua" },
  { id: "folder", label: "Folder" },
  { id: "document", label: "Dokumen" },
  { id: "image", label: "Gambar" },
  { id: "video", label: "Video" },
  { id: "audio", label: "Audio" },
  { id: "app", label: "Aplikasi" },
  { id: "archive", label: "Arsip" },
  { id: "code", label: "Kode" },
];

export const CategoryFilter: React.FC<CategoryFilterProps> = ({
  activeCategory,
  onSelectCategory,
  drives,
  activeDrive,
  onSelectDrive,
  folderScope,
  onClearFolderScope,
}) => {
  return (
    <div className="filter-bar">
      {folderScope ? (
        <div style={{ display: "flex", alignItems: "center", gap: 6, flex: 1, minWidth: 0, paddingRight: 8 }}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 6,
              background: "rgba(59, 130, 246, 0.15)",
              border: "1px solid rgba(59, 130, 246, 0.4)",
              color: "#60a5fa",
              borderRadius: 8,
              padding: "3px 8px 3px 10px",
              fontSize: 11.5,
              fontWeight: 500,
              maxWidth: "100%",
            }}
          >
            <FolderSearch size={14} style={{ flexShrink: 0 }} />
            <span
              style={{
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
                maxWidth: 400,
              }}
              title={folderScope}
            >
              Terkunci: {folderScope}
            </span>
            {onClearFolderScope && (
              <button
                type="button"
                onClick={onClearFolderScope}
                style={{
                  background: "transparent",
                  border: "none",
                  cursor: "pointer",
                  color: "#93c5fd",
                  padding: 2,
                  display: "flex",
                  alignItems: "center",
                  borderRadius: 4,
                }}
                title="Buka kunci folder (Cari ke seluruh sistem)"
              >
                <X size={13} />
              </button>
            )}
          </div>
        </div>
      ) : (
        <div className="category-chips">
          {CATEGORIES.map((cat) => (
            <button
              key={cat.id}
              type="button"
              className={`filter-chip ${activeCategory === cat.id ? "active" : ""}`}
              onClick={() => onSelectCategory(cat.id)}
            >
              {cat.label}
            </button>
          ))}
        </div>
      )}

      <div className="drive-chips">
        <button
          type="button"
          className={`drive-chip ${activeDrive === "ALL" ? "active" : ""}`}
          onClick={() => onSelectDrive("ALL")}
          title="Semua Drive"
        >
          <HardDrive size={13} />
          <span>Semua</span>
        </button>

        {drives.map((d) => (
          <button
            key={d.letter}
            type="button"
            className={`drive-chip ${activeDrive === d.letter ? "active" : ""}`}
            onClick={() => onSelectDrive(d.letter)}
            title={`${d.label} (${d.drive_type})`}
          >
            {d.letter}
          </button>
        ))}
      </div>
    </div>
  );
};


