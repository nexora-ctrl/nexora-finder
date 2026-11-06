import React, { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  CategoryType,
  DriveInfo,
  FileItem,
  IndexingStatus,
  SearchResult,
} from "./types";
import { SearchBar } from "./components/SearchBar";
import { CategoryFilter } from "./components/CategoryFilter";
import { ResultsList } from "./components/ResultsList";
import { ThumbnailGrid } from "./components/ThumbnailGrid";
import { PreviewPane } from "./components/PreviewPane";
import { StatusBar } from "./components/StatusBar";
import { SettingsModal } from "./components/SettingsModal";
import { ContextMenu } from "./components/ContextMenu";
import { UpdateModal } from "./components/UpdateModal";
import { UpdateStatusModal, UpdateStatusNotice } from "./components/UpdateStatusModal";
import { check } from "@tauri-apps/plugin-updater";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { relaunch } from "@tauri-apps/plugin-process";
import "./App.css";

export function App() {
  const [query, setQuery] = useState("");
  const [activeCategory, setActiveCategory] = useState<CategoryType>("all");
  const [activeDrive, setActiveDrive] = useState("ALL");
  const [folderScope, setFolderScope] = useState<string | null>(null);
  const [results, setResults] = useState<FileItem[]>([]);
  const [totalMatched, setTotalMatched] = useState<number | null>(null);
  const [elapsedMs, setElapsedMs] = useState<number | null>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [indexingStatus, setIndexingStatus] = useState<IndexingStatus | null>(null);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [viewMode, setViewMode] = useState<"list" | "grid">(() => {
    const saved = localStorage.getItem("nexora_view_mode");
    return saved === "grid" ? "grid" : "list";
  });
  const [theme, setTheme] = useState<"dark" | "light">(() => {
    const saved = localStorage.getItem("nexora_theme");
    return saved === "light" ? "light" : "dark";
  });
  const [updateAvailable, setUpdateAvailable] = useState<{
    version: string;
    body?: string;
    updaterObj: any;
  } | null>(null);
  const [updateNotice, setUpdateNotice] = useState<UpdateStatusNotice | null>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const checkedOnOpenRef = useRef(false);

  const checkForUpdates = useCallback(async (manual = false) => {
    try {
      if (manual) setIsCheckingUpdate(true);
      const update = await check();
      if (update?.available) {
        setUpdateAvailable({
          version: update.version,
          body: update.body,
          updaterObj: update,
        });

        // Show native Windows tray notification
        let hasPermission = await isPermissionGranted();
        if (!hasPermission) {
          const permission = await requestPermission();
          hasPermission = permission === "granted";
        }
        if (hasPermission) {
          sendNotification({
            title: "Nexora Finder • Pembaruan Tersedia",
            body: `Versi v${update.version} telah rilis! Klik untuk memperbarui otomatis.`,
          });
        }
      } else if (manual) {
        setUpdateNotice({
          type: "success",
          title: "Versi Terbaru Aktif",
          message: "Nexora Finder sudah menggunakan versi terbaru (v0.3.0-beta). Semua fitur berjalan optimal.",
        });
      }
    } catch (err: any) {
      console.info("Update check info:", err);
      if (manual) {
        const errMsg = String(err?.message || err || "");
        if (errMsg.includes("404") || errMsg.includes("status code: 404") || errMsg.includes("Could not fetch")) {
          setUpdateNotice({
            type: "info",
            title: "Nexora Finder v0.3.0-beta",
            message: "Aplikasi Anda sudah dalam versi terbaru. Belum ada rilis baru yang dipublikasikan di repositori GitHub.",
          });
        } else {
          setUpdateNotice({
            type: "warning",
            title: "Pengecekan Tertunda",
            message: "Tidak dapat memeriksa pembaruan saat ini. Pastikan komputer terhubung ke internet dan coba kembali nanti.",
          });
        }
      }
    } finally {
      if (manual) setIsCheckingUpdate(false);
    }
  }, []);

  const handleToggleViewMode = () => {
    setViewMode((prev) => {
      const next = prev === "list" ? "grid" : "list";
      localStorage.setItem("nexora_view_mode", next);
      return next;
    });
  };

  const isCompact = query.trim().length === 0 && !folderScope && !isSettingsOpen;

  const debounceTimer = useRef<number | null>(null);
  const searchRequestId = useRef(0);

  const handleThemeChange = (newTheme: "dark" | "light") => {
    setTheme(newTheme);
    localStorage.setItem("nexora_theme", newTheme);
  };

  // Sync compact window mode with Tauri backend
  useEffect(() => {
    invoke("set_window_compact", { compact: isCompact }).catch((err) =>
      console.warn("Failed to set window compact mode:", err)
    );
  }, [isCompact]);

  // Initial load: Fetch drives, status, and recent items
  useEffect(() => {
    invoke<DriveInfo[]>("get_drives")
      .then((d) => setDrives(d))
      .catch((err) => console.error("Failed to load drives:", err));

    invoke<IndexingStatus>("get_indexing_status")
      .then((s) => setIndexingStatus(s))
      .catch((err) => console.error("Failed to load status:", err));

    invoke<FileItem[]>("get_recent_files", { limit: 25 })
      .then((recent) => {
        setResults(recent);
        setTotalMatched(recent.length);
        setElapsedMs(0.5);
      })
      .catch((err) => console.error("Failed to load recent files:", err));

    // Check for updates on startup (after 2 seconds) and every 4 hours
    const updateTimer = setTimeout(() => checkForUpdates(false), 2000);
    const intervalTimer = setInterval(() => checkForUpdates(false), 4 * 60 * 60 * 1000);

    // Listen when window is reopened from tray -> check updates once on first open
    const unlistenShowPromise = listen("window-shown", () => {
      if (!checkedOnOpenRef.current) {
        checkedOnOpenRef.current = true;
        setTimeout(() => checkForUpdates(false), 800);
      }
    });

    // Listen to indexing updates from Rust backend
    const unlistenStatusPromise = listen<IndexingStatus>("indexing-status", (event) => {
      setIndexingStatus(event.payload);
    });

    // Listen to focus-search event from global shortcut / tray
    const unlistenFocusPromise = listen("focus-search", () => {
      const input = document.querySelector(".search-input") as HTMLInputElement;
      if (input) {
        input.focus();
        input.select();
      }
    });

    // Listen to open-folder-filter event from Windows Explorer context menu
    const unlistenFolderFilterPromise = listen<string>("open-folder-filter", (event) => {
      if (event.payload) {
        setFolderScope(event.payload);
        setQuery("");
        const input = document.querySelector(".search-input") as HTMLInputElement;
        if (input) {
          input.focus();
        }
      }
    });

    // Handle in-window global toggle shortcuts
    const handleGlobalWindowKeyDown = (e: KeyboardEvent) => {
      if (
        ((e.ctrlKey || e.metaKey) && e.altKey && (e.key === "s" || e.key === "S")) ||
        (e.altKey && e.code === "Space")
      ) {
        e.preventDefault();
        invoke("hide_window").catch(console.error);
      }
    };
    window.addEventListener("keydown", handleGlobalWindowKeyDown);

    return () => {
      clearTimeout(updateTimer);
      clearInterval(intervalTimer);
      unlistenShowPromise.then((unlisten) => unlisten());
      unlistenStatusPromise.then((unlisten) => unlisten());
      unlistenFocusPromise.then((unlisten) => unlisten());
      unlistenFolderFilterPromise.then((unlisten) => unlisten());
      window.removeEventListener("keydown", handleGlobalWindowKeyDown);
    };
  }, [checkForUpdates]);

  // Search function
  const performSearch = useCallback(
    async (searchTerm: string, cat: CategoryType, drv: string, folder: string | null) => {
      const thisId = ++searchRequestId.current;

      if (searchTerm.trim().length === 0 && !folder) {
        // When query is empty and no folder lock, show recent files
        try {
          const recent = await invoke<FileItem[]>("get_recent_files", { limit: 25 });
          if (thisId !== searchRequestId.current) return;
          setResults(recent);
          setTotalMatched(recent.length);
          setElapsedMs(0.5);
          setSelectedIndex(0);
        } catch (err) {
          console.error("Failed to query recent files:", err);
        }
        setIsLoading(false);
        return;
      }

      setIsLoading(true);
      try {
        const catParam = cat === "all" ? null : cat;
        const driveParam = drv === "ALL" ? null : drv;

        const res = await invoke<SearchResult>("search_files", {
          query: {
            query: searchTerm,
            category: catParam,
            drive: driveParam,
            folder: folder || null,
            limit: 150,
          },
        });

        if (thisId !== searchRequestId.current) return;

        setResults(res.items);
        setTotalMatched(res.total_matched);
        setElapsedMs(res.elapsed_ms);
        setSelectedIndex(0);
      } catch (err) {
        if (thisId === searchRequestId.current) {
          console.error("Search failed:", err);
          setResults([]);
          setTotalMatched(0);
        }
      } finally {
        if (thisId === searchRequestId.current) {
          setIsLoading(false);
        }
      }
    },
    [],
  );

  // Debounced search trigger: 5ms for instant responsiveness
  useEffect(() => {
    if (debounceTimer.current) {
      clearTimeout(debounceTimer.current);
    }

    debounceTimer.current = window.setTimeout(() => {
      performSearch(query, activeCategory, activeDrive, folderScope);
    }, 5);

    return () => {
      if (debounceTimer.current) {
        clearTimeout(debounceTimer.current);
      }
    };
  }, [query, activeCategory, activeDrive, folderScope, performSearch]);

  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    item: FileItem;
  } | null>(null);

  const handleOpenItem = async (item: FileItem) => {
    try {
      await invoke("open_item", { path: item.path });
      // Auto minimize / hide window after opening file or app
      invoke("hide_window").catch((err) => console.error(err));
    } catch (err) {
      console.error("Failed to open item:", err);
    }
  };

  const handleOpenItemWith = async (item: FileItem) => {
    try {
      await invoke("open_item_with", { path: item.path });
      // Auto minimize / hide window after opening Open With dialog
      invoke("hide_window").catch((err) => console.error(err));
    } catch (err) {
      console.error("Failed to open with:", err);
    }
  };

  const handleRevealItem = async (item: FileItem) => {
    try {
      await invoke("reveal_item", { path: item.path });
      // Auto minimize / hide window after revealing in Explorer
      invoke("hide_window").catch((err) => console.error(err));
    } catch (err) {
      console.error("Failed to reveal item:", err);
    }
  };

  const handleCopyPath = async (item: FileItem) => {
    try {
      await navigator.clipboard.writeText(item.path);
    } catch (err) {
      console.error("Failed to copy path:", err);
    }
  };

  const handleReindex = async () => {
    try {
      await invoke("start_indexing");
    } catch (err) {
      console.error("Failed to start indexing:", err);
    }
  };

  // Keyboard navigation & Shortcuts
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    // Read the real number of rendered grid columns so Up/Down moves straight vertically
    const getGridCols = () => {
      const grid = document.querySelector(".thumbnail-grid-container") as HTMLElement | null;
      if (!grid) return 1;
      const cols = getComputedStyle(grid).gridTemplateColumns.split(" ").filter(Boolean).length;
      return Math.max(1, cols);
    };
    const gridCols = viewMode === "grid" ? getGridCols() : 1;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => {
        const last = results.length - 1;
        const next = prev + gridCols;
        if (next <= last) return next;
        // Partial last row: drop to last item only if a lower row exists, otherwise stay
        return Math.floor(prev / gridCols) < Math.floor(last / gridCols) ? last : prev;
      });
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - gridCols >= 0 ? prev - gridCols : prev));
    } else if (e.key === "ArrowRight" && viewMode === "grid") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
    } else if (e.key === "ArrowLeft" && viewMode === "grid") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.max(prev - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (!isCompact && results[selectedIndex]) {
        if (e.shiftKey) {
          handleRevealItem(results[selectedIndex]);
        } else {
          handleOpenItem(results[selectedIndex]);
        }
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      if (isSettingsOpen) {
        setIsSettingsOpen(false);
      } else if (query.length > 0) {
        setQuery("");
      } else {
        invoke("hide_window").catch((err) => console.error(err));
      }
    }
  };

  const selectedItem = results[selectedIndex] ?? null;

  return (
    <div className={`spotlight-window ${isCompact ? "compact" : ""} ${theme === "light" ? "theme-light" : ""}`}>
      <header className="spotlight-header">
        <SearchBar
          value={query}
          onChange={setQuery}
          onClear={() => setQuery("")}
          onKeyDown={handleKeyDown}
          isLoading={isLoading}
          elapsedMs={elapsedMs}
          matchCount={totalMatched}
          onOpenSettings={() => setIsSettingsOpen(true)}
          viewMode={viewMode}
          onToggleViewMode={handleToggleViewMode}
        />

        {!isCompact && (
          <CategoryFilter
            activeCategory={activeCategory}
            onSelectCategory={(cat) => {
              setActiveCategory(cat);
              // Auto-switch to grid view if user clicks "image" category
              if (cat === "image") {
                setViewMode("grid");
                localStorage.setItem("nexora_view_mode", "grid");
              }
            }}
            drives={drives}
            activeDrive={activeDrive}
            onSelectDrive={setActiveDrive}
            folderScope={folderScope}
            onClearFolderScope={() => setFolderScope(null)}
          />
        )}
      </header>

      {!isCompact && (
        <>
          <main className="spotlight-body">
            {viewMode === "grid" ? (
              <ThumbnailGrid
                items={results}
                selectedIndex={selectedIndex}
                onSelect={setSelectedIndex}
                onOpen={handleOpenItem}
                onContextMenu={(item, e) => {
                  setContextMenu({ x: e.clientX, y: e.clientY, item });
                }}
                query={query}
                isLoading={isLoading}
              />
            ) : (
              <ResultsList
                items={results}
                selectedIndex={selectedIndex}
                onSelect={setSelectedIndex}
                onOpen={handleOpenItem}
                onContextMenu={(item, e) => {
                  setContextMenu({ x: e.clientX, y: e.clientY, item });
                }}
                query={query}
                isLoading={isLoading}
              />
            )}
          </main>

          <PreviewPane
            item={selectedItem}
            onOpen={handleOpenItem}
            onReveal={handleRevealItem}
            onCopyPath={handleCopyPath}
          />

          <StatusBar status={indexingStatus} onReindex={handleReindex} />
        </>
      )}

      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        onReindex={handleReindex}
        totalIndexed={indexingStatus?.total_indexed ?? 0}
        isIndexing={indexingStatus?.is_indexing ?? false}
        availableDrives={drives}
        theme={theme}
        onThemeChange={handleThemeChange}
        onCheckUpdate={() => checkForUpdates(true)}
        isCheckingUpdate={isCheckingUpdate}
      />

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          item={contextMenu.item}
          onClose={() => setContextMenu(null)}
          onOpen={handleOpenItem}
          onOpenWith={handleOpenItemWith}
          onReveal={handleRevealItem}
          onCopyPath={handleCopyPath}
        />
      )}

      <UpdateModal
        isOpen={Boolean(updateAvailable)}
        version={updateAvailable?.version ?? ""}
        body={updateAvailable?.body}
        isUpdating={isUpdating}
        onClose={() => setUpdateAvailable(null)}
        onUpdate={async () => {
          if (!updateAvailable?.updaterObj) return;
          try {
            setIsUpdating(true);
            await updateAvailable.updaterObj.downloadAndInstall();
            await relaunch();
          } catch (err) {
            console.error("Gagal update:", err);
            setUpdateNotice({
              type: "warning",
              title: "Gagal Memasang Pembaruan",
              message: "Terjadi kendala saat mengunduh berkas instalasi pembaruan. Silakan coba kembali nanti.",
            });
          } finally {
            setIsUpdating(false);
          }
        }}
      />

      <UpdateStatusModal
        notice={updateNotice}
        onClose={() => setUpdateNotice(null)}
      />
    </div>
  );
}

export default App;
