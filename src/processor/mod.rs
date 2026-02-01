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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processor::noise::WhiteNoise;

    #[test]
    fn test_consume_and_collect() {
        let mut oxidizer = Oxidizer::new(WhiteNoise::default());
        let data = vec![0.5, -0.5, 0.2];
        let output = oxidizer.consume(data.clone()).collect_samples();
        assert_eq!(data, output);
    }

    #[test]
    fn test_normalization_bounds() {
        let mut oxidizer = Oxidizer::new(WhiteNoise::default());
        // Sample way over limit 1.0
        oxidizer.consume(vec![5.0, -2.0]);
        oxidizer.normalize();
        let samples = oxidizer.collect_samples();

        let max_peak = samples.iter().map(|s| s.abs()).fold(0.0, f32::max);
        // Should be cut off to exactly 0.95
        assert!((max_peak - 0.95).abs() < 1e-6);
    }

    #[test]
    fn test_filter_smoothing() {
        let mut oxidizer = Oxidizer::new(WhiteNoise::default());
        // Jumpy signal: 0, 1, 0 ,1, ...
        let input = vec![0.0, 1.0, 0.0, 1.0];
        oxidizer.consume(input);
        oxidizer.process(OxidationLevel::Muffled);
        let output = oxidizer.collect_samples();

        // When using `Muffled` value (alpha 0.005), signal shouldn't
        // be able to jump to 1.0. It should be very smooth.
        assert!(output[1] < 0.1);
    }

    #[test]
    fn test_multiple_passes_attenuation() {
        // Generate clean zig-zag signal (square wave of Nyquist frequency)
        let input: Vec<f32> = (0..2000)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect();

        // Highest level
        let level = OxidationLevel::Muffled;

        // First pass
        let mut ox1 = Oxidizer::new(WhiteNoise::default());
        let res1 = ox1
            .consume(input.clone())
            .process_multiple(level, 1)
            .collect_samples();

        // Ten continuous passes
        let mut ox2 = Oxidizer::new(WhiteNoise::default());
        let res2 = ox2
            .consume(input)
            .process_multiple(level, 10)
            .collect_samples();

        // Calculate RMS (Root Mean Square) = signal's energy
        let rms1 = (res1.iter().map(|s| s * s).sum::<f32>() / res1.len() as f32).sqrt();
        let rms2 = (res2.iter().map(|s| s * s).sum::<f32>() / res2.len() as f32).sqrt();

        println!("RMS 1 pass: {}, RMS 10 passes: {}", rms1, rms2);

        // RMS after the single pass should be higher than after 10 passes
        assert!(
            rms2 < rms1,
            "Energy after 10 passes ({}) should be lower than after 1 pass ({})",
            rms2,
            rms1
        );
    }

    #[test]
    fn test_saturation_limits() {
        let mut oxidizer = Oxidizer::new(WhiteNoise::default());
        // Very high intensity of both the noise and the signal
        oxidizer.consume(vec![2.0, -2.0]);
        oxidizer.apply_noise_texture(1.0);
        let output = oxidizer.collect_samples();

        // tanh should never exceed +-1.0
        for sample in output {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn test_stereo_noise_decorrelation() {
        let mut oxidizer = Oxidizer::new(WhiteNoise::default());
        // Noise over silence
        let input = vec![0.0; 1000];
        let output = oxidizer
            .consume(input)
            .apply_noise_texture(1.0)
            .collect_samples();

        // Check L and R samples
        let mut identical_samples = 0;
        for i in (0..output.len()).step_by(2) {
            if (output[i] - output[i + 1]).abs() < 1e-6 {
                identical_samples += 1;
            }
        }

        // If the noise is stereo, L and R samples should be identical
        assert!(
            identical_samples < 100,
            "Left and Right channels should have decorrelated noise for spatial width"
        );
    }
}
