import React from "react";
import { IndexingStatus } from "../types";
import { Database, RefreshCw, CheckCircle2 } from "lucide-react";

interface StatusBarProps {
  status: IndexingStatus | null;
  onReindex: () => void;
}

export const StatusBar: React.FC<StatusBarProps> = ({ status, onReindex }) => {
  const isIndexing = status?.is_indexing ?? false;
  const totalCount = status?.total_indexed ?? 0;

  return (
    <footer className="app-status-bar">
      <div className="status-left">
        <Database size={13} className="text-cyan-400" />
        <span className="indexed-count-text">
          {totalCount.toLocaleString("id-ID")} berkas terindeks
        </span>
      </div>

      <div className="status-center">
        {isIndexing ? (
          <div className="indexing-indicator">
            <RefreshCw size={12} className="animate-spin text-amber-400" />
            <span className="indexing-text">
              Memindai {status?.current_drive} {status?.current_directory ? `• ${status.current_directory}` : ""}
            </span>
          </div>
        ) : (
          <div className="ready-indicator">
            <CheckCircle2 size={12} className="text-emerald-400" />
            <span className="ready-text">
              Indeks Siap {status?.last_completed ? `• ${status.last_completed}` : ""}
            </span>
          </div>
        )}
      </div>

      <div className="status-right">
        <button
          type="button"
          className="reindex-btn"
          onClick={onReindex}
          disabled={isIndexing}
          title="Mulai pemindaian ulang seluruh drive di latar belakang"
        >
          <RefreshCw size={11} className={isIndexing ? "animate-spin" : ""} />
          <span>Indeks Ulang</span>
        </button>

        <span className="shortcut-hint" title="Buka/Tutup dengan Ctrl+Alt+S atau Alt+Space">
          <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>S</kbd>
        </span>
      </div>
    </footer>
  );
};

