pub mod levels;
pub mod noise;

pub use levels::OxidationLevel;

use crate::processor::noise::NoiseGenerator;

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
    pub fn new(noise_generator: N) -> Self {
        Self {
            noise_generator,
            last_l: 0.0,
            last_r: 0.0,
            buffer: Vec::new(),
        }
    }

    pub fn consume(&mut self, samples: Vec<f32>) -> &mut Self {
        self.buffer = samples;

        self
    }

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

    pub fn collect_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.buffer)
    }

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

    pub fn process_multiple(&mut self, level: OxidationLevel, passes: u32) -> &mut Self {
        for _ in 0..passes {
            self.process(level);
        }

        self
    }
}
