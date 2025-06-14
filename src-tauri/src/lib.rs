mod audio_converter;
mod config;
mod manager;
mod types;

use manager::TranscriptionManager;
use types::*;

use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::{mpsc, Mutex};

type TranscriptionState = Arc<Mutex<TranscriptionManager>>;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_available_models(
    state: State<'_, TranscriptionState>,
) -> Result<Vec<WhisperModel>, String> {
    let manager = state.lock().await;
    Ok(manager.get_available_models())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .build(),
        )
        .setup(|app| {
            let manager = TranscriptionManager::new()
                .map_err(|e| format!("Failed to initialize transcription manager: {}", e))?;

            app.manage(Arc::new(Mutex::new(manager)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, get_available_models])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
