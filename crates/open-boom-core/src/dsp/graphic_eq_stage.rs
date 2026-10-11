//! 31-Band ISO 1/3-octave graphic equalizer stage.
//!
//! Features Robert Bristow-Johnson Audio EQ Cookbook second-order peaking biquads
//! with constant Q = 4.31, Direct Form II Transposed topology, sub-buffer block
//! parameter smoothing (every 32 samples), strict pole stability enforcement,
//! and atomic snapshot updates.

use super::biquad::{Biquad, BiquadCoefficients};
use crate::types::{
    EQ_Q, EQ_SUB_BUFFER_SIZE, GRAPHIC_EQ_BANDS, INTERNAL_SAMPLE_RATE_F32,
    ISO_31_BAND_FREQUENCIES, MAX_EQ_GAIN_DB, MIN_EQ_GAIN_DB,
};

/// Slew rate factor for exponential smoothing per 32-sample sub-buffer.
const SMOOTHING_FACTOR: f32 = 0.25;

/// 31-band cascade graphic equalizer for stereo audio processing.
pub struct GraphicEqStage {
    pub filters_l: [Biquad; GRAPHIC_EQ_BANDS],
    pub filters_r: [Biquad; GRAPHIC_EQ_BANDS],
    current_coeffs: [BiquadCoefficients; GRAPHIC_EQ_BANDS],
    target_coeffs: [BiquadCoefficients; GRAPHIC_EQ_BANDS],
    current_gains: [f32; GRAPHIC_EQ_BANDS],
    target_gains: [f32; GRAPHIC_EQ_BANDS],
    sample_rate: f32,
    sub_buffer_counter: usize,
    pub preamp_gain: f32,
    pub preamp_offset_db: f32,
    target_preamp_gain: f32,
    pub enabled: bool,
    is_smoothing: bool,
}

impl Default for GraphicEqStage {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphicEqStage {
    /// Creates a new 31-band graphic equalizer initialized with flat 0.0 dB response
    /// at the standard 48 kHz sample rate.
    pub fn new() -> Self {
        Self::with_sample_rate(INTERNAL_SAMPLE_RATE_F32)
    }

    /// Creates a new 31-band graphic equalizer with a custom sample rate.
    pub fn with_sample_rate(sample_rate: f32) -> Self {
        let safe_sr = if sample_rate.is_finite() && sample_rate > 0.0 {
            sample_rate
        } else {
            INTERNAL_SAMPLE_RATE_F32
        };

        let mut filters_l = [Biquad::default(); GRAPHIC_EQ_BANDS];
        let mut filters_r = [Biquad::default(); GRAPHIC_EQ_BANDS];
        let mut coeffs = [BiquadCoefficients::default(); GRAPHIC_EQ_BANDS];

        for i in 0..GRAPHIC_EQ_BANDS {
            let c = BiquadCoefficients::peaking_eq(
                safe_sr,
                ISO_31_BAND_FREQUENCIES[i],
                EQ_Q,
                0.0,
            );
            coeffs[i] = c;
            filters_l[i] = Biquad::new(c);
            filters_r[i] = Biquad::new(c);
        }

        Self {
            filters_l,
            filters_r,
            current_coeffs: coeffs,
            target_coeffs: coeffs,
            current_gains: [0.0; GRAPHIC_EQ_BANDS],
            target_gains: [0.0; GRAPHIC_EQ_BANDS],
            sample_rate: safe_sr,
            sub_buffer_counter: 0,
            preamp_gain: 1.0,
            preamp_offset_db: 0.0,
            target_preamp_gain: 1.0,
            enabled: true,
            is_smoothing: false,
        }
    }

    /// Updates the sample rate and recomputes all filter coefficients.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let safe_sr = if sample_rate.is_finite() && sample_rate > 0.0 {
            sample_rate
        } else {
            INTERNAL_SAMPLE_RATE_F32
        };
        self.sample_rate = safe_sr;

        for i in 0..GRAPHIC_EQ_BANDS {
            let c = BiquadCoefficients::peaking_eq(
                safe_sr,
                ISO_31_BAND_FREQUENCIES[i],
                EQ_Q,
                self.target_gains[i],
            );
            self.target_coeffs[i] = c;
            self.current_coeffs[i] = c;
            self.current_gains[i] = self.target_gains[i];
            self.filters_l[i].coeffs = c;
            self.filters_r[i].coeffs = c;
        }

