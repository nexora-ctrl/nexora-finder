import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FileItem } from "../types";
import { FileIcon } from "./FileIcon";
import { formatBytes } from "../utils/format";

interface ThumbnailItemProps {
  item: FileItem;
  isSelected: boolean;
  onSelect: () => void;
  onOpen: () => void;
}

// Global cache for thumbnail Data URLs in browser memory to eliminate refetch flickering
const thumbnailDataCache = new Map<string, string>();

export const ThumbnailCard: React.FC<ThumbnailItemProps> = ({
  item,
  isSelected,
  onSelect,
  onOpen,
}) => {
  const [thumbUrl, setThumbUrl] = useState<string | null>(
    thumbnailDataCache.get(item.path) || null
  );
  const [isError, setIsError] = useState(false);

  useEffect(() => {
    let isMounted = true;
    if (thumbnailDataCache.has(item.path)) {
      setThumbUrl(thumbnailDataCache.get(item.path)!);
      return;
    }

    // Request Windows thumbnail
    invoke<string>("get_thumbnail", { path: item.path, size: 180 })
      .then((dataUri) => {
        if (isMounted && dataUri) {
          thumbnailDataCache.set(item.path, dataUri);
          setThumbUrl(dataUri);
        }
      })
      .catch(() => {
        if (isMounted) {
          setIsError(true);
        }
      });

    return () => {
      isMounted = false;
    };
  }, [item.path]);

  return (
    <div
      className={`thumbnail-card ${isSelected ? "selected" : ""}`}
      onClick={onSelect}
      onDoubleClick={onOpen}
      title={item.path}
    >
      <div className="thumb-preview-box">
        {thumbUrl && !isError ? (
          <img
            src={thumbUrl}
            alt={item.filename}
            className="thumb-image-preview"
            loading="lazy"
          />
        ) : (
          <div className="thumb-fallback-icon">
            <FileIcon
              extension={item.extension}
              isDirectory={item.is_directory}
              size={36}
            />
          </div>
        )}
        {item.extension && !item.is_directory && (
          <span className="thumb-ext-badge">{item.extension.toUpperCase()}</span>
        )}
      </div>

      <div className="thumb-info-box">
        <span className="thumb-filename" title={item.filename}>
          {item.filename}
        </span>
        <span className="thumb-filesize">
          {item.is_directory ? "Folder" : formatBytes(item.size_bytes)}
        </span>
      </div>
    </div>
  );
};

