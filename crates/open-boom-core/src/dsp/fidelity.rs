//! Fidelity harmonic exciter and clarity enhancer.

pub struct FidelityModule {
    pub enabled: bool,
    pub amount: f32,
}

impl Default for FidelityModule {
    fn default() -> Self {
        Self {
            enabled: false,
            amount: 0.3,
        }
    }
}

impl FidelityModule {
    #[inline]
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        (left, right)
    }

    pub fn reset(&mut self) {}
}
