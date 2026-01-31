use rand::Rng;
use std::{f32, fs::File};
use symphonia::{
    self,
    core::{
        audio::Signal,
        codecs::{CODEC_TYPE_MP3, DecoderOptions},
        errors::Error,
        formats::FormatOptions,
        io::MediaSourceStream,
        meta::MetadataOptions,
    },
};

pub enum OxidizerAlgorithm {
    Light, // Pink Noise (warm and clean)
    Brown, // Brown Noise (deep and mellow)
    Heavy, // Extreme Low Pass (it's all about that bass, no treble)
}

struct Oxidizer {
    last_sample: f32,
    buffer: Vec<f32>,
}

impl Oxidizer {
    fn new() -> Self {
        Self {
            last_sample: 0.0,
            buffer: Vec::new(),
        }
    }

    fn consume(&mut self, samples: Vec<f32>) -> &mut Self {
        for sample in samples {
            self.buffer.push(sample);
        }

        self
    }

    fn process(&mut self, algorithm: OxidizerAlgorithm) -> &mut Self {
        let alpha = match algorithm {
            OxidizerAlgorithm::Light => 0.1,
            OxidizerAlgorithm::Brown => 0.02,
            OxidizerAlgorithm::Heavy => 0.005,
        };

        for sample in &mut self.buffer {
            let output = self.last_sample + alpha * (*sample - self.last_sample);
            self.last_sample = output;
            *sample = output
        }

        self
    }

    fn normalize(&mut self) -> &mut Self {
        let max_peak = self.buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);

        if max_peak > 0.0 {
            let scale_factor = 0.95 / max_peak;
            for sample in &mut self.buffer {
                *sample *= scale_factor;
            }
        }

        self
    }

    fn collect_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.buffer)
    }

    // Voss-McCartney Filter Bank algorithm
    fn apply_brownian_texture(&mut self, intensity: f32) -> &mut Self {
        let mut rng = rand::rng();
        let mut brown_noise_state: f32 = 0.0;
        let step_size = 0.1;
        let damping = 0.98;
        let perceived_intensity = (10.0f32.powf(intensity) - 1.0) / 9.0;

        for sample in &mut self.buffer {
            // Generate Brown Noise step (random walk)
            let white_step: f32 = rng.random_range(-1.0..1.0);
            // Smooth step
            brown_noise_state =
                (brown_noise_state * damping + white_step * step_size).clamp(-1.0, 1.0);
            let current_value = *sample;
            let noise_mask = brown_noise_state * perceived_intensity;
            let combined = current_value + noise_mask;
            *sample = combined.tanh();
        }

        self
    }
}

fn load_mp3(path: &str) -> Vec<f32> {
    println!("Loading file: {}", path);

    let src = File::open(path).expect("Cannot open file");
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    println!("Probing...");
    let mut probed = symphonia::default::get_probe()
        .format(
            &Default::default(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .expect("Unknown file format");

    let format = &mut probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec == CODEC_TYPE_MP3)
        .expect("Couldn't find MP3 track");

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .expect("Couldn't create decoder");

    let track_id = track.id;
    let mut samples: Vec<f32> = Vec::new();

    println!("Decoding MP3 file...");
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(symphonia::core::audio::AudioBufferRef::F32(buf)) => {
                samples.extend_from_slice(buf.chan(0));
            }
            Ok(_) => {}
            Err(Error::IoError(_)) => break,
            Err(e) => panic!("Decoding error: {:?}", e),
        }
    }
    samples
}

fn main() {
    let input_samples: Vec<f32> = load_mp3("the-smiths.mp3");

    println!("Oxidizing samples...");
    let output_samples = Oxidizer::new()
        .consume(input_samples)
        .process(OxidizerAlgorithm::Brown)
        .apply_brownian_texture(0.1)
        .normalize()
        .collect_samples();

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let output_path = "brownized.wav";
    println!("Writing to {}...", output_path);
    let mut writer = hound::WavWriter::create(output_path, spec).unwrap();
    output_samples.into_iter().for_each(|sample| {
        let scaled_sample = (sample * i16::MAX as f32) as i16;
        writer.write_sample(scaled_sample).unwrap()
    });

    writer.finalize().unwrap();
}
