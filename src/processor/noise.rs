use rand::Rng;
use rand::rngs::ThreadRng;

pub trait NoiseGenerator {
    fn next_sample(&mut self) -> f32;
}

/// Simple White Noise (radio static)
pub struct WhiteNoise {
    rng: ThreadRng,
}

impl Default for WhiteNoise {
    fn default() -> Self {
        Self { rng: rand::rng() }
    }
}

impl NoiseGenerator for WhiteNoise {
    fn next_sample(&mut self) -> f32 {
        self.rng.random_range(-1.0..1.0)
    }
}

// Leaky Random Walk (Brown Noise)
pub struct BrownianNoise {
    state: f32,
    damping: f32,
    step: f32,
    rng: ThreadRng,
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
            rng: rand::rng(),
        }
    }
}

impl NoiseGenerator for BrownianNoise {
    fn next_sample(&mut self) -> f32 {
        let white = self.rng.random_range(-1.0..1.0);
        self.state = (self.state * self.damping + (white * self.step)).clamp(-1.0, 1.0);
        self.state
    }
}
