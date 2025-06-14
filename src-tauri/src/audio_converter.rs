extern crate ffmpeg_next as ffmpeg;

use anyhow::Result;
use std::path::Path;

pub struct AudioConverter {}

impl AudioConverter {
    pub fn new() -> Self {
        ffmpeg::init().unwrap();

        Self {}
    }

    pub fn convert_to_wav(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let mut input = ffmpeg::format::input(&Path::new(input_path))?;
        let mut output = ffmpeg::format::output(&Path::new(output_path))?;

        // Find the best audio stream
        let input_stream = input
            .streams()
            .best(ffmpeg::media::Type::Audio)
            .ok_or_else(|| anyhow::anyhow!("No audio stream found"))?;
        let stream_index = input_stream.index();

        // Get decoder for the input audio stream
        let context_decoder =
            ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())?;
        let mut decoder = context_decoder.decoder().audio()?;

        // Set up encoder with the desired parameters
        let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::PCM_S16LE)
            .ok_or_else(|| anyhow::anyhow!("PCM_S16LE codec not found"))?;

        let mut output_stream = output.add_stream(codec)?;
        let mut encoder = ffmpeg::codec::context::Context::new_with_codec(codec)
            .encoder()
            .audio()?;

        // Configure encoder parameters matching your ffmpeg command
        encoder.set_rate(16000); // -ar 16000
        encoder.set_channel_layout(ffmpeg::util::channel_layout::ChannelLayout::MONO); // -ac 1
        encoder.set_format(ffmpeg::format::Sample::I16(
            ffmpeg::format::sample::Type::Packed,
        )); // pcm_s16le
        encoder.set_bit_rate(256000);
        encoder.set_max_bit_rate(256000);

        // Set output stream time base
        output_stream.set_time_base(ffmpeg::Rational(1, 16000));

        // Open the encoder
        let mut encoder = encoder.open_as(codec)?;
        output_stream.set_parameters(&encoder);

        // Write header
        output.write_header()?;

        // Create resampler for format conversion
        let mut resampler = ffmpeg::software::resampling::context::Context::get(
            decoder.format(),
            decoder.channel_layout(),
            decoder.rate(),
            ffmpeg::format::Sample::I16(ffmpeg::format::sample::Type::Packed),
            ffmpeg::util::channel_layout::ChannelLayout::MONO,
            16000,
        )?;

        let mut frame_index = 0;

        // Process packets
        for (stream, packet) in input.packets() {
            if stream.index() == stream_index {
                decoder.send_packet(&packet)?;
                self.receive_and_process_frames(
                    &mut decoder,
                    &mut resampler,
                    &mut encoder,
                    &mut output,
                    &mut frame_index,
                )?;
            }
        }

        // Flush decoder
        decoder.send_eof()?;
        self.receive_and_process_frames(
            &mut decoder,
            &mut resampler,
            &mut encoder,
            &mut output,
            &mut frame_index,
        )?;

        // Write trailer
        output.write_trailer()?;

        Ok(())
    }

    fn receive_and_process_frames(
        &self,
        decoder: &mut ffmpeg::decoder::Audio,
        resampler: &mut ffmpeg::software::resampling::context::Context,
        encoder: &mut ffmpeg::encoder::Audio,
        output: &mut ffmpeg::format::context::Output,
        frame_index: &mut usize,
    ) -> Result<()> {
        let mut decoded = ffmpeg::util::frame::audio::Audio::empty();

        while decoder.receive_frame(&mut decoded).is_ok() {
            let mut resampled = ffmpeg::util::frame::audio::Audio::empty();
            resampler.run(&decoded, &mut resampled)?;

            resampled.set_pts(Some(*frame_index as i64));
            *frame_index += resampled.samples();

            // Send frame to encoder
            encoder.send_frame(&resampled)?;

            // Receive encoded packets
            let mut encoded_packet = ffmpeg::Packet::empty();
            while encoder.receive_packet(&mut encoded_packet).is_ok() {
                encoded_packet.set_stream(0);
                encoded_packet.rescale_ts(
                    ffmpeg::Rational(1, 16000),
                    output.stream(0).unwrap().time_base(),
                );
                // output.write_interleaved(&encoded_packet)?;
                encoded_packet.write_interleaved(output).unwrap();
            }
        }

        Ok(())
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
        let converter = AudioConverter::new();
        let input = Path::new("testing.mp4");
        let output = Path::new("testing.wav");
        if output.exists() {
            fs::remove_file(output).unwrap();
        }
        converter
            .convert_to_wav(input, output)
            .expect("Conversion failed");
        assert!(output.exists(), "Output WAV file was not created");
        // Optionally, check file size or header
        let metadata = fs::metadata(output).unwrap();
        assert!(metadata.len() > 44, "WAV file too small");
    }
}
