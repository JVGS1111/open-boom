//! Linear/polyphase sample rate converter normalizing streams to 48 kHz.

use open_boom_core::types::INTERNAL_SAMPLE_RATE;

/// Resampler converting arbitrary source sample rates into internal 48 kHz float32 streams.
pub struct Resampler {
    source_rate: u32,
    target_rate: u32,
    ratio: f64,
    phase: f64,
    last_samples: [f32; 8],
}

impl Resampler {
    pub fn new(source_rate: u32) -> Self {
        let target_rate = INTERNAL_SAMPLE_RATE;
        let ratio = source_rate as f64 / target_rate as f64;
        Self {
            source_rate,
            target_rate,
            ratio,
            phase: 0.0,
            last_samples: [0.0; 8],
        }
    }

    pub fn source_rate(&self) -> u32 {
        self.source_rate
    }

    pub fn target_rate(&self) -> u32 {
        self.target_rate
    }

    pub fn ratio(&self) -> f64 {
        self.ratio
    }

    pub fn is_identity(&self) -> bool {
        self.source_rate == self.target_rate
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.last_samples.fill(0.0);
    }
}
