use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSettings {
    pub language: Option<String>,
    pub model: String,
    pub output_format: OutputFormat,
    pub keep_wav: bool,
    pub output_dir: Option<PathBuf>,
    pub parallel_jobs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Srt,
    Txt,
    Json,
    Vtt,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Srt => "srt",
            OutputFormat::Txt => "txt",
            OutputFormat::Json => "json",
            OutputFormat::Vtt => "vtt",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub status: FileStatus,
    pub progress: f32,
    pub error: Option<String>,
    pub output_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileStatus {
    Pending,
    Converting,
    Transcribing,
    Completed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModel {
    pub name: String,
    pub size: String,
    pub url: String,
    pub downloaded: bool,
    pub file_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub file_id: String,
    pub status: FileStatus,
    pub progress: f32,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionJob {
    pub id: String,
    pub file_path: PathBuf,
    pub settings: TranscriptionSettings,
    pub status: FileStatus,
    pub progress: f32,
    pub error: Option<String>,
    pub output_path: Option<PathBuf>,
}
