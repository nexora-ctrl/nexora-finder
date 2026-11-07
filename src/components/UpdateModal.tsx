import React from "react";
import { Download, Sparkles, X, ArrowUpCircle } from "lucide-react";

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
    <div className="settings-overlay update-modal-overlay" onClick={onClose} style={{ zIndex: 100 }}>
      <div
        className="settings-modal update-modal"
        onClick={(e) => e.stopPropagation()}
        style={{
          maxWidth: 460,
          padding: "22px 26px",
          borderRadius: 18,
        }}
      >
        <div className="settings-header" style={{ marginBottom: 18, paddingBottom: 14 }}>
          <div className="settings-title-group" style={{ gap: 10 }}>
            <div
              className="update-icon-box"
              style={{
                width: 32,
                height: 32,
                borderRadius: 10,
                background: "linear-gradient(135deg, rgba(59, 130, 246, 0.2), rgba(147, 51, 234, 0.2))",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                color: "#60a5fa",
              }}
            >
              <ArrowUpCircle size={18} />
            </div>
            <div>
              <h2 className="settings-title" style={{ fontSize: 15, margin: 0 }}>
                Pembaruan Versi Tersedia
              </h2>
              <span className="update-modal-subtitle" style={{ fontSize: 11, color: "var(--text-muted)" }}>
                Versi baru telah dirilis dan siap dipasang
              </span>
            </div>
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
            className="update-release-box"
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 10,
              padding: "14px 16px",
              borderRadius: 12,
            }}
          >
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <span className="update-app-title" style={{ fontWeight: 600, fontSize: 14 }}>
                  Nexora Finder v{version}
                </span>
                <span className="update-badge-pill">
                  Rilis Baru
                </span>
              </div>
              <span className="update-official-badge" style={{ fontSize: 11, display: "flex", alignItems: "center", gap: 4 }}>
                <Sparkles size={11} className="text-amber-400" /> Resmi
              </span>
            </div>
            <p
              className="update-notes-body"
              style={{
                margin: 0,
                fontSize: 12,
                lineHeight: 1.5,
                maxHeight: 120,
                overflowY: "auto",
              }}
            >
              {body || "Peningkatan performa pencarian, perbaikan stabilitas sistem, dan penambahan fitur terbaru."}
            </p>
          </div>

          <div style={{ display: "flex", gap: 10, justifyContent: "flex-end", marginTop: 8 }}>
            <button
              type="button"
              className="update-dismiss-btn"
              onClick={onClose}
              disabled={isUpdating}
            >
              Nanti Saja
            </button>
            <button
              type="button"
              className="settings-reindex-btn update-confirm-btn"
              style={{ margin: 0, padding: "8px 18px", fontSize: 12 }}
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
