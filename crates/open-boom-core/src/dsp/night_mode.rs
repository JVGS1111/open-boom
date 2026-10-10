//! Night Mode dynamic range compressor.

pub struct NightModeModule {
    pub enabled: bool,
    pub amount: f32,
}

impl Default for NightModeModule {
    fn default() -> Self {
        Self {
            enabled: false,
            amount: 0.5,
        }
    }
}

impl NightModeModule {
    #[inline]
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        (left, right)
    }

    pub fn reset(&mut self) {}
}
