use anyhow::{anyhow, Result};
use rubato::Resampler;
use std::path::Path;
use std::process::Command;

pub struct AudioConverter;

impl AudioConverter {
    pub fn new() -> Result<Self> {
        // Check if ffmpeg is available
        if !Self::is_ffmpeg_available() {
            return Err(anyhow!("FFmpeg is not installed or not available in PATH"));
        }
        Ok(AudioConverter)
    }

    fn is_ffmpeg_available() -> bool {
        Command::new("ffmpeg").arg("-version").output().is_ok()
    }

    pub fn convert_to_wav(
        &self,
        input_path: &Path,
        output_path: &Path,
        progress_callback: Option<Box<dyn Fn(f32) + Send>>,
    ) -> Result<()> {
        let input_str = input_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid input path"))?;
        let output_str = output_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid output path"))?;

        // Use ffmpeg command line tool for conversion
        let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(input_str)
            .arg("-ar")
            .arg("16000") // 16kHz sample rate
            .arg("-ac")
            .arg("1") // Mono
            .arg("-c:a")
            .arg("pcm_s16le") // 16-bit PCM
            .arg("-y") // Overwrite output files
            .arg(output_str)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("FFmpeg conversion failed: {}", error_msg));
        }

        if let Some(callback) = progress_callback {
            callback(100.0);
        }

        Ok(())
    }

    pub fn get_audio_info(&self, path: &Path) -> Result<(f64, i32, i32)> {
        let input_str = path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;

        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_format")
            .arg("-show_streams")
            .arg(input_str)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get audio info"));
        }

        let json_str = String::from_utf8(output.stdout)?;
        let json: serde_json::Value = serde_json::from_str(&json_str)?;

        // Extract audio stream info
        if let Some(streams) = json["streams"].as_array() {
            for stream in streams {
                if stream["codec_type"] == "audio" {
                    let duration = stream["duration"]
                        .as_str()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let sample_rate = stream["sample_rate"]
                        .as_str()
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(44100);
                    let channels = stream["channels"].as_i64().unwrap_or(2) as i32;

                    return Ok((duration, sample_rate, channels));
                }
            }
        }

        Err(anyhow!("No audio stream found"))
    }

    pub fn is_audio_file(&self, path: &Path) -> bool {
        let audio_extensions = [
            "mp3", "wav", "flac", "m4a", "aac", "ogg", "wma", "opus", "mp4", "mkv", "avi", "mov",
            "wmv", "flv", "webm", "3gp",
        ];

        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return audio_extensions.contains(&ext_str.to_lowercase().as_str());
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_convert_to_wav() {
        let converter = AudioConverter::new().expect("Failed to create AudioConverter");
        let input = Path::new("testing.mp4");
        let output = Path::new("testing.wav");
        if output.exists() {
            fs::remove_file(output).unwrap();
        }
        converter
            .convert_to_wav(input, output, None)
            .expect("Conversion failed");
        assert!(output.exists(), "Output WAV file was not created");
        // Optionally, check file size or header
        let metadata = fs::metadata(output).unwrap();
        assert!(metadata.len() > 44, "WAV file too small");
    }
}
