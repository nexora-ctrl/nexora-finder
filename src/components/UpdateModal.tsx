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
    <div className="settings-overlay" onClick={onClose}>
      <div
        className="settings-modal update-modal"
        onClick={(e) => e.stopPropagation()}
        style={{
          maxWidth: 460,
          padding: "22px 26px",
          background: "var(--bg-secondary)",
          borderRadius: 18,
          boxShadow: "0 24px 60px rgba(0,0,0,0.5)",
          border: "1px solid var(--border-color)",
        }}
      >
        <div className="settings-header" style={{ marginBottom: 18, paddingBottom: 14 }}>
          <div className="settings-title-group" style={{ gap: 10 }}>
            <div
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
              <span style={{ fontSize: 11, color: "var(--text-muted)" }}>
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
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 10,
              padding: "14px 16px",
              background: "rgba(59, 130, 246, 0.06)",
              border: "1px solid rgba(59, 130, 246, 0.18)",
              borderRadius: 12,
            }}
          >
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <span style={{ fontWeight: 600, fontSize: 14, color: "var(--text-primary)" }}>
                  Nexora Finder v{version}
                </span>
                <span
                  style={{
                    fontSize: 10,
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
              <span style={{ fontSize: 11, color: "var(--text-muted)", display: "flex", alignItems: "center", gap: 4 }}>
                <Sparkles size={11} className="text-amber-400" /> Resmi
              </span>
            </div>
            <p
              style={{
                margin: 0,
                fontSize: 12,
                color: "var(--text-secondary)",
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
              className="px-3.5 py-1.5 rounded-lg text-xs font-medium cursor-pointer"
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
