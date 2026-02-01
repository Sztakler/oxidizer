use rand::Rng;

pub enum Algorithm {
    Light, // Pink Noise (warm and clean)
    Brown, // Brown Noise (deep and mellow)
    Heavy, // Extreme Low Pass (it's all about that bass, no treble)
}

pub struct Oxidizer {
    last_l: f32,
    last_r: f32,
    brown_state_l: f32,
    brown_state_r: f32,
    buffer: Vec<f32>,
}

impl Default for Oxidizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Oxidizer {
    pub fn new() -> Self {
        Self {
            last_l: 0.0,
            last_r: 0.0,
            brown_state_l: 0.0,
            brown_state_r: 0.0,
            buffer: Vec::new(),
        }
    }

    pub fn consume(&mut self, samples: Vec<f32>) -> &mut Self {
        for sample in samples {
            self.buffer.push(sample);
        }

        self
    }

    pub fn process(&mut self, algorithm: Algorithm) -> &mut Self {
        let alpha = match algorithm {
            Algorithm::Light => 0.1,
            Algorithm::Brown => 0.02,
            Algorithm::Heavy => 0.005,
        };

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

    // Leaky Random Walk
    pub fn apply_brownian_texture(&mut self, intensity: f32) -> &mut Self {
        let mut rng = rand::rng();
        let step_size = 0.1;
        let damping = 0.98;
        let perceived_intensity = (10.0f32.powf(intensity) - 1.0) / 9.0;

        for i in (0..self.buffer.len()).step_by(2) {
            self.brown_state_l = (self.brown_state_l * damping
                + (rng.random_range(-1.0..1.0) * step_size))
                .clamp(-1.0, 1.0);
            self.brown_state_r = (self.brown_state_r * damping
                + (rng.random_range(-1.0..1.0) * step_size))
                .clamp(-1.0, 1.0);

            self.buffer[i] = (self.buffer[i] + self.brown_state_l * perceived_intensity).tanh();
            self.buffer[i + 1] =
                (self.buffer[i + 1] + self.brown_state_r * perceived_intensity).tanh();
        }

        self
    }
}
