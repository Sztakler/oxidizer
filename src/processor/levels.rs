#[derive(Debug, Clone, Copy)]
/// Represents the intensity of the "oxidation" (low-pass) filter effect.
pub enum OxidationLevel {
    Clear,   // Warm and clean.
    Deep,    // Deep and mellow tone. Significantly reduces high frequencies.
    Muffled, // Extreme Low Pass ("it's all about that bass, no treble"). Very dark and bass-heavy.
}
impl OxidationLevel {
    /// Returns the filter coefficient (alpha) for the One-Pole Low Pass algorithm.
    /// Lower values result in a lower cutoff frequency.
    pub fn alpha(&self) -> f32 {
        match self {
            OxidationLevel::Clear => 0.1,
            OxidationLevel::Deep => 0.02,
            OxidationLevel::Muffled => 0.005,
        }
    }

    /// Attempts to parse a string into an `OxidationLevel`.
    ///
    /// # Errors
    /// Returns an error string if the input does not match any known level.
    pub fn try_from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "clear" => Ok(OxidationLevel::Clear),
            "muffled" => Ok(OxidationLevel::Muffled),
            "deep" => Ok(OxidationLevel::Deep),
            _ => Err(format!("Unknown oxidation level: {}", s)),
        }
    }
}
