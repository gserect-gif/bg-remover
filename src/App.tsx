import { useEffect, useState, useCallback, useRef } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { DropZone } from "./components/DropZone";
import { PreviewSurface } from "./components/PreviewSurface";
import { ControlRail } from "./components/ControlRail";
import {
  warmUpModel,
  removeBackground,
  exportPng,
  pickImageFile,
  pickSaveLocation,
} from "./lib/backend";

export type AppPhase = "idle" | "loading_model" | "processing" | "done" | "error";

const SUPPORTED_EXT = ["png", "jpg", "jpeg", "webp"];

export default function App() {
  const [phase, setPhase] = useState<AppPhase>("idle");
  const [fileName, setFileName] = useState<string | null>(null);
  const [previewPath, setPreviewPath] = useState<string | null>(null);
  const [dimensions, setDimensions] = useState<{ w: number; h: number } | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const modelWarmedUp = useRef(false);

  // Warm up the model as soon as the app opens (not on first image) so
  // the user's actual first removal is fast, not paying init cost.
  useEffect(() => {
    if (modelWarmedUp.current) return;
    modelWarmedUp.current = true;
    warmUpModel().catch(() => {
      // Non-fatal here; the real error surfaces on first actual use.
    });
  }, []);

  const runRemoval = useCallback(async (path: string) => {
    setErrorMessage(null);
    setFileName(path.split(/[\\/]/).pop() ?? path);
    setPhase("processing");
    try {
      const result = await removeBackground(path);
      setPreviewPath(result.preview_path);
      setDimensions({ w: result.width, h: result.height });
      setPhase("done");
    } catch (err) {
      setErrorMessage(String(err));
      setPhase("error");
    }
  }, []);

  const handleBrowse = useCallback(async () => {
    const path = await pickImageFile();
    if (path) runRemoval(path);
  }, [runRemoval]);

  const handleExport = useCallback(async () => {
    if (!previewPath || !fileName) return;
    const baseName = fileName.replace(/\.[^.]+$/, "");
    const dest = await pickSaveLocation(`${baseName}-transparent.png`);
    if (!dest) return;
    try {
      await exportPng(previewPath, dest);
    } catch (err) {
      setErrorMessage(String(err));
      setPhase("error");
    }
  }, [previewPath, fileName]);

  const handleReset = useCallback(() => {
    setPhase("idle");
    setFileName(null);
    setPreviewPath(null);
    setDimensions(null);
    setErrorMessage(null);
  }, []);

  // Native OS-level drag-and-drop (works with Windows Explorer, unlike the
  // browser drag events which Tauri's webview intercepts by default).
  useEffect(() => {
    const webview = getCurrentWebview();
    const unlistenPromise = webview.onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        const paths = event.payload.paths;
        if (paths.length === 0) return;
        const first = paths[0];
        const ext = first.split(".").pop()?.toLowerCase() ?? "";
        if (!SUPPORTED_EXT.includes(ext)) {
          setErrorMessage("That file type isn't supported. Use PNG, JPG, or WebP.");
          setPhase("error");
          return;
        }
        runRemoval(first);
      }
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [runRemoval]);

  const showDropZone = phase === "idle" || (phase === "error" && !fileName);

  return (
    <div style={{ display: "flex", height: "100vh", width: "100vw" }}>
      <div style={{ flex: 1, display: "flex", flexDirection: "column", minWidth: 0 }}>
        {showDropZone ? (
          <DropZone onFileChosen={runRemoval} onBrowse={handleBrowse} errorMessage={errorMessage} />
        ) : (
          <PreviewSurface phase={phase} previewPath={previewPath} errorMessage={errorMessage} />
        )}
      </div>

      <ControlRail
        phase={phase}
        fileName={fileName}
        width={dimensions?.w ?? null}
        height={dimensions?.h ?? null}
        onExport={handleExport}
        onReset={handleReset}
        onBrowse={handleBrowse}
      />
    </div>
  );
}
