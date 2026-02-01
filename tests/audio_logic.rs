use oxidizer::processor::noise::BrownianNoise;
use oxidizer::{OxidationLevel, Oxidizer};

#[test]
fn test_stereo_integrity() {
    let mut oxidizer = Oxidizer::new(BrownianNoise::default());
    // Simulate 1 second of stereo signal
    let input = vec![0.1; 44100 * 2];

    let output = oxidizer
        .consume(input)
        .process(OxidationLevel::Clear)
        .collect_samples();

    // Check for any lost channel (length should remain the same)
    assert_eq!(output.len(), 44100 * 2);
}

#[test]
fn test_zero_input_handling() {
    let mut oxidizer = Oxidizer::new(BrownianNoise::default());
    let input = vec![]; // Empty vector

    let output = oxidizer
        .consume(input)
        .process(OxidationLevel::Deep)
        .collect_samples();

    assert!(output.is_empty());
}

#[test]
fn test_conversion_edge_cases() {
    // Check behaviour at extreme values (NaN, Infinity)
    let mut oxidizer = Oxidizer::new(BrownianNoise::default());
    let input = vec![f32::NAN, f32::INFINITY, f32::NEG_INFINITY];

    let output = oxidizer.consume(input).normalize().collect_samples();

    // After normalizing and processing there shouldn't be any infinite values
    for sample in output {
        assert!(sample.is_finite());
    }
}
