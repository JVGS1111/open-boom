//! 31-Band ISO 1/3-octave graphic equalizer stage.

use super::biquad::Biquad;
use crate::types::GRAPHIC_EQ_BANDS;

pub struct GraphicEqStage {
    pub filters_l: [Biquad; GRAPHIC_EQ_BANDS],
    pub filters_r: [Biquad; GRAPHIC_EQ_BANDS],
    pub preamp_gain: f32,
    pub enabled: bool,
}

impl Default for GraphicEqStage {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphicEqStage {
    pub fn new() -> Self {
        Self {
            filters_l: [Biquad::default(); GRAPHIC_EQ_BANDS],
            filters_r: [Biquad::default(); GRAPHIC_EQ_BANDS],
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

        for i in 0..GRAPHIC_EQ_BANDS {
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
