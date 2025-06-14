use crate::config::ConfigManager;
use crate::types::*;

use anyhow::{anyhow, Result};
use hound::{SampleFormat, WavReader};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperTranscriber {
    model_path: Option<String>,
}

impl WhisperTranscriber {
    pub fn new() -> Self {
        Self { model_path: None }
    }

    pub fn load_model(&mut self, model_path: &Path) -> Result<()> {
        let model_path_str = model_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid model path"))?;

        self.model_path = Some(model_path_str.to_string());
        Ok(())
    }

    pub fn parse_wav_file(path: &Path) -> Vec<i16> {
        let reader = WavReader::open(path).expect("failed to read file");

        if reader.spec().channels != 1 {
            panic!("expected mono audio file");
        }
        if reader.spec().sample_format != SampleFormat::Int {
            panic!("expected integer sample format");
        }
        if reader.spec().sample_rate != 16000 {
            panic!("expected 16KHz sample rate");
        }
        if reader.spec().bits_per_sample != 16 {
            panic!("expected 16 bits per sample");
        }

        reader
            .into_samples::<i16>()
            .map(|x| x.expect("sample"))
            .collect::<Vec<_>>()
    }

    pub fn transcribe_file(
        &mut self,
        audio_path: &Path,
        settings: &TranscriptionSettings,
        progress_callback: Option<Box<dyn Fn(f32) + Send>>,
    ) -> Result<String> {
        log::debug!("audio_path: {:?}", audio_path.to_str());
        log::debug!("setting: {:?}", settings);

        // Get model path from config manager
        let config = ConfigManager::new()?;
        let model_path = config.get_model_path(&settings.model);

        let original_samples = Self::parse_wav_file(audio_path);
        let mut samples = vec![0.0f32; original_samples.len()];
        whisper_rs::convert_integer_to_float_audio(&original_samples, &mut samples)
            .expect("failed to convert samples");

        let ctx = WhisperContext::new_with_params(
            &model_path.to_string_lossy(),
            WhisperContextParameters::default(),
        )?;

        let mut state = ctx.create_state().expect("failed to create key");
        let mut params = FullParams::new(SamplingStrategy::default());

        // Set language from settings
        if let Some(language) = &settings.language {
            params.set_language(Some(language));
        }

        // Set progress callback if provided
        if let Some(callback) = progress_callback {
            params.set_progress_callback_safe(move |progress| {
                callback(progress as f32);
            });
        }

        let st = std::time::Instant::now();
        state
            .full(params, &samples)
            .expect("failed to convert samples");
        let et = std::time::Instant::now();

        let num_segments = state
            .full_n_segments()
            .expect("failed to get number of segments");

        // Collect segments instead of printing them
        let mut segments = Vec::new();
        for i in 0..num_segments {
            let segment = state
                .full_get_segment_text(i)
                .expect("failed to get segment");
            let start_timestamp = state
                .full_get_segment_t0(i)
                .expect("failed to get start timestamp");
            let end_timestamp = state
                .full_get_segment_t1(i)
                .expect("failed to get end timestamp");

            segments.push((start_timestamp, end_timestamp, segment));
        }
        println!("took {}ms", (et - st).as_millis());

        let result = match settings.output_format {
            OutputFormat::Txt => segments
                .into_iter()
                .map(|(_, _, text)| text)
                .collect::<Vec<_>>()
                .join("\n"),
            OutputFormat::Srt => {
                let mut srt_content = String::new();
                for (index, (start, end, text)) in segments.iter().enumerate() {
                    let start_ms = start * 10;
                    let end_ms = end * 10;

                    let start_hours = start_ms / 3600000;
                    let start_minutes = (start_ms % 3600000) / 60000;
                    let start_seconds = (start_ms % 60000) / 1000;
                    let start_millis = start_ms % 1000;

                    let end_hours = end_ms / 3600000;
                    let end_minutes = (end_ms % 3600000) / 60000;
                    let end_seconds = (end_ms % 60000) / 1000;
                    let end_millis = end_ms % 1000;

                    srt_content.push_str(&format!(
                        "{}\n{:02}:{:02}:{:02},{:03} --> {:02}:{:02}:{:02},{:03}\n{}\n\n",
                        index + 1,
                        start_hours,
                        start_minutes,
                        start_seconds,
                        start_millis,
                        end_hours,
                        end_minutes,
                        end_seconds,
                        end_millis,
                        text
                    ));
                }
                srt_content
            }
            OutputFormat::Vtt => {
                let mut vtt_content = String::from("WEBVTT\n\n");
                for (start, end, text) in segments.iter() {
                    let start_ms = start * 10;
                    let end_ms = end * 10;

                    let start_hours = start_ms / 3600000;
                    let start_minutes = (start_ms % 3600000) / 60000;
                    let start_seconds = (start_ms % 60000) / 1000;
                    let start_millis = start_ms % 1000;

                    let end_hours = end_ms / 3600000;
                    let end_minutes = (end_ms % 3600000) / 60000;
                    let end_seconds = (end_ms % 60000) / 1000;
                    let end_millis = end_ms % 1000;

                    vtt_content.push_str(&format!(
                        "{:02}:{:02}:{:02}.{:03} --> {:02}:{:02}:{:02}.{:03}\n{}\n\n",
                        start_hours,
                        start_minutes,
                        start_seconds,
                        start_millis,
                        end_hours,
                        end_minutes,
                        end_seconds,
                        end_millis,
                        text
                    ));
                }
                vtt_content
            }
            OutputFormat::Json => {
                let json_segments: Vec<serde_json::Value> = segments
                    .into_iter()
                    .enumerate()
                    .map(|(index, (start, end, text))| {
                        serde_json::json!({
                            "id": index,
                            "start": start as f64 / 100.0, // Convert centiseconds to seconds
                            "end": end as f64 / 100.0,
                            "text": text
                        })
                    })
                    .collect();

                let json_data = serde_json::json!({
                    "segments": json_segments
                });
                serde_json::to_string_pretty(&json_data)?
            }
        };

        Ok(result)
    }
}
