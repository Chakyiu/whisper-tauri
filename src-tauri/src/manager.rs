use crate::audio_converter::AudioConverter;
use crate::config::ConfigManager;
use crate::model_downloader::ModelDownloader;
use crate::transcriber::WhisperTranscriber;
use crate::types::*;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct TranscriptionManager {
    config: ConfigManager,
    downloader: ModelDownloader,
    progress_sender: Option<mpsc::UnboundedSender<ProgressUpdate>>,
    jobs: Arc<Mutex<HashMap<String, TranscriptionJob>>>,
    active_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl TranscriptionManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: ConfigManager::new()?,
            downloader: ModelDownloader::new(),
            progress_sender: None,
            jobs: Arc::new(Mutex::new(HashMap::new())),
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn get_available_models(&self) -> Vec<WhisperModel> {
        self.config.get_available_models()
    }

    pub async fn download_model(
        &self,
        model_name: &str,
        progress_callback: impl Fn(f32) + Send + Sync + 'static,
    ) -> Result<()> {
        let models = self.config.get_available_models();
        let model = models
            .iter()
            .find(|m| m.name == model_name)
            .ok_or_else(|| anyhow!("Model not found: {}", model_name))?;

        let output_path = self.config.get_model_path(model_name);

        self.downloader
            .download_model(model, &output_path, progress_callback)
            .await?;

        Ok(())
    }

    pub fn save_settings(&self, settings: &TranscriptionSettings) -> Result<()> {
        self.config.save_settings(settings)
    }

    pub fn load_settings(&self) -> Result<TranscriptionSettings> {
        self.config.load_settings()
    }

    pub async fn add_files(&self, file_paths: Vec<PathBuf>) -> Vec<FileEntry> {
        let mut files = Vec::new();

        for path in file_paths {
            let converter = AudioConverter::new();
            if !converter.is_audio_file(&path) {
                continue;
            }

            let id = Uuid::new_v4().to_string();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

            let file_entry = FileEntry {
                id,
                path,
                name,
                size,
                status: FileStatus::Pending,
                progress: 0.0,
                error: None,
                output_path: None,
            };

            files.push(file_entry);
        }

        files
    }

    pub fn set_progress_sender(&mut self, sender: mpsc::UnboundedSender<ProgressUpdate>) {
        self.progress_sender = Some(sender);
    }

    pub async fn start_transcription(
        &self,
        files: Vec<FileEntry>,
        settings: TranscriptionSettings,
    ) -> Result<()> {
        let model_path = self.config.get_model_path(&settings.model);

        if !model_path.exists() {
            return Err(anyhow!("Model not downloaded: {}", settings.model));
        }

        // Create jobs
        let mut jobs_map = self.jobs.lock().await;
        for file in files {
            let job = TranscriptionJob {
                id: file.id.clone(),
                file_path: file.path.clone(),
                settings: settings.clone(),
                status: FileStatus::Pending,
                progress: 0.0,
                error: None,
                output_path: None,
            };
            jobs_map.insert(file.id, job);
        }
        drop(jobs_map);

        // Start processing jobs
        self.process_jobs(settings.parallel_jobs).await;

        Ok(())
    }

    async fn process_jobs(&self, max_parallel: usize) {
        let jobs = self.jobs.clone();
        let active_tasks = self.active_tasks.clone();
        let progress_sender = self.progress_sender.clone();
        let config = self.config.clone();

        // Get pending jobs
        let pending_jobs: Vec<TranscriptionJob> = {
            let jobs_map = jobs.lock().await;
            jobs_map
                .values()
                .filter(|job| matches!(job.status, FileStatus::Pending))
                .cloned()
                .collect()
        };

        // Process jobs in chunks
        for chunk in pending_jobs.chunks(max_parallel) {
            let mut handles = Vec::new();

            for job in chunk {
                let job = job.clone();
                let jobs_clone = jobs.clone();
                let progress_sender = progress_sender.clone();
                let config_clone = config.clone();

                let handle = tokio::spawn(async move {
                    Self::process_single_job(job, jobs_clone, progress_sender, config_clone).await;
                });

                handles.push(handle);
            }

            // Wait for all jobs in this chunk to complete
            for handle in handles {
                let _ = handle.await;
            }
        }
    }

    async fn process_single_job(
        mut job: TranscriptionJob,
        jobs: Arc<Mutex<HashMap<String, TranscriptionJob>>>,
        progress_sender: Option<mpsc::UnboundedSender<ProgressUpdate>>,
        config: ConfigManager,
    ) {
        // Update job status
        job.status = FileStatus::Converting;
        Self::update_job_progress(&jobs, &job, progress_sender.as_ref()).await;

        // Convert audio to WAV
        let wav_path = Self::get_temp_wav_path(&job.file_path);
        let converter = AudioConverter::new();

        let convert_result = converter.convert_to_wav(&job.file_path, &wav_path);

        if let Err(e) = convert_result {
            job.status = FileStatus::Error;
            job.error = Some(format!("Conversion failed: {}", e));
            Self::update_job_progress(&jobs, &job, progress_sender.as_ref()).await;
            return;
        }

        // Load model and transcribe
        job.status = FileStatus::Transcribing;
        job.progress = 30.0;
        Self::update_job_progress(&jobs, &job, progress_sender.as_ref()).await;

        let model_path = config.get_model_path(&job.settings.model);
        let mut transcriber = WhisperTranscriber::new();

        if let Err(e) = transcriber.load_model(&model_path) {
            job.status = FileStatus::Error;
            job.error = Some(format!("Failed to load model: {}", e));
            Self::update_job_progress(&jobs, &job, progress_sender.as_ref()).await;
            return;
        }

        let transcription_result = transcriber.transcribe_file(
            &wav_path,
            &job.settings,
            Some(Box::new({
                let jobs = jobs.clone();
                let job_id = job.id.clone();
                let progress_sender = progress_sender.clone();
                move |progress| {
                    let jobs = jobs.clone();
                    let job_id = job_id.clone();
                    let progress_sender = progress_sender.clone();
                    tokio::spawn(async move {
                        let mut jobs_map = jobs.lock().await;
                        if let Some(job) = jobs_map.get_mut(&job_id) {
                            job.progress = 30.0 + (progress * 0.7); // 70% for transcription
                            job.status = FileStatus::Transcribing;
                        }
                        if let Some(sender) = &progress_sender {
                            let _ = sender.send(ProgressUpdate {
                                file_id: job_id.clone(),
                                status: FileStatus::Transcribing,
                                progress: 30.0 + (progress * 0.7),
                                message: Some("Transcribing...".to_string()),
                            });
                        }
                    });
                }
            })),
        );

        log::debug!("Transcribed: result={:?}", transcription_result);

        match transcription_result {
            Ok(text) => {
                // Save output
                let output_path = Self::get_output_path(&job.file_path, &job.settings);
                if let Err(e) = std::fs::write(&output_path, text) {
                    job.status = FileStatus::Error;
                    job.error = Some(format!("Failed to save output: {}", e));
                } else {
                    job.status = FileStatus::Completed;
                    job.progress = 100.0;
                    job.output_path = Some(output_path);
                }
            }
            Err(e) => {
                job.status = FileStatus::Error;
                job.error = Some(format!("Transcription failed: {}", e));
            }
        }

        // Clean up WAV file if needed
        if !job.settings.keep_wav {
            let _ = std::fs::remove_file(&wav_path);
        }

        Self::update_job_progress(&jobs, &job, progress_sender.as_ref()).await;
    }

    async fn update_job_progress(
        jobs: &Arc<Mutex<HashMap<String, TranscriptionJob>>>,
        job: &TranscriptionJob,
        progress_sender: Option<&mpsc::UnboundedSender<ProgressUpdate>>,
    ) {
        // Update job in map
        let mut jobs_map = jobs.lock().await;
        jobs_map.insert(job.id.clone(), job.clone());

        // Send progress update
        if let Some(sender) = progress_sender {
            let _ = sender.send(ProgressUpdate {
                file_id: job.id.clone(),
                status: job.status.clone(),
                progress: job.progress,
                message: job.error.clone(),
            });
        }
    }

    fn get_temp_wav_path(input_path: &Path) -> PathBuf {
        let mut wav_path = input_path.to_path_buf();
        wav_path.set_extension("wav");
        wav_path
    }

    fn get_output_path(input_path: &Path, settings: &TranscriptionSettings) -> PathBuf {
        let default_dir = input_path.parent().unwrap().to_path_buf();
        let output_dir = settings.output_dir.as_ref().unwrap_or(&default_dir);

        let mut output_path = output_dir.join(input_path.file_stem().unwrap_or_default());
        output_path.set_extension(settings.output_format.extension());
        output_path
    }

    pub async fn get_job_status(&self, job_id: &str) -> Option<TranscriptionJob> {
        self.jobs.lock().await.get(job_id).cloned()
    }

    pub async fn get_all_jobs(&self) -> Vec<TranscriptionJob> {
        self.jobs.lock().await.values().cloned().collect()
    }

    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        // Cancel the task if it's running
        let mut tasks = self.active_tasks.lock().await;
        if let Some(handle) = tasks.remove(job_id) {
            handle.abort();
        }

        // Update job status
        let mut jobs = self.jobs.lock().await;
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = FileStatus::Error;
            job.error = Some("Cancelled by user".to_string());
        }

        Ok(())
    }

    pub async fn clear_completed_jobs(&self) {
        let mut jobs = self.jobs.lock().await;
        jobs.retain(|_, job| !matches!(job.status, FileStatus::Completed));
    }
}
