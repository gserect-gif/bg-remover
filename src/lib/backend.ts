import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

export interface RemovalResult {
  preview_path: string;
  width: number;
  height: number;
}

/** Loads the ONNX model into memory ahead of the first real request,
 * so the user's first image doesn't pay model-init latency on top of
 * inference latency. Safe to call multiple times; backend no-ops if
 * already loaded. */
export async function warmUpModel(): Promise<void> {
  return invoke("warm_up_model");
}

export async function removeBackground(inputPath: string): Promise<RemovalResult> {
  return invoke("remove_background", { inputPath });
}

export async function exportPng(sourceRgbaPath: string, destPath: string): Promise<void> {
  return invoke("export_png", { sourceRgbaPath, destPath });
}

export async function pickImageFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] }],
  });
  if (!selected) return null;
  return Array.isArray(selected) ? selected[0] : selected;
}

export async function pickSaveLocation(suggestedName: string): Promise<string | null> {
  return save({
    defaultPath: suggestedName,
    filters: [{ name: "PNG Image", extensions: ["png"] }],
  });
}
