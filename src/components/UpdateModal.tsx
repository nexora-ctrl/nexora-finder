import React from "react";
import { Download, Sparkles, X } from "lucide-react";

interface UpdateModalProps {
  isOpen: boolean;
  version: string;
  body?: string;
  onUpdate: () => void;
  onClose: () => void;
  isUpdating: boolean;
}

export const UpdateModal: React.FC<UpdateModalProps> = ({
  isOpen,
  version,
  body,
  onUpdate,
  onClose,
  isUpdating,
}) => {
  if (!isOpen) return null;

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div
        className="settings-modal update-modal"
        onClick={(e) => e.stopPropagation()}
        style={{ maxWidth: 440 }}
      >
        <div className="settings-header">
          <div className="settings-title-group">
            <Sparkles className="settings-icon text-amber-400" size={18} />
            <h2 className="settings-title">Pembaruan Tersedia</h2>
          </div>
          <button
            type="button"
            className="settings-close-btn"
            onClick={onClose}
            aria-label="Tutup"
          >
            <X size={16} />
          </button>
        </div>

        <div className="settings-body" style={{ gap: 16 }}>
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 8,
              padding: "12px 14px",
              background: "rgba(59, 130, 246, 0.08)",
              border: "1px solid rgba(59, 130, 246, 0.2)",
              borderRadius: 10,
            }}
          >
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
              <span style={{ fontWeight: 600, fontSize: 13, color: "var(--text-primary)" }}>
                Nexora Finder v{version}
              </span>
              <span
                style={{
                  fontSize: 11,
                  padding: "2px 8px",
                  borderRadius: 999,
                  background: "rgba(37, 99, 235, 0.2)",
                  color: "#60a5fa",
                  fontWeight: 600,
                }}
              >
                Rilis Baru
              </span>
            </div>
            <p style={{ margin: 0, fontSize: 12, color: "var(--text-secondary)", lineHeight: 1.5 }}>
              {body || "Peningkatan performa pencarian, perbaikan bug, dan kestabilan sistem terbaru telah tersedia di GitHub."}
            </p>
          </div>

          <div style={{ display: "flex", gap: 10, justifyContent: "flex-end", marginTop: 8 }}>
            <button
              type="button"
              className="px-3 py-1.5 rounded-lg text-xs font-medium cursor-pointer"
              style={{
                background: "transparent",
                border: "1px solid var(--border-color)",
                color: "var(--text-secondary)",
              }}
              onClick={onClose}
              disabled={isUpdating}
            >
              Nanti Saja
            </button>
            <button
              type="button"
              className="settings-reindex-btn"
              style={{ margin: 0, padding: "8px 16px" }}
              onClick={onUpdate}
              disabled={isUpdating}
            >
              <Download size={14} className={isUpdating ? "animate-spin" : ""} />
              <span>{isUpdating ? "Mengunduh & Memperbarui..." : "Perbarui Sekarang"}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

