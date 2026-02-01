#[derive(Debug, Clone, Copy)]
pub enum OxidationLevel {
    Clear,   // Warm and clean
    Deep,    // Deep and mellow
    Muffled, // Extreme Low Pass (it's all about that bass, no treble)
}
impl OxidationLevel {
    pub fn alpha(&self) -> f32 {
        match self {
            OxidationLevel::Clear => 0.1,
            OxidationLevel::Deep => 0.02,
            OxidationLevel::Muffled => 0.005,
        }
    }

    pub fn try_from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "clear" => Ok(OxidationLevel::Clear),
            "muffled" => Ok(OxidationLevel::Muffled),
            "deep" => Ok(OxidationLevel::Deep),
            _ => Err(format!("Unknown oxidation level: {}", s)),
        }
    }
}
