import { useState, useCallback, DragEvent } from "react";

interface Props {
  onFileChosen: (path: string) => void;
  onBrowse: () => void;
  errorMessage: string | null;
}

export function DropZone({ onFileChosen, onBrowse, errorMessage }: Props) {
  const [isDragOver, setIsDragOver] = useState(false);

  const handleDragOver = useCallback((e: DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
  }, []);

  // Tauri's native drag-drop event (wired in App.tsx via the webview API)
  // handles the actual file path resolution on Windows; this HTML handler
  // exists for hover-state visuals only.
  const handleDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault();
      setIsDragOver(false);
    },
    []
  );

  return (
    <div
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 20,
        margin: 20,
        borderRadius: 12,
        border: `1.5px dashed ${isDragOver ? "#9fb4c4" : "#3a3a3a"}`,
        background: isDragOver ? "rgba(122, 139, 153, 0.06)" : "transparent",
        transition: "border-color 120ms ease, background 120ms ease",
        cursor: "pointer",
      }}
      onClick={onBrowse}
      role="button"
      tabIndex={0}
      aria-label="Import an image"
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") onBrowse();
      }}
    >
      <ImportGlyph active={isDragOver} />
      <div style={{ textAlign: "center" }}>
        <div style={{ fontSize: 16, fontWeight: 600, color: "#e8e6e1", marginBottom: 6 }}>
          Drop an image here
        </div>
        <div style={{ fontSize: 13, color: "#8b8781" }}>
          or click to browse &middot; PNG, JPG, WebP
        </div>
      </div>

      {errorMessage && (
        <div
          style={{
            marginTop: 8,
            fontSize: 13,
            color: "#e8746a",
            background: "rgba(232, 116, 106, 0.1)",
            border: "1px solid rgba(232, 116, 106, 0.3)",
            borderRadius: 6,
            padding: "8px 14px",
            maxWidth: 340,
            textAlign: "center",
          }}
        >
          {errorMessage}
        </div>
      )}
    </div>
  );
}

function ImportGlyph({ active }: { active: boolean }) {
  const color = active ? "#9fb4c4" : "#5a5652";
  return (
    <svg width="40" height="40" viewBox="0 0 40 40" fill="none">
      <path
        d="M20 6V26M20 6L13 13M20 6L27 13"
        stroke={color}
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M7 26V31C7 32.6569 8.34315 34 10 34H30C31.6569 34 33 32.6569 33 31V26"
        stroke={color}
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
