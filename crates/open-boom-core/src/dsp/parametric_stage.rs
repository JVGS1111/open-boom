//! Parametric AutoEq stage evaluating up to 12 biquad filters per channel.

use super::biquad::Biquad;
use crate::types::MAX_PARAMETRIC_FILTERS;

pub struct ParametricStage {
    pub filters_l: [Biquad; MAX_PARAMETRIC_FILTERS],
    pub filters_r: [Biquad; MAX_PARAMETRIC_FILTERS],
    pub active_count: usize,
    pub preamp_gain: f32,
    pub enabled: bool,
}

impl Default for ParametricStage {
    fn default() -> Self {
        Self::new()
    }
}

impl ParametricStage {
    pub fn new() -> Self {
        Self {
            filters_l: [Biquad::default(); MAX_PARAMETRIC_FILTERS],
            filters_r: [Biquad::default(); MAX_PARAMETRIC_FILTERS],
            active_count: 0,
            preamp_gain: 1.0,
            enabled: false,
        }
    }

    #[inline]
    pub fn process_stereo(&mut self, mut left: f32, mut right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }

        left *= self.preamp_gain;
        right *= self.preamp_gain;

        for i in 0..self.active_count.min(MAX_PARAMETRIC_FILTERS) {
            left = self.filters_l[i].process_sample(left);
            right = self.filters_r[i].process_sample(right);
        }

        (left, right)
    }

    pub fn reset(&mut self) {
        for f in &mut self.filters_l {
            f.reset();
        }
        for f in &mut self.filters_r {
            f.reset();
        }
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
}
