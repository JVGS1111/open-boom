//! Dedicated Parametric AutoEq Stage evaluating up to 12 biquad filters per channel
//! positioned sequentially before the 31-band graphic equalizer.
//!
//! Features Robert Bristow-Johnson Peaking, Low-Shelf, and High-Shelf biquads,
//! automatic negative preamp attenuation offset, sub-buffer block smoothing (32 samples),
//! and Direct Form II Transposed sample processing.

use super::biquad::{Biquad, BiquadCoefficients};
use crate::types::{
    EQ_SUB_BUFFER_SIZE, INTERNAL_SAMPLE_RATE_F32, MAX_PARAMETRIC_FILTERS,
    ParametricFilterPod, ParametricFilterType,
};

/// Slew rate factor for exponential parameter smoothing per 32-sample sub-buffer.
const SMOOTHING_FACTOR: f32 = 0.25;

/// Dedicated parametric AutoEq correction stage for stereo audio processing.
pub struct ParametricStage {
    pub filters_l: [Biquad; MAX_PARAMETRIC_FILTERS],
    pub filters_r: [Biquad; MAX_PARAMETRIC_FILTERS],
    current_coeffs: [BiquadCoefficients; MAX_PARAMETRIC_FILTERS],
    target_coeffs: [BiquadCoefficients; MAX_PARAMETRIC_FILTERS],
    target_filters: [ParametricFilterPod; MAX_PARAMETRIC_FILTERS],
    pub active_count: usize,
    sample_rate: f32,
    sub_buffer_counter: usize,
    pub preamp_gain: f32,
    pub preamp_offset_db: f32,
    target_preamp_gain: f32,
    pub enabled: bool,
    is_smoothing: bool,
}

impl Default for ParametricStage {
    fn default() -> Self {
        Self::new()
    }
}

impl ParametricStage {
    /// Creates a new parametric AutoEq stage initialized with passthrough filters
    /// at the default 48 kHz internal sample rate.
    pub fn new() -> Self {
        Self::with_sample_rate(INTERNAL_SAMPLE_RATE_F32)
    }

    /// Creates a new parametric AutoEq stage with a custom sample rate.
    pub fn with_sample_rate(sample_rate: f32) -> Self {
        let safe_sr = if sample_rate.is_finite() && sample_rate > 0.0 {
            sample_rate
        } else {
            INTERNAL_SAMPLE_RATE_F32
        };

        let filters_l = [Biquad::default(); MAX_PARAMETRIC_FILTERS];
        let filters_r = [Biquad::default(); MAX_PARAMETRIC_FILTERS];
        let coeffs = [BiquadCoefficients::identity(); MAX_PARAMETRIC_FILTERS];
        let target_filters = [ParametricFilterPod::default(); MAX_PARAMETRIC_FILTERS];

        Self {
            filters_l,
            filters_r,
            current_coeffs: coeffs,
            target_coeffs: coeffs,
            target_filters,
            active_count: 0,
            sample_rate: safe_sr,
            sub_buffer_counter: 0,
            preamp_gain: 1.0,
            preamp_offset_db: 0.0,
            target_preamp_gain: 1.0,
            enabled: true,
            is_smoothing: false,
        }
    }

    /// Calculates automatic negative preamp attenuation offset from positive filter gains:
    /// $A_{\text{preamp}} = -\max(0, \max_i(G_i))$.
    #[inline]
    pub fn calculate_auto_preamp_offset(filters: &[ParametricFilterPod]) -> f32 {
        let max_pos = filters
            .iter()
            .map(|f| if f.gain_db.is_nan() { 0.0 } else { f.gain_db })
            .fold(0.0f32, f32::max);

        if max_pos > 0.0 {
            -max_pos
        } else {
            0.0
        }
    }

