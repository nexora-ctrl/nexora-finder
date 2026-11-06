import React, { useRef, useEffect } from "react";
import { Search, X, Loader2, Settings } from "lucide-react";

interface SearchBarProps {
  value: string;
  onChange: (val: string) => void;
  onClear: () => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  isLoading: boolean;
  elapsedMs: number | null;
  matchCount: number | null;
  onOpenSettings: () => void;
}

export const SearchBar: React.FC<SearchBarProps> = ({
  value,
  onChange,
  onClear,
  onKeyDown,
  isLoading,
  elapsedMs,
  matchCount,
  onOpenSettings,
}) => {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div className="search-bar-wrapper">
      <div className="search-icon-container">
        {isLoading ? (
          <Loader2 className="animate-spin text-cyan-400" size={22} />
        ) : (
          <Search className="text-cyan-400" size={22} />
        )}
      </div>

      <input
        ref={inputRef}
        type="text"
        className="search-input"
        placeholder="Ketik untuk mencari berkas atau folder..."
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={onKeyDown}
        autoFocus
        spellCheck={false}
      />

      <div className="search-badge-container">
        {elapsedMs !== null && matchCount !== null && (
          <span className="search-stats-badge">
            {matchCount.toLocaleString("id-ID")} hasil ({elapsedMs.toFixed(1)} ms)
          </span>
        )}

        {value.length > 0 && (
          <button
            type="button"
            className="search-clear-btn"
            onClick={onClear}
            title="Bersihkan (Esc)"
          >
            <X size={15} />
          </button>
        )}

        <button
          type="button"
          className="search-settings-btn"
          onClick={onOpenSettings}
          title="Pengaturan"
        >
          <Settings size={16} />
        </button>
      </div>
    </div>
  );
};

