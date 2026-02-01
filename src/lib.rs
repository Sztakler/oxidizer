//! # Oxidizer Library
//!
//! `oxidizer` is an audio processing engine for artistic sound degradation.
//! It utilizes Low Pass filters and noise generators to emulate warm and
//! "oxidized" sound.
//!
//! ## Core Workflow
//!
//! 1. **Consume**: Load raw PCM samples into the engine (zero-copy).
//! 2. **Process**: Apply low-pass filtration based on [`OxidationLevel`].
//! 3. **Texture**: Overlay generated noise and apply `tanh` saturation.
//! 4. **Normalize**: Ensure the output stays within safe digital bounds (-0.5 dBFS).
//! 5. **Collect**: Extract the processed buffer for playback or storage.
//!
//! ## Quick Start
//!
//! ```rust
//! use oxidizer::{Oxidizer, OxidationLevel};
//! use oxidizer::processor::noise::BrownianNoise;
//!
//! let samples = vec![0.0; 44100]; // Your audio data
//! let mut ox = Oxidizer::new(BrownianNoise::default());
//!
//! let processed = ox.consume(samples)
//!     .process(OxidationLevel::Deep)
//!     .apply_noise_texture(0.05)
//!     .normalize()
//!     .collect_samples();
//! ```
//!

pub mod error;
pub mod io;
pub mod processor;

pub use error::{OxidizerError, Result};
pub use processor::{OxidationLevel, Oxidizer};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
