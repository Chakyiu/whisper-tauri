mod audio_converter;
mod config;
mod manager;
mod model_downloader;
mod transcriber;
mod types;

use manager::TranscriptionManager;
use types::*;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::{mpsc, Mutex};

type TranscriptionState = Arc<Mutex<TranscriptionManager>>;

#[tauri::command]
async fn greet(name: String) -> Result<String, String> {
    Ok(name)
}

#[tauri::command]
async fn get_available_models(
    state: State<'_, TranscriptionState>,
) -> Result<Vec<WhisperModel>, String> {
    let manager = state.lock().await;
    Ok(manager.get_available_models())
}

#[tauri::command]
async fn download_model(
    model_name: String,
    state: State<'_, TranscriptionState>,
    app: AppHandle,
) -> Result<(), String> {
    let manager = state.lock().await;
    let model_name_clone = model_name.clone();
    let app_clone = app.clone();

    manager
        .download_model(&model_name, move |progress| {
            let _ = app_clone.emit(
                "model-download-progress",
                &serde_json::json!({
                    "model": model_name_clone,
                    "progress": progress
                }),
            );
        })
        .await
        .map_err(|e| {
            log::error!("Failed to download model: {}", e);
            e.to_string()
        })?;

    let _ = app.emit("model-download-complete", &model_name);
    Ok(())
}

#[tauri::command]
async fn load_settings(
    state: State<'_, TranscriptionState>,
) -> Result<TranscriptionSettings, String> {
    let manager = state.lock().await;
    manager.load_settings().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_settings(
    settings: TranscriptionSettings,
    state: State<'_, TranscriptionState>,
) -> Result<(), String> {
    let manager = state.lock().await;
    manager.save_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_files(
    file_paths: Vec<String>,
    state: State<'_, TranscriptionState>,
) -> Result<Vec<FileEntry>, String> {
    let paths: Vec<PathBuf> = file_paths.into_iter().map(PathBuf::from).collect();
    let files = {
        let manager = state.lock().await;
        manager.add_files(paths).await
    };
    Ok(files)
}

#[tauri::command]
async fn start_transcription(
    files: Vec<FileEntry>,
    settings: TranscriptionSettings,
    state: State<'_, TranscriptionState>,
    app: AppHandle,
) -> Result<(), String> {
    // Create a separate instance for this operation
    let mut transcription_manager = TranscriptionManager::new().map_err(|e| e.to_string())?;

    // Set up progress reporting
    let (tx, mut rx) = mpsc::unbounded_channel();
    transcription_manager.set_progress_sender(tx);

    // Spawn task to listen for progress updates
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(update) = rx.recv().await {
            let _ = app_clone.emit("transcription-progress", &update);
        }
    });

    // Start transcription in background
    tokio::spawn(async move {
        let _ = transcription_manager
            .start_transcription(files, settings)
            .await;
    });

    Ok(())
}

#[tauri::command]
async fn get_job_status(
    job_id: String,
    state: State<'_, TranscriptionState>,
) -> Result<Option<TranscriptionJob>, String> {
    let manager = state.lock().await;
    Ok(manager.get_job_status(&job_id).await)
}

#[tauri::command]
async fn get_all_jobs(
    state: State<'_, TranscriptionState>,
) -> Result<Vec<TranscriptionJob>, String> {
    let manager = state.lock().await;
    Ok(manager.get_all_jobs().await)
}

#[tauri::command]
async fn cancel_job(job_id: String, state: State<'_, TranscriptionState>) -> Result<(), String> {
    let manager = state.lock().await;
    manager.cancel_job(&job_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_completed_jobs(state: State<'_, TranscriptionState>) -> Result<(), String> {
    let manager = state.lock().await;
    manager.clear_completed_jobs().await;
    Ok(())
}

#[tauri::command]
async fn open_output_folder(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    if let Some(parent) = path_buf.parent() {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(parent)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(parent)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
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
        .invoke_handler(tauri::generate_handler![
            greet,
            get_available_models,
            download_model,
            load_settings,
            save_settings,
            add_files,
            start_transcription,
            get_job_status,
            get_all_jobs,
            cancel_job,
            clear_completed_jobs,
            open_output_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
