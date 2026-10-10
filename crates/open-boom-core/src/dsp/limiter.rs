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
            // 0.2ms transparent attack ensuring lookahead convergence within 96 samples,
            // and ~50ms transparent release envelope
            attack_coeff: (-1.0f32 / (0.0002f32 * 48000.0f32)).exp(),
            release_coeff: (-1.0f32 / (0.050f32 * 48000.0f32)).exp(),
        }
    }

    /// Linear ceiling amplitude threshold (-0.2 dBFS ≈ 0.9772372).
    #[inline]
    pub fn ceiling_linear(&self) -> f32 {
        self.ceiling_linear
    }

    /// Current internal gain reduction envelope (0.0 to 1.0).
    #[inline]
    pub fn envelope(&self) -> f32 {
        self.envelope
    }

    /// Process a stereo sample frame [L, R] in-place with 2ms lookahead.
    ///
    /// Clamps output strictly at -0.2 dBFS (amplitude <= 0.9772372), guaranteeing
    /// 0% digital clipping distortion (< 1.0) and peak output <= -0.1 dBFS under any boost.
    #[inline]
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Sanitize incoming samples against NaN and Inf
        let in_l = if left.is_nan() || left.is_infinite() { 0.0 } else { left };
        let in_r = if right.is_nan() || right.is_infinite() { 0.0 } else { right };

        // Read delayed sample from the circular buffer (2ms = 96 samples lookahead)
        let delayed_l = self.delay_line_l[self.write_idx];
        let delayed_r = self.delay_line_r[self.write_idx];

        // Store current incoming sample
        self.delay_line_l[self.write_idx] = in_l;
        self.delay_line_r[self.write_idx] = in_r;
        self.write_idx = (self.write_idx + 1) % LIMITER_LOOKAHEAD_SAMPLES;

        // Detect true peak across the incoming samples and the entire lookahead buffer.
        // This gives 2ms advance warning to ramp down gain before hot transients exit.
        let mut peak = in_l.abs().max(in_r.abs()).max(delayed_l.abs()).max(delayed_r.abs());
        for i in 0..LIMITER_LOOKAHEAD_SAMPLES {
            peak = peak.max(self.delay_line_l[i].abs()).max(self.delay_line_r[i].abs());
        }

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
        self.envelope = self.envelope.clamp(0.0, 1.0);

        self.gain_reduction_db = if self.envelope <= 1e-5 {
            -100.0
        } else if self.envelope < 0.9999 {
            (20.0 * self.envelope.log10()).min(0.0)
        } else {
            0.0
        };

        if self.gain_reduction_db.is_nan() || self.gain_reduction_db.is_infinite() {
            self.gain_reduction_db = 0.0;
        }

        // Apply gain and enforce inviolable brickwall ceiling clamp: y[n] = (x[n - D] * g[n]).clamp(-C, C)
        let out_l = (delayed_l * self.envelope).clamp(-self.ceiling_linear, self.ceiling_linear);
        let out_r = (delayed_r * self.envelope).clamp(-self.ceiling_linear, self.ceiling_linear);

        let safe_l = if out_l.is_nan() || out_l.is_infinite() { 0.0 } else { out_l };
        let safe_r = if out_r.is_nan() || out_r.is_infinite() { 0.0 } else { out_r };

        (safe_l, safe_r)
    }

    /// Returns current gain reduction in dB (e.g. 0.0 or negative).
    #[inline]
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

    #[test]
    fn test_limiter_brickwall_ceiling_clamp_absolute() {
        let mut limiter = MasterLimiter::new();
        let ceiling = limiter.ceiling_linear();
        assert!((ceiling - 0.9772372).abs() < 1e-4);

        // Feed pathological impulse of amplitude 10.0 (+20 dBFS)
        limiter.process_stereo(10.0, 10.0);

        // For the next 200 samples, verify every single sample is strictly <= ceiling
        for _ in 0..200 {
            let (out_l, out_r) = limiter.process_stereo(0.0, 0.0);
            assert!(
                out_l.abs() <= ceiling + 1e-6,
                "Output sample {} exceeds ceiling {}",
                out_l,
                ceiling
            );
            assert!(
                out_r.abs() <= ceiling + 1e-6,
                "Output sample {} exceeds ceiling {}",
                out_r,
                ceiling
            );
            assert!(out_l.abs() < 1.0, "0% digital clipping guarantee violated");
            assert!(out_r.abs() < 1.0, "0% digital clipping guarantee violated");
        }
    }
}
