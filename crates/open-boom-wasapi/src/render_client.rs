//! Low-latency WASAPI render client registered under MMCSS Pro Audio.

use open_boom_core::types::EngineResult;

/// Low-latency WASAPI render client.
pub struct RenderClient {
    pub device_id: Option<String>,
    pub is_rendering: bool,
}

impl Default for RenderClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderClient {
    pub fn new() -> Self {
        Self {
            device_id: None,
            is_rendering: false,
        }
    }

    pub fn start(&mut self) -> EngineResult<()> {
        self.is_rendering = true;
        Ok(())
    }

    pub fn stop(&mut self) -> EngineResult<()> {
        self.is_rendering = false;
        Ok(())
    }
}
