// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod inference;
mod pipeline;
mod error;

use inference::InferenceEngine;
use std::sync::Mutex;
use tauri::{Manager, State};

/// Holds the loaded ONNX session for the lifetime of the app.
/// The model is loaded exactly once, on first use, and reused for every
/// subsequent image so we never pay model-init cost twice.
pub struct AppState {
    engine: Mutex<Option<InferenceEngine>>,
    model_path: Mutex<Option<std::path::PathBuf>>,
}

#[tauri::command]
async fn remove_background(
    state: State<'_, AppState>,
    input_path: String,
) -> Result<pipeline::RemovalResult, String> {
    pipeline::process_image(&state, &input_path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_png(source_rgba_path: String, dest_path: String) -> Result<(), String> {
    pipeline::export_png(&source_rgba_path, &dest_path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn warm_up_model(state: State<'_, AppState>) -> Result<(), String> {
    pipeline::ensure_engine_loaded(&state).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            engine: Mutex::new(None),
            model_path: Mutex::new(None),
        })
        .setup(|app| {
            // Resolve the bundled model resource path once at startup and
            // stash it so the pipeline can lazily load it on first request.
            let resource_path = app
                .path()
                .resolve("models/isnet-general-use.onnx", tauri::path::BaseDirectory::Resource)
                .expect("failed to resolve bundled model path");

            let state: State<AppState> = app.state();
            *state.model_path.lock().unwrap() = Some(resource_path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            remove_background,
            export_png,
            warm_up_model
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
