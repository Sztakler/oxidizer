use clap::Parser;
use oxidizer::OxidationLevel;
use oxidizer::OxidizerError;
use oxidizer::error::Result;
use oxidizer::io;
use oxidizer::processor::Oxidizer;
use oxidizer::processor::noise;
use oxidizer::processor::noise::NoiseGenerator;
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

    /// The level of oxidation (muffled, deep, clear)
    #[arg(short, long, default_value = "deep")]
    level: String,

    /// The type of noise texture (brown, white)
    #[arg(short, long, default_value = "brown")]
    noise: String,

    /// Intensity of the effect (0.0 to 1.0)
    #[arg(short = 't', long, default_value_t = 0.05)]
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
    let mut samples: Vec<f32> = io::load_audio(input_path)?;

    samples = match args.noise.as_str() {
        "white" => run_process(samples, noise::WhiteNoise::default(), &args)?,
        _ => run_process(samples, noise::BrownianNoise::default(), &args)?,
    };

    io::save_audio(&args.output, samples, args.sample_rate)?;

    Ok(())
}

fn run_process<N: NoiseGenerator>(samples: Vec<f32>, noise: N, args: &Args) -> Result<Vec<f32>> {
    let mut oxidizer = Oxidizer::new(noise);
    let level = OxidationLevel::try_from_str(&args.level).map_err(OxidizerError::InvalidValue)?;

    let processed = oxidizer
        .consume(samples)
        .process_multiple(level, args.passes)
        .apply_noise_texture(args.intensity)
        .normalize()
        .collect_samples();

    Ok(processed)
}
