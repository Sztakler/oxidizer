use rand::Rng;
use rand::rngs::ThreadRng;

/// Defines the behaviour for audio noise generators.
pub trait NoiseGenerator {
    /// Generates the next audio sample, typically in the range [-1.0, 1.0].
    fn next_sample(&mut self) -> f32;
}

/// Simple White Noise generator.
///
/// Produces a signal with equal intensity at all frequencies,
/// sounding like a radio static or falling rain.
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

/// Brownian Noise generator (aka Brown Noise or Red Noise).
///
/// Uses a "Leaky Random Walk" algorithm. It has much higher energy at lower
/// frequencies, resulting in a much deeper and warmer sound compared to the
/// white noise.
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
    // Creates a new BrownianNoise generator with custom characteristics.
    //
    // * `damping`: How much of the previous state is retained (0.0 to 1.0).
    // * `step`: The maximum change applied by the random walk in each sample.
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
        // Apply leaky integration: new_state = old_state * damping + random_step
        self.state = (self.state * self.damping + (white * self.step)).clamp(-1.0, 1.0);
        self.state
    }
}
