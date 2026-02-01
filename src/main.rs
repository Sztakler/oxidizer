use clap::Parser;
use oxidizer::OxidationLevel;
use oxidizer::OxidizerError;
use oxidizer::error::Result;
use oxidizer::io;
use oxidizer::processor::Oxidizer;
use oxidizer::processor::noise;
use oxidizer::processor::noise::NoiseGenerator;
use std::f32;

/// Command-line arguments for the Oxidizer application.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "An audio transformer that makes everything sound like a Brownian noise"
)]
struct Args {
    /// Path to the input file (e.g., music.mp3). Supports multiple formats via Symphonia.
    #[arg(short, long)]
    input: String,

    /// Path where the processed .wav file will be saved.    #[arg(short, long, default_value = "output.wav")]
    output: String,

    /// The level of oxidation. Options: 'muffled', 'deep', 'clear'.
    #[arg(short, long, default_value = "deep")]
    level: String,

    /// The characteristic of the background hiss. 'brown' (bass-heavy) or 'white' (full-spectrum).
    #[arg(short, long, default_value = "brown")]
    noise: String,

    /// Scale of the noise and saturation effect. Typically 0.0 (subtle) to 1.0 (crushed).
    #[arg(short = 't', long, default_value_t = 0.05)]
    intensity: f32,

    /// Sample rate for the output WAV file. Should match the input for pitch consistency. Lower rates may result in a slowed down audio (pitch-shift).
    #[arg(short = 's', long, default_value_t = 44100)]
    sample_rate: u32,

    /// Number of filter iterations.
    /// Each pass doubles the filter slope (e.g., from 6dB/oct to 12dB/oct).
    #[arg(short, long, default_value_t = 1)]
    passes: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input_path = std::path::Path::new(&args.input);
    let input_samples: Vec<f32> = io::load_audio(input_path)?;

    // Dispatch processing based on selected noise generator
    let output_samples = match args.noise.as_str() {
        "white" => run_process(input_samples, noise::WhiteNoise::default(), &args)?,
        _ => run_process(input_samples, noise::BrownianNoise::default(), &args)?,
    };

    io::save_audio(&args.output, output_samples, args.sample_rate)?;

    Ok(())
}

// Orchestrates the oxidation pipeline using a generic noise generator.
fn run_process<N: NoiseGenerator>(samples: Vec<f32>, noise: N, args: &Args) -> Result<Vec<f32>> {
    let mut oxidizer = Oxidizer::new(noise);
    let level = OxidationLevel::try_from_str(&args.level).map_err(OxidizerError::InvalidValue)?;

    // Ownership-based pipeline (zero-copy)
    let processed = oxidizer
        .consume(samples)
        .process_multiple(level, args.passes)
        .apply_noise_texture(args.intensity)
        .normalize()
        .collect_samples();

    Ok(processed)
}
