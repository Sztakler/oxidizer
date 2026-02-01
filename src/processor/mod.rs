pub mod levels;
pub mod noise;

pub use levels::OxidationLevel;

use crate::processor::noise::NoiseGenerator;

/// The main engine responsible for "oxidizing" (low-pass filtering)
/// and applying noise textures to audio buffers.
pub struct Oxidizer<N: NoiseGenerator> {
    noise_generator: N,
    last_l: f32,
    last_r: f32,
    buffer: Vec<f32>,
}

impl<N: NoiseGenerator + Default> Default for Oxidizer<N> {
    fn default() -> Self {
        Self::new(N::default())
    }
}

impl<N: NoiseGenerator> Oxidizer<N> {
    /// Creates a new Oxidizer instance with a specific noise generator.
    pub fn new(noise_generator: N) -> Self {
        Self {
            noise_generator,
            last_l: 0.0,
            last_r: 0.0,
            buffer: Vec::new(),
        }
    }

    /// Takes ownership of the input sample vector.
    /// This is a zero-copy operation that reuses the allocated memory of the input vector.
    pub fn consume(&mut self, samples: Vec<f32>) -> &mut Self {
        self.buffer = samples;

        self
    }

    /// Processes the audio buffer using a One-Pole Low Pass Filter.
    /// The `alpha` value from the `OxidationLevel` determines the filter's cutoff frequency.
    pub fn process(&mut self, level: OxidationLevel) -> &mut Self {
        let alpha = level.alpha();

        for i in (0..self.buffer.len()).step_by(2) {
            self.last_l = self.last_l + alpha * (self.buffer[i] - self.last_l);
            self.buffer[i] = self.last_l;

            self.last_r = self.last_r + alpha * (self.buffer[i + 1] - self.last_r);
            self.buffer[i + 1] = self.last_r;
        }

        self
    }

    /// Normalizes the audio buffer so the highest peak reaches 0.95 (approx. -0.5 dBFS).
    /// This prevents digital clipping after noise and filter processing.
    pub fn normalize(&mut self) -> &mut Self {
        let max_peak = self.buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);

        if max_peak > 0.0 {
            let scale_factor = 0.95 / max_peak;
            for sample in &mut self.buffer {
                *sample *= scale_factor;
            }
        }

        self
    }

    // Extracts the processes samples from the engine, leaving the internal buffer empty.
    pub fn collect_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.buffer)
    }

    /// Applies a noise texture to the audio signal. Intensity is mapped logarithmically.
    /// The result is processed though a `tanh()` function for soft-clipping and saturation.
    pub fn apply_noise_texture(&mut self, intensity: f32) -> &mut Self {
        let perceived_intensity = (10.0f32.powf(intensity) - 1.0) / 9.0;

        for i in (0..self.buffer.len()).step_by(2) {
            let noise_l = self.noise_generator.next_sample();
            let noise_r = self.noise_generator.next_sample();

            self.buffer[i] = (self.buffer[i] + noise_l * perceived_intensity).tanh();
            if i + 1 < self.buffer.len() {
                self.buffer[i + 1] = (self.buffer[i + 1] + noise_r * perceived_intensity).tanh();
            }
        }

        self
    }

    /// Executes the filtration process multiple times.
    /// Each pass further muffles the high frequencies and deepens the "oxidation" effect.
    pub fn process_multiple(&mut self, level: OxidationLevel, passes: u32) -> &mut Self {
        for _ in 0..passes {
            self.process(level);
        }

        self
    }
}
