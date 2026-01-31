use std::fs::File;
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

struct Oxidizer {
    last_sample: f32,
    alpha: f32,
}

impl Oxidizer {
    fn process(&mut self, input: f32) -> f32 {
        let output = self.last_sample + self.alpha * (input - self.last_sample);
        self.last_sample = output;
        output
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

    let mut oxidizer = Oxidizer {
        last_sample: 0.0,
        alpha: 0.5,
    };

    println!("Oxidizing...");
    let output_samples: Vec<f32> = input_samples
        .into_iter()
        .map(|sample| oxidizer.process(sample))
        .collect();

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
