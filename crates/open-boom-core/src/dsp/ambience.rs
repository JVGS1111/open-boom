//! Ambience reverb module.

pub struct AmbienceModule {
    pub enabled: bool,
    pub amount: f32,
}

impl Default for AmbienceModule {
    fn default() -> Self {
        Self {
            enabled: false,
            amount: 0.25,
        }
    }
}

impl AmbienceModule {
    #[inline]
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        (left, right)
    }

    pub fn reset(&mut self) {}
}
