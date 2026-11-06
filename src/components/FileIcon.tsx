import React from "react";
import {
  Folder,
  FileText,
  FileSpreadsheet,
  Image as ImageIcon,
  Film,
  Music,
  Terminal,
  Archive,
  Code2,
  File,
} from "lucide-react";

interface FileIconProps {
  extension: string;
  isDirectory: boolean;
  className?: string;
  size?: number;
}

export const FileIcon: React.FC<FileIconProps> = ({
  extension,
  isDirectory,
  className = "",
  size = 18,
}) => {
  if (isDirectory) {
    return <Folder size={size} className={`text-amber-400 fill-amber-400/20 ${className}`} />;
  }

  const ext = extension.toLowerCase();

  switch (ext) {
    case "pdf":
      return <FileText size={size} className={`text-rose-400 ${className}`} />;
    case "doc":
    case "docx":
    case "txt":
    case "md":
    case "rtf":
      return <FileText size={size} className={`text-sky-400 ${className}`} />;
    case "xls":
    case "xlsx":
    case "csv":
      return <FileSpreadsheet size={size} className={`text-emerald-400 ${className}`} />;
    case "png":
    case "jpg":
    case "jpeg":
    case "webp":
    case "gif":
    case "bmp":
    case "svg":
    case "ico":
      return <ImageIcon size={size} className={`text-teal-400 ${className}`} />;
    case "mp4":
    case "mkv":
    case "avi":
    case "mov":
    case "wmv":
    case "webm":
      return <Film size={size} className={`text-purple-400 ${className}`} />;
    case "mp3":
    case "wav":
    case "flac":
    case "aac":
    case "ogg":
    case "m4a":
      return <Music size={size} className={`text-pink-400 ${className}`} />;
    case "exe":
    case "msi":
    case "bat":
    case "cmd":
    case "ps1":
    case "lnk":
      return <Terminal size={size} className={`text-indigo-400 ${className}`} />;
    case "zip":
    case "rar":
    case "7z":
    case "tar":
    case "gz":
    case "iso":
      return <Archive size={size} className={`text-amber-500 ${className}`} />;
    case "rs":
    case "ts":
    case "tsx":
    case "js":
    case "jsx":
    case "py":
    case "html":
    case "css":
    case "json":
    case "c":
    case "cpp":
    case "h":
      return <Code2 size={size} className={`text-cyan-400 ${className}`} />;
    default:
      return <File size={size} className={`text-slate-400 ${className}`} />;
  }
};

