use crate::types::*;
use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use reqwest;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct ModelDownloader {
    client: reqwest::Client,
}

impl ModelDownloader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn download_model<F>(
        &self,
        model: &WhisperModel,
        output_path: &Path,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(i32) + Send + Sync,
    {
        log::debug!("Downloading Model: {}", model.name);
        let response = self.client.get(&model.url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download model: HTTP {}",
                response.status()
            ));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        let mut file = File::create(output_path).await?;

        let mut last_progress: i32 = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if total_size > 0  {
                let progress: i32 = ((downloaded as f32 / total_size as f32) * 100.0) as i32;
                if progress > last_progress {
                    last_progress = progress;
                    progress_callback(progress);
                }
            }
        }

        file.flush().await?;
        log::debug!("Download model complete!");
        Ok(())
    }

    pub async fn check_model_availability(&self, url: &str) -> Result<u64> {
        let response = self.client.head(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Model not available: HTTP {}", response.status()));
        }

        Ok(response.content_length().unwrap_or(0))
    }
}
