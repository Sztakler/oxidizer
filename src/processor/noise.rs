use rand::Rng;

pub trait NoiseGenerator {
    fn next_sample(&mut self) -> f32;
}

/// Simple White Noise (radio static)
pub struct WhiteNoise;

impl Default for WhiteNoise {
    fn default() -> Self {
        Self
    }
}

impl NoiseGenerator for WhiteNoise {
    fn next_sample(&mut self) -> f32 {
        let mut rng = rand::rng();
        rng.random_range(-1.0..1.0)
    }
}

// Leaky Random Walk (Brown Noise)
pub struct BrownianNoise {
    state: f32,
    damping: f32,
    step: f32,
}

impl Default for BrownianNoise {
    fn default() -> Self {
        Self::new(0.98, 0.1)
    }
}

impl BrownianNoise {
    pub fn new(damping: f32, step: f32) -> Self {
        Self {
            state: 0.0,
            damping,
            step,
        }
    }
}

impl NoiseGenerator for BrownianNoise {
    fn next_sample(&mut self) -> f32 {
        let mut rng = rand::rng();
        let white = rng.random_range(-1.0..1.0);
        self.state = (self.state * self.damping + (white * self.step)).clamp(-1.0, 1.0);
        self.state
    }
}
