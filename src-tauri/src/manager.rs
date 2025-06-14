use anyhow::Result;

use crate::audio_converter::AudioConverter;
use crate::config::ConfigManager;
use crate::types::*;

pub struct TranscriptionManager {
    config: ConfigManager,
}

impl TranscriptionManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: ConfigManager::new()?,
        })
    }

    pub fn get_available_models(&self) -> Vec<WhisperModel> {
        self.config.get_available_models()
    }
}
