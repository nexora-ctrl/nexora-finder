import React, { useState, useEffect } from "react";
import { X, Power, Minimize2, Keyboard, RefreshCw, Check, SlidersHorizontal, ShieldCheck, HardDrive, Sun, Moon, Sparkles, Droplets } from "lucide-react";
import { isEnabled, enable, disable } from "@tauri-apps/plugin-autostart";
import { invoke } from "@tauri-apps/api/core";
import { DriveInfo } from "../types";
import { formatBytes } from "../utils/format";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onReindex: () => void;
  totalIndexed: number;
  isIndexing: boolean;
  availableDrives: DriveInfo[];
  theme: "dark" | "light";
  onThemeChange: (theme: "dark" | "light") => void;
  bgOpacity: number;
  onBgOpacityChange: (opacity: number) => void;
  onCheckUpdate: () => void;
  isCheckingUpdate: boolean;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  onReindex,
  totalIndexed,
  isIndexing,
  availableDrives,
  theme,
  onThemeChange,
  bgOpacity,
  onBgOpacityChange,
  onCheckUpdate,
  isCheckingUpdate,
}) => {
  const [autostart, setAutostart] = useState(false);
  const [minimizeToTray, setMinimizeToTray] = useState(true);
  const [startMinimized, setStartMinimized] = useState(true);
  const [driveScope, setDriveScope] = useState<"all" | "custom">("all");
  const [selectedDrives, setSelectedDrives] = useState<string[]>([]);
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

    // Load drive scope settings
    invoke<string[]>("get_indexed_drives")
      .then((drives) => {
        if (drives.length === 1 && drives[0] === "ALL") {
          setDriveScope("all");
          setSelectedDrives(availableDrives.map((d) => d.letter));
        } else {
          setDriveScope("custom");
          setSelectedDrives(drives);
        }
      })
      .catch(() => {
        setDriveScope("all");
        setSelectedDrives(availableDrives.map((d) => d.letter));
      });
  }, [isOpen, availableDrives]);

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

  const handleSelectDriveScope = async (scope: "all" | "custom") => {
    setDriveScope(scope);
    try {
      if (scope === "all") {
        await invoke("set_indexed_drives", { drives: ["ALL"] });
      } else {
        const toSave = selectedDrives.length > 0 ? selectedDrives : availableDrives.map((d) => d.letter);
        setSelectedDrives(toSave);
        await invoke("set_indexed_drives", { drives: toSave });
      }
      showSavedToast();
    } catch (err) {
      console.error("Failed to update drive scope:", err);
    }
  };

  const handleToggleDriveChoice = async (letter: string) => {
    let nextList: string[];
    if (selectedDrives.includes(letter)) {
      if (selectedDrives.length === 1) {
        // Prevent deselecting all drives
        return;
      }
      nextList = selectedDrives.filter((d) => d !== letter);
    } else {
      nextList = [...selectedDrives, letter];
    }
    setSelectedDrives(nextList);
    try {
      await invoke("set_indexed_drives", { drives: nextList });
      showSavedToast();
    } catch (err) {
      console.error("Failed to save selected drives:", err);
    }
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
            <h2 className="settings-title">Pengaturan Nexora Finder</h2>
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
          {/* Setting 0: Theme Mode */}
          <div className="setting-card-section">
            <div className="setting-card-header">
              <div className="setting-icon-box">
                {theme === "light" ? (
                  <Sun size={18} className="text-amber-400" />
                ) : (
                  <Moon size={18} className="text-cyan-400" />
                )}
              </div>
              <div className="setting-text">
                <span className="setting-label">Tema Tampilan</span>
                <p className="setting-desc">
                  Pilih nuansa antarmuka antara Mode Gelap (Dark Liquid Glass) atau Mode Terang (Light Liquid Glass).
                </p>
              </div>
            </div>

            <div className="theme-toggle-options">
              <button
                type="button"
                className={`theme-pill ${theme === "dark" ? "active" : ""}`}
                onClick={() => {
                  onThemeChange("dark");
                  showSavedToast();
                }}
              >
                <Moon size={13} />
                <span>Mode Gelap</span>
              </button>
              <button
                type="button"
                className={`theme-pill ${theme === "light" ? "active" : ""}`}
                onClick={() => {
                  onThemeChange("light");
                  showSavedToast();
                }}
              >
                <Sun size={13} />
                <span>Mode Terang</span>
              </button>
            </div>
          </div>

          {/* Setting: Background Transparency */}
          <div className="setting-card-section">
            <div className="setting-card-header">
              <div className="setting-icon-box">
                <Droplets size={18} className="text-cyan-400" />
              </div>
              <div className="setting-text">
                <span className="setting-label">Transparansi Latar Belakang</span>
                <p className="setting-desc">
                  Sesuaikan tingkat kepekatan efek kaca buram (frosted glass) antarmuka jendela.
                </p>
              </div>
            </div>

            <div className="opacity-slider-container">
              <div className="opacity-slider-header">
                <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>Tingkat Opasitas</span>
                <span className="opacity-value-badge">{bgOpacity}%</span>
              </div>

              <input
                type="range"
                min="40"
                max="100"
                step="2"
                value={bgOpacity}
                onChange={(e) => onBgOpacityChange(Number(e.target.value))}
                className="opacity-slider-input"
              />

              <div className="opacity-preset-group">
                <button
                  type="button"
                  className={`opacity-preset-btn ${bgOpacity === 55 ? "active" : ""}`}
                  onClick={() => {
                    onBgOpacityChange(55);
                    showSavedToast();
                  }}
                >
                  Kaca (55%)
                </button>
                <button
                  type="button"
                  className={`opacity-preset-btn ${bgOpacity === 88 ? "active" : ""}`}
                  onClick={() => {
                    onBgOpacityChange(88);
                    showSavedToast();
                  }}
                >
                  Seimbang (88%)
                </button>
                <button
                  type="button"
                  className={`opacity-preset-btn ${bgOpacity === 98 ? "active" : ""}`}
                  onClick={() => {
                    onBgOpacityChange(98);
                    showSavedToast();
                  }}
                >
                  Solid (98%)
                </button>
              </div>
            </div>
          </div>

          {/* Setting: Drive Scanning Scope */}
          <div className="setting-card-section">
            <div className="setting-card-header">
              <div className="setting-icon-box">
                <HardDrive size={18} className="text-cyan-400" />
              </div>
              <div className="setting-text">
                <span className="setting-label">Cakupan Drive Pemindaian</span>
                <p className="setting-desc">
                  Atur apakah pencarian membaca seluruh drive atau hanya drive spesifik yang dipilih.
                </p>
              </div>
            </div>

            <div className="drive-scope-options">
              <button
                type="button"
                className={`scope-pill ${driveScope === "all" ? "active" : ""}`}
                onClick={() => handleSelectDriveScope("all")}
              >
                <span>Semua Drive (Default)</span>
              </button>
              <button
                type="button"
                className={`scope-pill ${driveScope === "custom" ? "active" : ""}`}
                onClick={() => handleSelectDriveScope("custom")}
              >
                <span>Pilih Drive Spesifik</span>
              </button>
            </div>

            {driveScope === "custom" && (
              <div className="custom-drives-list">
                {availableDrives.map((d) => {
                  const isChecked = selectedDrives.includes(d.letter);
                  return (
                    <div
                      key={d.letter}
                      className={`custom-drive-item ${isChecked ? "selected" : ""}`}
                      onClick={() => handleToggleDriveChoice(d.letter)}
                    >
                      <div className="custom-drive-left">
                        <span className={`drive-check-box ${isChecked ? "checked" : ""}`}>
                          {isChecked && <Check size={11} strokeWidth={3} />}
                        </span>
                        <span className="custom-drive-letter">{d.letter}</span>
                        <span className="custom-drive-label">
                          {d.label ? `(${d.label})` : d.drive_type}
                        </span>
                      </div>
                      <span className="custom-drive-size">
                        {formatBytes(d.total_space_bytes)}
                      </span>
                    </div>
                  );
                })}
              </div>
            )}
          </div>

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
              <span className="index-summary-label">
                {driveScope === "all"
                  ? "Semua Drive Terindeks"
                  : `Drive (${selectedDrives.join(", ")}) Terindeks`}
              </span>
            </div>
            <button
              type="button"
              className="settings-reindex-btn"
              onClick={onReindex}
              disabled={isIndexing}
            >
              <RefreshCw size={13} className={isIndexing ? "animate-spin" : ""} />
              <span>
                {isIndexing
                  ? "Memindai..."
                  : driveScope === "all"
                  ? "Indeks Ulang Semua Drive"
                  : "Indeks Ulang Drive Terpilih"}
              </span>
            </button>
          </div>
        </div>

        <div className="settings-footer" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <span className="version-tag">Nexora Finder • Beta v0.4.0</span>
            <button
              type="button"
              onClick={onCheckUpdate}
              disabled={isCheckingUpdate}
              style={{
                background: "rgba(59, 130, 246, 0.12)",
                border: "1px solid rgba(59, 130, 246, 0.3)",
                color: "var(--accent-primary, #3b82f6)",
                padding: "3px 8px",
                borderRadius: 6,
                fontSize: 11,
                fontWeight: 600,
                display: "inline-flex",
                alignItems: "center",
                gap: 5,
                cursor: isCheckingUpdate ? "wait" : "pointer",
                transition: "all 0.15s ease",
              }}
              title="Periksa apakah ada versi baru di GitHub"
            >
              <Sparkles size={11} className={isCheckingUpdate ? "animate-spin" : ""} />
              <span>{isCheckingUpdate ? "Memeriksa..." : "Periksa Pembaruan"}</span>
            </button>
          </div>
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