        self.is_smoothing = false;
        self.sub_buffer_counter = 0;
        self.reset();
    }

    /// Sets the gain for an individual frequency band (0..30) in decibels.
    /// Clamps to [-12.0, +12.0 dB] and smoothly interpolates across sub-buffers.
    pub fn set_band_gain(&mut self, band_idx: usize, gain_db: f32) {
        if band_idx >= GRAPHIC_EQ_BANDS {
            return;
        }

        let safe_gain = if gain_db.is_nan() {
            0.0
        } else {
            gain_db.clamp(MIN_EQ_GAIN_DB, MAX_EQ_GAIN_DB)
        };

        self.target_gains[band_idx] = safe_gain;
        self.target_coeffs[band_idx] = BiquadCoefficients::peaking_eq(
            self.sample_rate,
            ISO_31_BAND_FREQUENCIES[band_idx],
            EQ_Q,
            safe_gain,
        );
        self.is_smoothing = true;
    }

    /// Sets the gain for an individual frequency band immediately without smoothing.
    pub fn set_band_gain_immediate(&mut self, band_idx: usize, gain_db: f32) {
        if band_idx >= GRAPHIC_EQ_BANDS {
            return;
        }

        let safe_gain = if gain_db.is_nan() {
            0.0
        } else {
            gain_db.clamp(MIN_EQ_GAIN_DB, MAX_EQ_GAIN_DB)
        };

        let coeffs = BiquadCoefficients::peaking_eq(
            self.sample_rate,
            ISO_31_BAND_FREQUENCIES[band_idx],
            EQ_Q,
            safe_gain,
        );

        self.target_gains[band_idx] = safe_gain;
        self.current_gains[band_idx] = safe_gain;
        self.target_coeffs[band_idx] = coeffs;
        self.current_coeffs[band_idx] = coeffs;
        self.filters_l[band_idx].coeffs = coeffs;
        self.filters_r[band_idx].coeffs = coeffs;
    }

    /// Sets gains for all 31 bands with sub-buffer parameter smoothing.
    pub fn set_gains(&mut self, gains: &[f32; GRAPHIC_EQ_BANDS]) {
        for (i, &gain) in gains.iter().enumerate() {
            self.set_band_gain(i, gain);
        }
    }

    /// Sets gains for all 31 bands immediately without smoothing.
    pub fn set_gains_immediate(&mut self, gains: &[f32; GRAPHIC_EQ_BANDS]) {
        for (i, &gain) in gains.iter().enumerate() {
            self.set_band_gain_immediate(i, gain);
        }
    }

    /// Atomically loads a graphic EQ snapshot and preamp attenuation offset.
    pub fn load_snapshot(&mut self, gains_db: [f32; GRAPHIC_EQ_BANDS], preamp_offset_db: f32) {
        self.set_gains(&gains_db);
        self.set_preamp_offset_db(preamp_offset_db);
    }

    /// Atomically loads a graphic EQ snapshot immediately without smoothing.
    pub fn load_snapshot_immediate(&mut self, gains_db: [f32; GRAPHIC_EQ_BANDS], preamp_offset_db: f32) {
        self.set_gains_immediate(&gains_db);
        self.set_preamp_offset_db_immediate(preamp_offset_db);
    }

    /// Sets the preamp offset in decibels with smoothing.
    pub fn set_preamp_offset_db(&mut self, offset_db: f32) {
        let safe_db = if offset_db.is_nan() {
            0.0
        } else {
            offset_db.clamp(-24.0, 12.0)
        };
        self.preamp_offset_db = safe_db;
        self.target_preamp_gain = 10.0f32.powf(safe_db / 20.0);
        self.is_smoothing = true;
    }

    /// Sets the preamp offset in decibels immediately.
    pub fn set_preamp_offset_db_immediate(&mut self, offset_db: f32) {
        let safe_db = if offset_db.is_nan() {
            0.0
        } else {
            offset_db.clamp(-24.0, 12.0)
        };
        self.preamp_offset_db = safe_db;
        let gain = 10.0f32.powf(safe_db / 20.0);
        self.target_preamp_gain = gain;
        self.preamp_gain = gain;
    }

    /// Returns whether the graphic EQ stage is bypassed.
    #[inline]
    pub fn is_bypassed(&self) -> bool {
        !self.enabled
    }

    /// Toggles bypass state.
    #[inline]
    pub fn set_bypass(&mut self, bypassed: bool) {
        self.enabled = !bypassed;
    }

    /// Returns whether the graphic EQ stage is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets whether the graphic EQ stage is enabled.
    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns current smoothed band gains.
    #[inline]
    pub fn current_gains(&self) -> &[f32; GRAPHIC_EQ_BANDS] {
        &self.current_gains
    }

    /// Returns target band gains.
    #[inline]
    pub fn target_gains(&self) -> &[f32; GRAPHIC_EQ_BANDS] {
        &self.target_gains
    }

    /// Returns current filter coefficients for a band.
    #[inline]
    pub fn coefficients(&self, band_idx: usize) -> Option<BiquadCoefficients> {
        self.current_coeffs.get(band_idx).copied()
    }

    /// Flushes all pending smoothing steps immediately to reach target parameters.
    pub fn flush_smoothing(&mut self) {
        self.current_gains = self.target_gains;
        self.current_coeffs = self.target_coeffs;
        self.preamp_gain = self.target_preamp_gain;
        for i in 0..GRAPHIC_EQ_BANDS {
            self.filters_l[i].coeffs = self.target_coeffs[i];
            self.filters_r[i].coeffs = self.target_coeffs[i];
        }
        self.is_smoothing = false;
    }

    /// Performs a single sub-buffer smoothing step, interpolating coefficients
    /// and preamp gain towards their target values.
    pub fn step_smoothing(&mut self) {
        if !self.is_smoothing {
            return;
        }

        let mut still_smoothing = false;

        // Smooth preamp gain
        let p_diff = self.target_preamp_gain - self.preamp_gain;
        if p_diff.abs() > 1e-4 {
            self.preamp_gain += p_diff * SMOOTHING_FACTOR;
            still_smoothing = true;
        } else {
            self.preamp_gain = self.target_preamp_gain;
        }

        // Smooth biquad coefficients for all 31 bands
        for i in 0..GRAPHIC_EQ_BANDS {
            let g_diff = self.target_gains[i] - self.current_gains[i];
            if g_diff.abs() > 1e-3 || self.current_coeffs[i] != self.target_coeffs[i] {
                let next_coeffs = self.current_coeffs[i].lerp(&self.target_coeffs[i], SMOOTHING_FACTOR);
                let coeff_diff = (next_coeffs.b0 - self.target_coeffs[i].b0).abs()
                    .max((next_coeffs.b1 - self.target_coeffs[i].b1).abs())
                    .max((next_coeffs.b2 - self.target_coeffs[i].b2).abs())
                    .max((next_coeffs.a1 - self.target_coeffs[i].a1).abs())
                    .max((next_coeffs.a2 - self.target_coeffs[i].a2).abs());

                if coeff_diff < 1e-5 && g_diff.abs() < 1e-3 {
                    self.current_coeffs[i] = self.target_coeffs[i];
                    self.current_gains[i] = self.target_gains[i];
                } else {
                    self.current_coeffs[i] = next_coeffs;
                    self.current_gains[i] += g_diff * SMOOTHING_FACTOR;
                    still_smoothing = true;
                }

                self.filters_l[i].coeffs = self.current_coeffs[i];
                self.filters_r[i].coeffs = self.current_coeffs[i];
            }
        }

        self.is_smoothing = still_smoothing;
    }

    /// Resets all internal delay lines to zero.
    pub fn reset(&mut self) {
        for f in &mut self.filters_l {
            f.reset();
        }
        for f in &mut self.filters_r {
            f.reset();
        }
        self.sub_buffer_counter = 0;
    }

    /// Process a stereo frame through the 31-band cascade with sub-buffer parameter smoothing.
    #[inline]
    pub fn process_stereo(&mut self, mut left: f32, mut right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }

        self.sub_buffer_counter += 1;
        if self.sub_buffer_counter >= EQ_SUB_BUFFER_SIZE {
            self.sub_buffer_counter = 0;
            self.step_smoothing();
        }

        left *= self.preamp_gain;
        right *= self.preamp_gain;

        for i in 0..GRAPHIC_EQ_BANDS {
            left = self.filters_l[i].process_sample(left);
            right = self.filters_r[i].process_sample(right);
        }

        (left, right)
    }

    /// Process a single mono sample through the left channel cascade only (without touching right delay lines).
    #[inline]
    pub fn process_mono_left(&mut self, mut sample: f32) -> f32 {
        if !self.enabled {
            return sample;
        }

        self.sub_buffer_counter += 1;
        if self.sub_buffer_counter >= EQ_SUB_BUFFER_SIZE {
            self.sub_buffer_counter = 0;
            self.step_smoothing();
        }

        sample *= self.preamp_gain;

        for i in 0..GRAPHIC_EQ_BANDS {
            sample = self.filters_l[i].process_sample(sample);
        }

        sample
    }

    /// Calculates theoretical cumulative frequency response in dB at the specified frequency.
    pub fn cumulative_response_db(&self, freq: f32) -> f32 {
        let mut total_db = self.preamp_offset_db;
        for i in 0..GRAPHIC_EQ_BANDS {
            total_db += self.target_coeffs[i].frequency_response_db(freq, self.sample_rate);
        }
        total_db
    }
}
