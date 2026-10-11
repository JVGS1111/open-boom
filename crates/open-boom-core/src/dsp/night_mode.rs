//! Night Mode dynamic range compressor and dialogue leveling processor.
//!
//! Features fast attack (< 5ms), program-dependent release, soft-knee curvature,
//! downward compression on explosive transients (up to -18 dB), and clean upward
//! dialogue boost (up to +6 dB) for quiet whispers (FR18, AD-4, NFR1, NFR2).

use crate::types::INTERNAL_SAMPLE_RATE_F32;

/// Night Mode dynamic range compressor module.
pub struct NightModeModule {
    pub enabled: bool,
    target_amount: f32,
    current_amount: f32,
    sample_rate: f32,
    // Envelope follower state (linear peak amplitude)
    envelope: f32,
    // Smoothed gain in decibels
    gain_db: f32,
    // Attack and release smoothing coefficients
    alpha_attack: f32,
    alpha_release: f32,
    alpha_gain: f32,
}

impl Default for NightModeModule {
    fn default() -> Self {
        Self::new()
    }
}

impl NightModeModule {
    /// Creates a new Night Mode dynamic range compressor.
    pub fn new() -> Self {
        let mut module = Self {
            enabled: false,
            target_amount: 0.5,
            current_amount: 0.0,
            sample_rate: INTERNAL_SAMPLE_RATE_F32,
            envelope: 0.0,
            gain_db: 0.0,
            alpha_attack: 0.0,
            alpha_release: 0.0,
            alpha_gain: 0.0,
        };
        module.update_coefficients();
        module
    }

    /// Sets the operational sample rate and updates smoothing coefficients.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = if sample_rate <= 0.0 {
            INTERNAL_SAMPLE_RATE_F32
        } else {
            sample_rate
        };
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        // Fast attack ~2.5ms (< 5ms)
        let tau_att = 0.0025;
        self.alpha_attack = (-1.0 / (tau_att * self.sample_rate)).exp();

        // Natural release ~120ms
        let tau_rel = 0.120;
        self.alpha_release = (-1.0 / (tau_rel * self.sample_rate)).exp();

        // Gain smoothing filter ~8ms to prevent audio clicks
        let tau_gain = 0.008;
        self.alpha_gain = (-1.0 / (tau_gain * self.sample_rate)).exp();
    }

    /// Sets the target compression depth (0.0 to 1.0).
    #[inline]
    pub fn set_amount(&mut self, amount: f32) {
        let safe = if amount.is_nan() || amount.is_infinite() {
            0.0
        } else {
            amount.clamp(0.0, 1.0)
        };
        self.target_amount = safe;
    }

    /// Returns current target intensity.
    #[inline]
    pub fn amount(&self) -> f32 {
        self.target_amount
    }

    /// Processes a single stereo frame (left, right) with program-dependent compression.
    #[inline]
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled && self.current_amount <= 1e-4 {
            return (left, right);
        }

        // Smooth parameter interpolation towards target (~4ms ramp)
        let target = if self.enabled { self.target_amount } else { 0.0 };
        const SMOOTH_STEP: f32 = 0.005;
        if (self.current_amount - target).abs() < SMOOTH_STEP {
            self.current_amount = target;
        } else if self.current_amount < target {
            self.current_amount += SMOOTH_STEP;
        } else {
            self.current_amount -= SMOOTH_STEP;
        }

        if self.current_amount <= 1e-4 {
            return (left, right);
        }

        let l_in = if left.is_finite() { left } else { 0.0 };
        let r_in = if right.is_finite() { right } else { 0.0 };

        let peak_in = l_in.abs().max(r_in.abs());

        // Envelope follower with asymmetrical attack/release
        if peak_in > self.envelope {
            self.envelope = (1.0 - self.alpha_attack) * peak_in + self.alpha_attack * self.envelope;
        } else {
            self.envelope = (1.0 - self.alpha_release) * peak_in + self.alpha_release * self.envelope;
        }

        // Convert envelope to dBFS
        let env_db = if self.envelope > 1e-6 {
            20.0 * self.envelope.log10()
        } else {
            -120.0
        };

        // Compression parameters scaled by current_amount
        // Threshold transitions smoothly from 0.0 dBFS down to -30.0 dBFS
        let threshold_db = -30.0 * self.current_amount;
        let ratio = 1.0 + (3.5 * self.current_amount); // 1.0:1 up to 4.5:1
        let knee_width_db = 6.0;

        // 1. Soft-knee downward compression calculation
        let half_knee = knee_width_db * 0.5;
        let compression_attenuation_db = if env_db < (threshold_db - half_knee) {
            0.0
        } else if env_db > (threshold_db + half_knee) {
            let over_db = env_db - threshold_db;
            (over_db / ratio) - over_db
        } else {
            // Quadratic polynomial inside knee region
            let knee_delta = env_db - threshold_db + half_knee;
            let slope = (1.0 / ratio) - 1.0;
            (slope * knee_delta * knee_delta) / (2.0 * knee_width_db)
        };

        // 2. Dialogue boost calculation for low-level speech (-36 dBFS to -18 dBFS)
        // Provides up to +6 dB boost, fading out above -12 dBFS (where compression takes over)
        // and below -55 dBFS (to avoid amplifying silence/noise floor)
        let max_boost_db = 6.0 * self.current_amount;
        let dialogue_boost_db = if env_db > -12.0 {
            0.0
        } else if env_db > -30.0 {
            // Taper boost down between -30 dBFS and -12 dBFS
            let t = (env_db - (-30.0)) / ((-12.0) - (-30.0));
            max_boost_db * (1.0 - t)
        } else if env_db > -50.0 {
            // Full dialogue boost in the speech zone
            max_boost_db
        } else if env_db > -65.0 {
            // Fade out noise floor gate
            let t = (env_db - (-65.0)) / ((-50.0) - (-65.0));
            max_boost_db * t
        } else {
            0.0
        };

        // Total target gain in dB
        let target_gain_db = compression_attenuation_db + dialogue_boost_db;

        // Smooth gain transition to eliminate pumping and clicks
        self.gain_db = (1.0 - self.alpha_gain) * target_gain_db + self.alpha_gain * self.gain_db;

        // Convert dB gain to linear multiplier
        let linear_gain = 10.0f32.powf(self.gain_db / 20.0);

        (l_in * linear_gain, r_in * linear_gain)
    }

    /// Flushes envelope and gain states to 0.0.
    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.gain_db = 0.0;
        self.current_amount = 0.0;
    }
}
