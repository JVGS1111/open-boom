//! Spatial stereo widener and frequency-selective crossfeed processor.
//!
//! Decomposes stereo audio into Mid/Side components, widens upper stereo soundstage
//! while preserving center dialogue/vocal anchoring, centers sub-bass (< 200 Hz) to prevent
//! phase fatigue, and guarantees bit-identical passthrough for monaural signals
//! (FR19, AD-4, NFR1, NFR2).

use crate::types::INTERNAL_SAMPLE_RATE_F32;

/// Spatial stereo widener and crossfeed module.
pub struct SpatialWidenerModule {
    pub enabled: bool,
    target_width: f32,
    current_width: f32,
    sample_rate: f32,
    // 1-pole low-pass filter state for bass centering (< 200 Hz)
    bass_filter_state: f32,
    alpha_bass: f32,
}

impl Default for SpatialWidenerModule {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialWidenerModule {
    /// Creates a new Spatial stereo widener module.
    pub fn new() -> Self {
        let mut module = Self {
            enabled: false,
            target_width: 0.6,
            current_width: 0.0,
            sample_rate: INTERNAL_SAMPLE_RATE_F32,
            bass_filter_state: 0.0,
            alpha_bass: 0.0,
        };
        module.update_coefficients();
        module
    }

    /// Sets the operational sample rate and updates filter coefficients.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = if sample_rate <= 0.0 {
            INTERNAL_SAMPLE_RATE_F32
        } else {
            sample_rate
        };
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        // 1-pole low-pass cutoff at ~200 Hz for sub-bass centering
        let cutoff_hz = 200.0;
        let omega = 2.0 * std::f32::consts::PI * cutoff_hz / self.sample_rate;
        self.alpha_bass = (-omega).exp();
    }

    /// Sets the target stereo width (0.0 to 1.0).
    #[inline]
    pub fn set_amount(&mut self, width: f32) {
        let safe = if width.is_nan() || width.is_infinite() {
            0.0
        } else {
            width.clamp(0.0, 1.0)
        };
        self.target_width = safe;
    }

    /// Alias for `set_amount` to maintain compatibility with width terminology.
    #[inline]
    pub fn set_width(&mut self, width: f32) {
        self.set_amount(width);
    }

    /// Returns the current target width.
    #[inline]
    pub fn width(&self) -> f32 {
        self.target_width
    }

    /// Processes a single stereo frame (left, right) with Mid/Side expansion and bass centering.
    #[inline]
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled && self.current_width <= 1e-4 {
            return (left, right);
        }

        // Smooth parameter interpolation towards target (~4ms ramp)
        let target = if self.enabled { self.target_width } else { 0.0 };
        const SMOOTH_STEP: f32 = 0.005;
        if (self.current_width - target).abs() < SMOOTH_STEP {
            self.current_width = target;
        } else if self.current_width < target {
            self.current_width += SMOOTH_STEP;
        } else {
            self.current_width -= SMOOTH_STEP;
        }

        if self.current_width <= 1e-4 {
            return (left, right);
        }

        let l_in = if left.is_finite() { left } else { 0.0 };
        let r_in = if right.is_finite() { right } else { 0.0 };

        // 1. Mid/Side decomposition
        // Mid (center mono information) and Side (stereo difference)
        let mid = 0.5 * (l_in + r_in);
        let side = 0.5 * (l_in - r_in);

        // Mathematical invariant: If pure mono (L == R), side is strictly 0.0
        if side.abs() < 1e-8 {
            return (l_in, r_in);
        }

        // 2. Frequency-selective crossfeed: isolate low frequencies in Side channel (< 200 Hz)
        self.bass_filter_state = (1.0 - self.alpha_bass) * side + self.alpha_bass * self.bass_filter_state;
        let side_low = self.bass_filter_state;
        let side_high = side - side_low;

        // 3. Selective expansion:
        // - Sub-bass side is kept tight/centered (attenuate low-frequency side slightly with width)
        // - Upper frequencies side is boosted up to +150% (multiplier 1.0 to 2.5)
        let low_factor = 1.0 - (0.35 * self.current_width);
        let high_factor = 1.0 + (1.50 * self.current_width);

        let side_expanded = (side_low * low_factor) + (side_high * high_factor);

        // 4. Reconstruct stereo Left and Right channels:
        // L = Mid + Side_expanded
        // R = Mid - Side_expanded
        let out_l = mid + side_expanded;
        let out_r = mid - side_expanded;

        (out_l, out_r)
    }

    /// Flushes all internal filter states to 0.0.
    pub fn reset(&mut self) {
        self.bass_filter_state = 0.0;
        self.current_width = 0.0;
    }
}
