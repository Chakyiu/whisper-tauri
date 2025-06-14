use crate::types::*;
use anyhow::{anyhow, Result};
use dirs;
use serde_json;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ConfigManager {
    config_dir: PathBuf,
    models_dir: PathBuf,
    settings_file: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Unable to find home directory"))?;
        let config_dir = home_dir.join(".whisper-tauri");
        let models_dir = config_dir.join("models");
        let settings_file = config_dir.join("settings.json");

        // Create directories if they don't exist
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&models_dir)?;

        Ok(ConfigManager {
            config_dir,
            models_dir,
            settings_file,
        })
    }

    pub fn get_models_dir(&self) -> &PathBuf {
        &self.models_dir
    }

    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn save_settings(&self, settings: &TranscriptionSettings) -> Result<()> {
        let json = serde_json::to_string_pretty(settings)?;
        fs::write(&self.settings_file, json)?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<TranscriptionSettings> {
        if self.settings_file.exists() {
            let content = fs::read_to_string(&self.settings_file)?;
            let settings: TranscriptionSettings = serde_json::from_str(&content)?;
            Ok(settings)
        } else {
            // Return default settings
            Ok(TranscriptionSettings {
                language: None,
                model: "base".to_string(),
                output_format: OutputFormat::Srt,
                keep_wav: false,
                output_dir: None,
                parallel_jobs: 1,
            })
        }
    }

    pub fn get_available_models(&self) -> Vec<WhisperModel> {
        let mut models = vec![
            WhisperModel {
                name: "tiny".to_string(),
                size: "39 MB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
                    .to_string(),
                downloaded: false,
                file_path: None,
            },
            WhisperModel {
                name: "base".to_string(),
                size: "142 MB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
                    .to_string(),
                downloaded: false,
                file_path: None,
            },
            WhisperModel {
                name: "small".to_string(),
                size: "466 MB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
                    .to_string(),
                downloaded: false,
                file_path: None,
            },
            WhisperModel {
                name: "medium".to_string(),
                size: "1.5 GB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
                    .to_string(),
                downloaded: false,
                file_path: None,
            },
            WhisperModel {
                name: "large-v1".to_string(),
                size: "2.9 GB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v1.bin"
                    .to_string(),
                downloaded: false,
                file_path: None,
            },
            WhisperModel {
                name: "large-v2".to_string(),
                size: "2.9 GB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v2.bin"
                    .to_string(),
                downloaded: false,
                file_path: None,
            },
            WhisperModel {
                name: "large-v3".to_string(),
                size: "2.9 GB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                    .to_string(),
                downloaded: false,
                file_path: None,
            },
            WhisperModel {
                name: "large-v3-turbo".to_string(),
                size: "1.6 GB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin"
                    .to_string(),
                downloaded: false,
                file_path: None,
            },
        ];

        // Check which models are already downloaded
        for model in &mut models {
            let model_path = self.models_dir.join(format!("ggml-{}.bin", model.name));
            if model_path.exists() {
                model.downloaded = true;
                model.file_path = Some(model_path);
            }
        }

        models
    }

    pub fn get_model_path(&self, model_name: &str) -> PathBuf {
        self.models_dir.join(format!("ggml-{}.bin", model_name))
    }
}
