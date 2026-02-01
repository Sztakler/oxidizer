use crate::error::{OxidizerError, Result};
use std::fs::File;
use symphonia::core::{
    audio::Signal,
    codecs::{CODEC_TYPE_NULL, DecoderOptions},
    errors::Error,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
};

/// Decodes an audio file from the given path into a flat vector of f32 samples.
///
/// This function supports any format recognized by Symphonia (MP3, WAV, FLAC, etc.).
/// It automatically converts mono signals to stereo by duplicating the channel,
/// resulting in an interleaved [L, R, L, R, ...] buffer.
pub fn load_audio(path: &std::path::Path) -> Result<Vec<f32>> {
    let src = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    // Probe the input file for format and metadata
    let mut probed = symphonia::default::get_probe()
        .format(
            &Default::default(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| OxidizerError::Symphonia(e.to_string()))?;

    let format = &mut probed.format;

    // Find the first track with a valid codec
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| OxidizerError::Decoding("No supported audio track found".to_string()))?;

    // Initialize the decoder for the selected track
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| OxidizerError::Decoding(format!("Couldn't create a decoder: {}", e)))?;

    let track_id = track.id;
    let mut samples: Vec<f32> = Vec::new();

    // Decode packets loop
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(symphonia::core::audio::AudioBufferRef::F32(buf)) => {
                let chan_l = buf.chan(0);
                // If the source is mono, use channel 0 for both L and R
                let chan_r = if buf.spec().channels.count() > 1 {
                    buf.chan(1)
                } else {
                    buf.chan(0)
                };

                // Interleave channels into the samples vector
                for i in 0..buf.frames() {
                    samples.push(chan_l[i]);
                    samples.push(chan_r[i]);
                }
            }
            // Currently ignores non-f32 buffers (e.g., S16, S24, S32)
            Ok(_) => {}
            Err(Error::IoError(_)) => break,
            Err(e) => {
                return Err(OxidizerError::Symphonia(e.to_string()));
            }
        }
    }
    Ok(samples)
}

/// Saves the provided f32 sample data into a 16-bit PCM WAV file.
///
/// The data is expected to be interleaved stereo. Samples are scaled from the
/// [-1.0, 1.0] range to the 16-bit integer range [-32768, 32767].
pub fn save_audio(path: &String, data: Vec<f32>, sample_rate: u32) -> Result<()> {
    // Define the WAV forma: Stereo, 16-bit PCM
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Scale and write each sample
    let mut writer =
        hound::WavWriter::create(path, spec).map_err(|e| OxidizerError::Encoding(e.to_string()))?;
    for sample in data {
        let scaled_sample = (sample * i16::MAX as f32) as i16;
        writer
            .write_sample(scaled_sample)
            .map_err(|e| OxidizerError::Decoding(e.to_string()))?;
    }

    // Finalize the WAV file to update the header with correct data length
    writer
        .finalize()
        .map_err(|e| OxidizerError::Encoding(e.to_string()))?;
    Ok(())
}