    /// Updates the sample rate and recomputes all active filter coefficients.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let safe_sr = if sample_rate.is_finite() && sample_rate > 0.0 {
            sample_rate
        } else {
            INTERNAL_SAMPLE_RATE_F32
        };
        self.sample_rate = safe_sr;

        for i in 0..self.active_count.min(MAX_PARAMETRIC_FILTERS) {
            let f = &self.target_filters[i];
            let c = match f.filter_type {
                ParametricFilterType::Peaking => {
                    BiquadCoefficients::peaking_eq(safe_sr, f.frequency_hz, f.q, f.gain_db)
                }
                ParametricFilterType::LowShelf => {
                    BiquadCoefficients::low_shelf(safe_sr, f.frequency_hz, f.q, f.gain_db)
                }
                ParametricFilterType::HighShelf => {
                    BiquadCoefficients::high_shelf(safe_sr, f.frequency_hz, f.q, f.gain_db)
                }
            };
            self.target_coeffs[i] = c;
            self.current_coeffs[i] = c;
            self.filters_l[i].coeffs = c;
            self.filters_r[i].coeffs = c;
        }

        self.is_smoothing = false;
        self.sub_buffer_counter = 0;
        self.reset();
    }

    /// Loads a parametric filter array and sets preamp attenuation offset with 32-sample smoothing.
    pub fn load_profile(&mut self, filters: &[ParametricFilterPod], preamp_offset_db: f32) {
        let count = filters.len().min(MAX_PARAMETRIC_FILTERS);
        let auto_offset = Self::calculate_auto_preamp_offset(&filters[..count]);

        let final_offset = if preamp_offset_db.is_finite() && preamp_offset_db < auto_offset {
            preamp_offset_db.clamp(-36.0, 0.0)
        } else {
            auto_offset.clamp(-36.0, 0.0)
        };

        self.preamp_offset_db = final_offset;
        self.target_preamp_gain = 10.0f32.powf(final_offset / 20.0);
        self.active_count = count;

        for i in 0..count {
            let f = &filters[i];
            self.target_filters[i] = *f;
            let c = match f.filter_type {
                ParametricFilterType::Peaking => {
                    BiquadCoefficients::peaking_eq(self.sample_rate, f.frequency_hz, f.q, f.gain_db)
                }
                ParametricFilterType::LowShelf => {
                    BiquadCoefficients::low_shelf(self.sample_rate, f.frequency_hz, f.q, f.gain_db)
                }
                ParametricFilterType::HighShelf => {
                    BiquadCoefficients::high_shelf(self.sample_rate, f.frequency_hz, f.q, f.gain_db)
                }
            };
            self.target_coeffs[i] = c;
        }

        for i in count..MAX_PARAMETRIC_FILTERS {
            self.target_filters[i] = ParametricFilterPod::default();
            self.target_coeffs[i] = BiquadCoefficients::identity();
            self.current_coeffs[i] = BiquadCoefficients::identity();
            self.filters_l[i].coeffs = BiquadCoefficients::identity();
            self.filters_r[i].coeffs = BiquadCoefficients::identity();
            self.filters_l[i].reset();
            self.filters_r[i].reset();
        }

        self.is_smoothing = true;
    }

    /// Loads a parametric filter array and sets preamp attenuation offset immediately without smoothing.
    pub fn load_profile_immediate(&mut self, filters: &[ParametricFilterPod], preamp_offset_db: f32) {
        let count = filters.len().min(MAX_PARAMETRIC_FILTERS);
        let auto_offset = Self::calculate_auto_preamp_offset(&filters[..count]);

        let final_offset = if preamp_offset_db.is_finite() && preamp_offset_db < auto_offset {
            preamp_offset_db
        } else {
            auto_offset
        };

        self.preamp_offset_db = final_offset;
        let gain = 10.0f32.powf(final_offset / 20.0);
        self.target_preamp_gain = gain;
        self.preamp_gain = gain;
        self.active_count = count;

        for i in 0..count {
            let f = &filters[i];
            self.target_filters[i] = *f;
            let c = match f.filter_type {
                ParametricFilterType::Peaking => {
                    BiquadCoefficients::peaking_eq(self.sample_rate, f.frequency_hz, f.q, f.gain_db)
                }
                ParametricFilterType::LowShelf => {
                    BiquadCoefficients::low_shelf(self.sample_rate, f.frequency_hz, f.q, f.gain_db)
                }
                ParametricFilterType::HighShelf => {
                    BiquadCoefficients::high_shelf(self.sample_rate, f.frequency_hz, f.q, f.gain_db)
                }
            };
            self.target_coeffs[i] = c;
            self.current_coeffs[i] = c;
            self.filters_l[i].coeffs = c;
            self.filters_r[i].coeffs = c;
        }

        for i in count..MAX_PARAMETRIC_FILTERS {
            self.target_filters[i] = ParametricFilterPod::default();
            self.target_coeffs[i] = BiquadCoefficients::identity();
            self.current_coeffs[i] = BiquadCoefficients::identity();
            self.filters_l[i].coeffs = BiquadCoefficients::identity();
            self.filters_r[i].coeffs = BiquadCoefficients::identity();
        }

        self.is_smoothing = false;
        self.reset();
    }

    /// Clears the active profile and smoothly resets preamp gain to 0.0 dB.
    pub fn clear_profile(&mut self) {
        self.active_count = 0;
        self.preamp_offset_db = 0.0;
        self.target_preamp_gain = 1.0;
        for i in 0..MAX_PARAMETRIC_FILTERS {
            self.target_filters[i] = ParametricFilterPod::default();
            self.target_coeffs[i] = BiquadCoefficients::identity();
            self.current_coeffs[i] = BiquadCoefficients::identity();
            self.filters_l[i].coeffs = BiquadCoefficients::identity();
            self.filters_r[i].coeffs = BiquadCoefficients::identity();
            self.filters_l[i].reset();
            self.filters_r[i].reset();
        }
        self.is_smoothing = true;
    }

    /// Clears the active profile immediately and flushes delay lines.
    pub fn clear_profile_immediate(&mut self) {
        self.active_count = 0;
        self.preamp_offset_db = 0.0;
        self.preamp_gain = 1.0;
        self.target_preamp_gain = 1.0;
        for i in 0..MAX_PARAMETRIC_FILTERS {
            self.target_filters[i] = ParametricFilterPod::default();
            self.target_coeffs[i] = BiquadCoefficients::identity();
            self.current_coeffs[i] = BiquadCoefficients::identity();
            self.filters_l[i].coeffs = BiquadCoefficients::identity();
            self.filters_r[i].coeffs = BiquadCoefficients::identity();
        }
        self.is_smoothing = false;
        self.reset();
    }

    /// Returns whether the parametric AutoEq stage is bypassed.
    #[inline]
    pub fn is_bypassed(&self) -> bool {
        !self.enabled
    }

    /// Toggles bypass state.
    #[inline]
    pub fn set_bypass(&mut self, bypassed: bool) {
        self.enabled = !bypassed;
    }

    /// Returns whether the parametric stage is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets whether the parametric stage is enabled.
    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns current filter coefficients for a slot.
    #[inline]
    pub fn coefficients(&self, idx: usize) -> Option<BiquadCoefficients> {
        self.current_coeffs.get(idx).copied()
    }

    /// Flushes all pending smoothing steps immediately to reach target parameters.
    pub fn flush_smoothing(&mut self) {
        self.current_coeffs = self.target_coeffs;
        self.preamp_gain = self.target_preamp_gain;
        for i in 0..MAX_PARAMETRIC_FILTERS {
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

        // Smooth biquad coefficients for all 12 parametric filter slots
        for i in 0..MAX_PARAMETRIC_FILTERS {
            if self.current_coeffs[i] != self.target_coeffs[i] {
                let next_coeffs = self.current_coeffs[i].lerp(&self.target_coeffs[i], SMOOTHING_FACTOR);
                let coeff_diff = (next_coeffs.b0 - self.target_coeffs[i].b0).abs()
                    .max((next_coeffs.b1 - self.target_coeffs[i].b1).abs())
                    .max((next_coeffs.b2 - self.target_coeffs[i].b2).abs())
                    .max((next_coeffs.a1 - self.target_coeffs[i].a1).abs())
                    .max((next_coeffs.a2 - self.target_coeffs[i].a2).abs());

                if coeff_diff < 1e-5 {
                    self.current_coeffs[i] = self.target_coeffs[i];
                } else {
                    self.current_coeffs[i] = next_coeffs;
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

    /// Processes a single stereo audio frame using Direct Form II Transposed topology.
    ///
    /// Applies linear preamp attenuation at stage input, then passes signal through
    /// active parametric biquad filters in series.
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

        if self.active_count == 0 && (self.preamp_gain - 1.0).abs() < 1e-4 {
            return (left, right);
        }

        left *= self.preamp_gain;
        right *= self.preamp_gain;

        let count = self.active_count.min(MAX_PARAMETRIC_FILTERS);
        for i in 0..count {
            left = self.filters_l[i].process_sample(left);
            right = self.filters_r[i].process_sample(right);
        }

        (left, right)
    }

    /// Processes a single left/mono audio sample for odd trailing samples.
    #[inline]
    pub fn process_mono_left(&mut self, mut input: f32) -> f32 {
        if !self.enabled {
            return input;
        }

        if self.active_count == 0 && (self.preamp_gain - 1.0).abs() < 1e-4 {
            return input;
        }

        input *= self.preamp_gain;
        let count = self.active_count.min(MAX_PARAMETRIC_FILTERS);
        for i in 0..count {
            input = self.filters_l[i].process_sample(input);
        }
        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parametric_stage_bounds_clamp() {
        let mut stage = ParametricStage::new();
        stage.enabled = true;
        // Exceed MAX_PARAMETRIC_FILTERS
        stage.active_count = MAX_PARAMETRIC_FILTERS + 10;
        let (l, r) = stage.process_stereo(0.5, -0.5);
        assert!(!l.is_nan());
        assert!(!r.is_nan());
    }

    #[test]
    fn test_auto_preamp_offset_calculation() {
        let filters = [
            ParametricFilterPod {
                filter_type: ParametricFilterType::LowShelf,
                frequency_hz: 105.0,
                q: 0.71,
                gain_db: 5.5,
            },
            ParametricFilterPod {
                filter_type: ParametricFilterType::Peaking,
                frequency_hz: 3200.0,
                q: 2.0,
                gain_db: 6.5,
            },
            ParametricFilterPod {
                filter_type: ParametricFilterType::Peaking,
                frequency_hz: 1000.0,
                q: 1.0,
                gain_db: -3.0,
            },
        ];

        let offset = ParametricStage::calculate_auto_preamp_offset(&filters);
        assert_eq!(offset, -6.5);

        // Negative cuts only
        let cut_filters = [
            ParametricFilterPod {
                filter_type: ParametricFilterType::Peaking,
                frequency_hz: 1000.0,
                q: 1.0,
                gain_db: -4.0,
            },
        ];
        let cut_offset = ParametricStage::calculate_auto_preamp_offset(&cut_filters);
        assert_eq!(cut_offset, 0.0);
    }
}
