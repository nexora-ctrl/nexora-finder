import React from "react";
import { CheckCircle2, AlertCircle, Info, X } from "lucide-react";

export interface UpdateStatusNotice {
  type: "success" | "info" | "warning";
  title: string;
  message: string;
}

interface UpdateStatusModalProps {
  notice: UpdateStatusNotice | null;
  onClose: () => void;
}

export const UpdateStatusModal: React.FC<UpdateStatusModalProps> = ({ notice, onClose }) => {
  if (!notice) return null;

  const isSuccess = notice.type === "success";
  const isWarning = notice.type === "warning";

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div
        className="settings-modal"
        onClick={(e) => e.stopPropagation()}
        style={{
          maxWidth: 420,
          padding: "20px 24px",
          background: "var(--bg-secondary)",
          borderRadius: 16,
          boxShadow: "0 20px 50px rgba(0,0,0,0.45)",
          border: "1px solid var(--border-color)",
        }}
      >
        <div style={{ display: "flex", alignItems: "flex-start", gap: 14 }}>
          <div
            style={{
              padding: 10,
              borderRadius: 12,
              background: isSuccess
                ? "rgba(16, 185, 129, 0.12)"
                : isWarning
                ? "rgba(245, 158, 11, 0.12)"
                : "rgba(59, 130, 246, 0.12)",
              color: isSuccess ? "#10b981" : isWarning ? "#f59e0b" : "#3b82f6",
              flexShrink: 0,
            }}
          >
            {isSuccess ? (
              <CheckCircle2 size={22} />
            ) : isWarning ? (
              <AlertCircle size={22} />
            ) : (
              <Info size={22} />
            )}
          </div>

          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <h3
                style={{
                  margin: 0,
                  fontSize: 14,
                  fontWeight: 600,
                  color: "var(--text-primary)",
                  letterSpacing: "-0.01em",
                }}
              >
                {notice.title}
              </h3>
              <button
                type="button"
                onClick={onClose}
                className="settings-close-btn"
                style={{ margin: -4 }}
                aria-label="Tutup"
              >
                <X size={15} />
              </button>
            </div>

            <p
              style={{
                margin: "8px 0 0 0",
                fontSize: 12,
                color: "var(--text-secondary)",
                lineHeight: 1.5,
              }}
            >
              {notice.message}
            </p>

            <div style={{ display: "flex", justifyContent: "flex-end", marginTop: 16 }}>
              <button
                type="button"
                className="settings-reindex-btn"
                onClick={onClose}
                style={{ margin: 0, padding: "6px 16px", fontSize: 12 }}
              >
                Mengerti
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

