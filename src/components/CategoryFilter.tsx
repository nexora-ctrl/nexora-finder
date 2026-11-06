import React from "react";
import { CategoryType, DriveInfo } from "../types";
import { HardDrive } from "lucide-react";

interface CategoryFilterProps {
  activeCategory: CategoryType;
  onSelectCategory: (cat: CategoryType) => void;
  drives: DriveInfo[];
  activeDrive: string;
  onSelectDrive: (drive: string) => void;
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
}) => {
  return (
    <div className="filter-bar">
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

