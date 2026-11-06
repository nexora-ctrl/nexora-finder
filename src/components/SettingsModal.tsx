import React, { useState, useEffect } from "react";
import { X, Power, Minimize2, Keyboard, RefreshCw, Check, SlidersHorizontal, ShieldCheck } from "lucide-react";
import { isEnabled, enable, disable } from "@tauri-apps/plugin-autostart";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onReindex: () => void;
  totalIndexed: number;
  isIndexing: boolean;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  onReindex,
  totalIndexed,
  isIndexing,
}) => {
  const [autostart, setAutostart] = useState(false);
  const [minimizeToTray, setMinimizeToTray] = useState(true);
  const [startMinimized, setStartMinimized] = useState(true);
  const [savedNotice, setSavedNotice] = useState(false);

  useEffect(() => {
    // Load autostart status
    isEnabled()
      .then((enabled) => setAutostart(enabled))
      .catch((err) => console.warn("Autostart check failed:", err));

    // Load local preferences
    const storedMinToTray = localStorage.getItem("nexora_minimize_to_tray");
    if (storedMinToTray !== null) {
      setMinimizeToTray(storedMinToTray === "true");
    }

    const storedStartMin = localStorage.getItem("nexora_start_minimized");
    if (storedStartMin !== null) {
      setStartMinimized(storedStartMin === "true");
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const handleToggleAutostart = async () => {
    try {
      if (autostart) {
        await disable();
        setAutostart(false);
      } else {
        await enable();
        setAutostart(true);
      }
      showSavedToast();
    } catch (err) {
      console.error("Failed to toggle autostart:", err);
    }
  };

  const handleToggleMinToTray = () => {
    const nextVal = !minimizeToTray;
    setMinimizeToTray(nextVal);
    localStorage.setItem("nexora_minimize_to_tray", String(nextVal));
    showSavedToast();
  };

  const handleToggleStartMinimized = () => {
    const nextVal = !startMinimized;
    setStartMinimized(nextVal);
    localStorage.setItem("nexora_start_minimized", String(nextVal));
    showSavedToast();
  };

  const showSavedToast = () => {
    setSavedNotice(true);
    setTimeout(() => setSavedNotice(false), 1800);
  };

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-modal" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <div className="settings-title-group">
            <SlidersHorizontal size={18} className="text-cyan-400" />
            <h2 className="settings-title">Pengaturan Nexora Search</h2>
          </div>
          <button
            type="button"
            className="settings-close-btn"
            onClick={onClose}
            title="Tutup (Esc)"
          >
            <X size={16} />
          </button>
        </div>

        <div className="settings-body">
          {/* Setting 1: Windows Startup */}
          <div className="setting-card">
            <div className="setting-icon-box">
              <Power size={18} className="text-cyan-400" />
            </div>
            <div className="setting-text">
              <span className="setting-label">Jalankan saat Windows Startup</span>
              <p className="setting-desc">
                Otomatis aktif di latar belakang saat Windows 11 dinyalakan.
              </p>
            </div>
            <button
              type="button"
              className={`toggle-switch ${autostart ? "active" : ""}`}
              onClick={handleToggleAutostart}
              aria-label="Toggle Windows Startup"
            >
              <span className="toggle-thumb" />
            </button>
          </div>

          {/* Setting 2: Run directly to Tray */}
          <div className="setting-card">
            <div className="setting-icon-box">
              <Minimize2 size={18} className="text-cyan-400" />
            </div>
            <div className="setting-text">
              <span className="setting-label">Awal Run Langsung ke Tray</span>
              <p className="setting-desc">
                Saat aplikasi dibuka, langsung standby di system tray tanpa memunculkan window.
              </p>
            </div>
            <button
              type="button"
              className={`toggle-switch ${startMinimized ? "active" : ""}`}
              onClick={handleToggleStartMinimized}
              aria-label="Toggle Start Minimized"
            >
              <span className="toggle-thumb" />
            </button>
          </div>

          {/* Setting 3: Minimize to Tray on Close */}
          <div className="setting-card">
            <div className="setting-icon-box">
              <ShieldCheck size={18} className="text-cyan-400" />
            </div>
            <div className="setting-text">
              <span className="setting-label">Minimize ke System Tray</span>
              <p className="setting-desc">
                Menutup jendela (Esc) hanya menyembunyikan ke tray agar pencarian tetap instan.
              </p>
            </div>
            <button
              type="button"
              className={`toggle-switch ${minimizeToTray ? "active" : ""}`}
              onClick={handleToggleMinToTray}
              aria-label="Toggle Minimize to Tray"
            >
              <span className="toggle-thumb" />
            </button>
          </div>

          {/* Setting 4: Shortcut Guide */}
          <div className="setting-card-info">
            <div className="setting-icon-box">
              <Keyboard size={18} className="text-amber-400" />
            </div>
            <div className="setting-text">
              <span className="setting-label">Shortcut Global Cepat</span>
              <div className="shortcut-pills">
                <div className="shortcut-pill-item">
                  <span>Utama:</span>
                  <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>S</kbd>
                </div>
                <div className="shortcut-pill-item">
                  <span>Alternatif:</span>
                  <kbd>Alt</kbd> + <kbd>Space</kbd>
                </div>
              </div>
            </div>
          </div>

          {/* Setting 5: Index Status & Reindex */}
          <div className="setting-card-action">
            <div className="index-summary">
              <span className="index-summary-count">
                {totalIndexed.toLocaleString("id-ID")}
              </span>
              <span className="index-summary-label">Berkas & Folder Terindeks</span>
            </div>
            <button
              type="button"
              className="settings-reindex-btn"
              onClick={onReindex}
              disabled={isIndexing}
            >
              <RefreshCw size={13} className={isIndexing ? "animate-spin" : ""} />
              <span>{isIndexing ? "Memindai..." : "Indeks Ulang Semua Drive"}</span>
            </button>
          </div>
        </div>

        <div className="settings-footer">
          <span className="version-tag">Nexora Search • Standalone Edition v1.0.0</span>
          {savedNotice && (
            <span className="saved-notice">
              <Check size={12} className="text-emerald-400" /> Tersimpan
            </span>
          )}
        </div>
      </div>
    </div>
  );
};

