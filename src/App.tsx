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
import { PreviewPane } from "./components/PreviewPane";
import { StatusBar } from "./components/StatusBar";
import { SettingsModal } from "./components/SettingsModal";
import "./App.css";

export function App() {
  const [query, setQuery] = useState("");
  const [activeCategory, setActiveCategory] = useState<CategoryType>("all");
  const [activeDrive, setActiveDrive] = useState("ALL");
  const [results, setResults] = useState<FileItem[]>([]);
  const [totalMatched, setTotalMatched] = useState<number | null>(null);
  const [elapsedMs, setElapsedMs] = useState<number | null>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [indexingStatus, setIndexingStatus] = useState<IndexingStatus | null>(null);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  const isCompact = query.trim().length === 0 && !isSettingsOpen;

  const debounceTimer = useRef<number | null>(null);

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
      unlistenStatusPromise.then((unlisten) => unlisten());
      unlistenFocusPromise.then((unlisten) => unlisten());
      window.removeEventListener("keydown", handleGlobalWindowKeyDown);
    };
  }, []);

  // Search function
  const performSearch = useCallback(
    async (searchTerm: string, cat: CategoryType, drv: string) => {
      if (searchTerm.trim().length === 0) {
        // When query is empty, show recent files
        try {
          const recent = await invoke<FileItem[]>("get_recent_files", { limit: 25 });
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
            limit: 150,
          },
        });

        setResults(res.items);
        setTotalMatched(res.total_matched);
        setElapsedMs(res.elapsed_ms);
        setSelectedIndex(0);
      } catch (err) {
        console.error("Search failed:", err);
        setResults([]);
        setTotalMatched(0);
      } finally {
        setIsLoading(false);
      }
    },
    [],
  );

  // Debounced search trigger when query or filters change
  useEffect(() => {
    if (debounceTimer.current) {
      clearTimeout(debounceTimer.current);
    }

    debounceTimer.current = window.setTimeout(() => {
      performSearch(query, activeCategory, activeDrive);
    }, 25);

    return () => {
      if (debounceTimer.current) {
        clearTimeout(debounceTimer.current);
      }
    };
  }, [query, activeCategory, activeDrive, performSearch]);

  const handleOpenItem = async (item: FileItem) => {
    try {
      await invoke("open_item", { path: item.path });
    } catch (err) {
      console.error("Failed to open item:", err);
    }
  };

  const handleRevealItem = async (item: FileItem) => {
    try {
      await invoke("reveal_item", { path: item.path });
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
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
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
    <div className={`spotlight-window ${isCompact ? "compact" : ""}`}>
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
        />

        {!isCompact && (
          <CategoryFilter
            activeCategory={activeCategory}
            onSelectCategory={setActiveCategory}
            drives={drives}
            activeDrive={activeDrive}
            onSelectDrive={setActiveDrive}
          />
        )}
      </header>

      {!isCompact && (
        <>
          <main className="spotlight-body">
            <ResultsList
              items={results}
              selectedIndex={selectedIndex}
              onSelect={setSelectedIndex}
              onOpen={handleOpenItem}
              query={query}
              isLoading={isLoading}
            />
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
      />
    </div>
  );
}

export default App;
