//! 3D Surround spatializer down-folding 7.1 surround to binaural stereo.

use crate::types::SURROUND_7_1_CHANNELS;

pub struct Spatializer3D {
    pub enabled: bool,
    pub intensity: f32,
}

impl Default for Spatializer3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Spatializer3D {
    pub fn new() -> Self {
        Self {
            enabled: false,
            intensity: 0.7,
        }
    }

    /// Down-fold 8 surround channels [FL, FR, FC, LFE, BL, BR, SL, SR] to binaural stereo [L, R].
    #[inline]
    pub fn process_frame(&self, channels: &[f32; SURROUND_7_1_CHANNELS]) -> (f32, f32) {
        if !self.enabled {
            // Direct stereo passthrough using FL and FR
            return (channels[0], channels[1]);
        }

        let fl = channels[0];
        let fr = channels[1];
        let fc = channels[2] * 0.7071;
        let lfe = channels[3] * 0.5;
        let bl = channels[4] * 0.6;
        let br = channels[5] * 0.6;
        let sl = channels[6] * 0.65;
        let sr = channels[7] * 0.65;

        let out_l = fl + fc + lfe + bl + sl;
        let out_r = fr + fc + lfe + br + sr;

        (out_l * 0.5, out_r * 0.5)
    }

    pub fn reset(&mut self) {}
}
