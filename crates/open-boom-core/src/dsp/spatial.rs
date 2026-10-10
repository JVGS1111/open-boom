//! Spatial stereo widener and crossfeed module.

pub struct SpatialWidenerModule {
    pub enabled: bool,
    pub width: f32,
}

impl Default for SpatialWidenerModule {
    fn default() -> Self {
        Self {
            enabled: false,
            width: 0.4,
        }
    }
}

impl SpatialWidenerModule {
    #[inline]
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        (left, right)
    }

    pub fn reset(&mut self) {}
}
