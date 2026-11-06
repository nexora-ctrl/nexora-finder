import React, { useEffect, useRef } from "react";
import { FileItem } from "../types";
import { FolderOpen, ExternalLink, Copy, Play } from "lucide-react";

interface ContextMenuProps {
  x: number;
  y: number;
  item: FileItem;
  onClose: () => void;
  onOpen: (item: FileItem) => void;
  onOpenWith: (item: FileItem) => void;
  onReveal: (item: FileItem) => void;
  onCopyPath: (item: FileItem) => void;
}

export const ContextMenu: React.FC<ContextMenuProps> = ({
  x,
  y,
  item,
  onClose,
  onOpen,
  onOpenWith,
  onReveal,
  onCopyPath,
}) => {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("mousedown", handleClickOutside);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("mousedown", handleClickOutside);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [onClose]);

  // Adjust coordinates if menu overflows window boundary (800x580)
  const menuWidth = 200;
  const menuHeight = 160;
  const adjustedX = Math.min(x, window.innerWidth - menuWidth - 10);
  const adjustedY = Math.min(y, window.innerHeight - menuHeight - 10);

  return (
    <div
      ref={menuRef}
      className="custom-context-menu"
      style={{ left: `${Math.max(10, adjustedX)}px`, top: `${Math.max(10, adjustedY)}px` }}
      onClick={(e) => e.stopPropagation()}
      onContextMenu={(e) => e.preventDefault()}
    >
      <div
        className="context-menu-item"
        onClick={() => {
          onClose();
          onOpen(item);
        }}
      >
        <Play size={14} className="menu-icon text-cyan-400" />
        <span>Buka</span>
      </div>

      {!item.is_directory && (
        <div
          className="context-menu-item"
          onClick={() => {
            onClose();
            onOpenWith(item);
          }}
        >
          <ExternalLink size={14} className="menu-icon text-blue-400" />
          <span>Buka dengan... (Open with)</span>
        </div>
      )}

      <div
        className="context-menu-item"
        onClick={() => {
          onClose();
          onReveal(item);
        }}
      >
        <FolderOpen size={14} className="menu-icon text-amber-400" />
        <span>Buka lokasi berkas (Open file location)</span>
      </div>

      <div className="context-menu-separator" />

      <div
        className="context-menu-item"
        onClick={() => {
          onClose();
          onCopyPath(item);
        }}
      >
        <Copy size={14} className="menu-icon text-slate-400" />
        <span>Salin jalur lengkap</span>
      </div>
    </div>
  );
};

