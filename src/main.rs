use clap::Parser;
use oxidizer::OxidationAlgorithm;
use oxidizer::error::Result;
use oxidizer::io;
use oxidizer::processor::Oxidizer;
use oxidizer::processor::noise;
use std::f32;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "An audio transformer that makes everything sound like a Brownian noise"
)]
struct Args {
    /// Path to the input file (e.g., music.mp3)
    #[arg(short, long)]
    input: String,
    /// Path to the output file (e.g., output.wav)
    #[arg(short, long, default_value = "output.wav")]
    output: String,

    /// The oxidation algorithm to use
    #[arg(short, long, default_value = "brown")]
    algorithm: String,

    /// Intensity of the effect (0.0 to 1.0)
    #[arg(short = 'n', long, default_value_t = 0.05)]
    intensity: f32,

    /// Sample rate of the audio (e.g. 44100 Hz)
    #[arg(short = 's', long, default_value_t = 44100)]
    sample_rate: u32,

    /// Apply an extra pass of the filter for more "rust"
    #[arg(short, long, default_value_t = 1)]
    passes: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input_path = std::path::Path::new(&args.input);
    let input_samples: Vec<f32> = io::load_audio(input_path)?;

    let algorithm = match args.algorithm.to_lowercase().as_str() {
        "light" => OxidationAlgorithm::Light,
        "heavy" => OxidationAlgorithm::Heavy,
        _ => OxidationAlgorithm::Brown,
    };

    let mut oxidizer = Oxidizer::new(noise::WhiteNoise::default());
    oxidizer.consume(input_samples);

    for _ in 0..args.passes {
        oxidizer.process(algorithm);
    }

    let output_samples = oxidizer
        .apply_noise_texture(args.intensity)
        .normalize()
        .collect_samples();

    io::save_audio(&args.output, output_samples, args.sample_rate)?;
    Ok(())
}
