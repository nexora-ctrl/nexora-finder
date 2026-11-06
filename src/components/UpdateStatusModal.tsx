import React from "react";

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

  return (
    <div className="settings-overlay update-status-overlay" onClick={onClose} style={{ zIndex: 100 }}>
      <div
        className="settings-modal update-status-dialog"
        onClick={(e) => e.stopPropagation()}
        style={{
          maxWidth: 380,
          padding: "20px 24px",
          borderRadius: 16,
        }}
      >
        <div>
          <h3
            style={{
              margin: 0,
              fontSize: 14,
              fontWeight: 600,
              color: "var(--text-primary)",
              letterSpacing: "-0.01em",
              lineHeight: 1.4,
            }}
          >
            {notice.title}
          </h3>

          <p
            style={{
              margin: "8px 0 0 0",
              fontSize: 12.5,
              color: "var(--text-secondary)",
              lineHeight: 1.55,
            }}
          >
            {notice.message}
          </p>

          <div style={{ display: "flex", justifyContent: "flex-end", marginTop: 16 }}>
            <button
              type="button"
              className="settings-reindex-btn"
              onClick={onClose}
              style={{
                margin: 0,
                padding: "6px 20px",
                fontSize: 12.5,
                fontWeight: 600,
                minWidth: 70,
              }}
            >
              OK
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
