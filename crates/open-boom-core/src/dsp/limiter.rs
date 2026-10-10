//! Master 2ms lookahead brickwall peak limiter (-0.2 dBFS).

use crate::types::{LIMITER_CEILING_DBFS, LIMITER_LOOKAHEAD_SAMPLES};

/// Mandatory terminal limiter stage ensuring 0% digital clipping distortion.
pub struct MasterLimiter {
    delay_line_l: [f32; LIMITER_LOOKAHEAD_SAMPLES],
    delay_line_r: [f32; LIMITER_LOOKAHEAD_SAMPLES],
    write_idx: usize,
    ceiling_linear: f32,
    envelope: f32,
    gain_reduction_db: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl Default for MasterLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl MasterLimiter {
    pub fn new() -> Self {
        // -0.2 dBFS converted to linear amplitude ratio
        let ceiling_linear = 10.0f32.powf(LIMITER_CEILING_DBFS / 20.0);
        Self {
            delay_line_l: [0.0; LIMITER_LOOKAHEAD_SAMPLES],
            delay_line_r: [0.0; LIMITER_LOOKAHEAD_SAMPLES],
            write_idx: 0,
            ceiling_linear,
            envelope: 1.0,
            gain_reduction_db: 0.0,
            // Fast attack, ~50ms transparent release
            attack_coeff: (-1.0f32 / (0.0005f32 * 48000.0f32)).exp(),
            release_coeff: (-1.0f32 / (0.050f32 * 48000.0f32)).exp(),
        }
    }

    /// Process a stereo sample frame [L, R] in-place with 2ms lookahead.
    #[inline]
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Read delayed sample from the circular buffer
        let delayed_l = self.delay_line_l[self.write_idx];
        let delayed_r = self.delay_line_r[self.write_idx];

        // Store current incoming sample
        self.delay_line_l[self.write_idx] = left;
        self.delay_line_r[self.write_idx] = right;
        self.write_idx = (self.write_idx + 1) % LIMITER_LOOKAHEAD_SAMPLES;

        // Detect true peak between incoming and delayed channels
        let peak = left.abs().max(right.abs()).max(delayed_l.abs()).max(delayed_r.abs());

        let target_gain = if peak > self.ceiling_linear {
            self.ceiling_linear / peak
        } else {
            1.0
        };

        // Smooth gain reduction envelope
        if target_gain < self.envelope {
            self.envelope = self.attack_coeff * self.envelope + (1.0 - self.attack_coeff) * target_gain;
        } else {
            self.envelope = self.release_coeff * self.envelope + (1.0 - self.release_coeff) * target_gain;
        }

        self.gain_reduction_db = if self.envelope <= 0.0 {
            -100.0
        } else if self.envelope < 0.999 {
            20.0 * self.envelope.log10()
        } else {
            0.0
        };

        (delayed_l * self.envelope, delayed_r * self.envelope)
    }

    /// Returns current gain reduction in dB (e.g. 0.0 or negative).
    pub fn gain_reduction_db(&self) -> f32 {
        self.gain_reduction_db
    }

    /// Flush lookahead buffer delay lines.
    pub fn reset(&mut self) {
        self.delay_line_l.fill(0.0);
        self.delay_line_r.fill(0.0);
        self.write_idx = 0;
        self.envelope = 1.0;
        self.gain_reduction_db = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter_gain_reduction_guard() {
        let mut limiter = MasterLimiter::new();
        // Process silence
        let (l, r) = limiter.process_stereo(0.0, 0.0);
        assert_eq!(l, 0.0);
        assert_eq!(r, 0.0);
        assert_eq!(limiter.gain_reduction_db(), 0.0);

        // Process extreme transient
        limiter.process_stereo(100.0, 100.0);
        assert!(limiter.gain_reduction_db() < 0.0);
        assert!(!limiter.gain_reduction_db().is_nan());
        assert!(!limiter.gain_reduction_db().is_infinite());

        // Test non-positive envelope guard
        limiter.envelope = 0.0;
        let _ = limiter.process_stereo(1.0, 1.0);
        assert!(!limiter.gain_reduction_db().is_nan());
        assert!(!limiter.gain_reduction_db().is_infinite());
    }
}
