import React, { useState } from "react";
import { FileItem } from "../types";
import { ExternalLink, FolderOpen, Copy, Check } from "lucide-react";
import { formatBytes, formatDate } from "../utils/format";

interface PreviewPaneProps {
  item: FileItem | null;
  onOpen: (item: FileItem) => void;
  onReveal: (item: FileItem) => void;
  onCopyPath: (item: FileItem) => void;
}

export const PreviewPane: React.FC<PreviewPaneProps> = ({
  item,
  onOpen,
  onReveal,
  onCopyPath,
}) => {
  const [copied, setCopied] = useState(false);

  if (!item) return null;

  const handleCopy = () => {
    onCopyPath(item);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="preview-pane">
      <div className="preview-info">
        <span className="preview-path" title={item.path}>
          {item.path}
        </span>
        <div className="preview-meta">
          <span>{item.is_directory ? "Folder" : formatBytes(item.size_bytes)}</span>
          <span className="meta-separator">•</span>
          <span>Diubah: {formatDate(item.modified_at)}</span>
        </div>
      </div>

      <div className="preview-actions">
        <button
          type="button"
          className="action-btn primary"
          onClick={() => onOpen(item)}
          title="Buka Berkas (Enter)"
        >
          <ExternalLink size={14} />
          <span>Buka</span>
          <kbd>↵</kbd>
        </button>

        <button
          type="button"
          className="action-btn secondary"
          onClick={() => onReveal(item)}
          title="Buka Lokasi di Windows Explorer (Shift+Enter)"
        >
          <FolderOpen size={14} />
          <span>Lokasi</span>
          <kbd>⇧↵</kbd>
        </button>

        <button
          type="button"
          className="action-btn secondary"
          onClick={handleCopy}
          title="Salin Path Lengkap (Ctrl+C)"
        >
          {copied ? <Check size={14} className="text-emerald-400" /> : <Copy size={14} />}
          <span>{copied ? "Tersalin!" : "Salin"}</span>
          <kbd>Ctrl+C</kbd>
        </button>
      </div>
    </div>
  );
};

