import { useState, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";

interface DropZoneProps {
  onFilesAdded: (files: string[]) => void;
}

export default function DropZone({ onFilesAdded }: DropZoneProps) {
  const [isDragOver, setIsDragOver] = useState(false);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setIsDragOver(false);

      const items = Array.from(e.dataTransfer.items);
      const files: string[] = [];

      items.forEach((item) => {
        const entry = item.webkitGetAsEntry?.();
        if (entry) {
          if (entry.isFile) {
            const file = item.getAsFile();
            if (file?.name.toLowerCase().endsWith(".pdf")) {
              files.push(file.name);
            }
          } else if (entry.isDirectory) {
            files.push(entry.name);
          }
        }
      });

      if (files.length > 0) {
        onFilesAdded(files);
      }
    },
    [onFilesAdded]
  );

  const handleClick = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        directory: false,
        filters: [
          {
            name: "PDF 文件",
            extensions: ["pdf"],
          },
        ],
      });
      if (selected && Array.isArray(selected)) {
        onFilesAdded(selected);
      }
    } catch (error) {
      console.error("Failed to open file dialog:", error);
    }
  }, [onFilesAdded]);

  const handleSelectFolder = useCallback(async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      const selected = await open({
        directory: true,
        multiple: true,
      });
      if (selected) {
        const folders = Array.isArray(selected) ? selected : [selected];
        onFilesAdded(folders);
      }
    } catch (error) {
      console.error("Failed to open folder dialog:", error);
    }
  }, [onFilesAdded]);

  return (
    <div
      className={`drop-zone ${isDragOver ? "drag-over" : ""}`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      onClick={handleClick}
    >
      <div className="drop-zone-icon">📥</div>
      <div className="drop-zone-text">拖入 PDF 文件或文件夹</div>
      <div className="drop-zone-hint">或点击选择文件</div>
      <div style={{ marginTop: 10 }}>
        <button
          className="btn btn-secondary btn-small"
          onClick={handleSelectFolder}
        >
          选择文件夹
        </button>
      </div>
    </div>
  );
}
