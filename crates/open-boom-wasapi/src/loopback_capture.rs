//! WASAPI process loopback capture adapter.

use open_boom_core::types::EngineResult;

/// Configuration for WASAPI loopback capture.
#[derive(Debug, Clone)]
pub struct LoopbackConfig {
    pub target_process_id: Option<u32>,
    pub exclude_current_process: bool,
    pub sample_rate: u32,
    pub channels: u16,
}

impl Default for LoopbackConfig {
    fn default() -> Self {
        Self {
            target_process_id: None,
            exclude_current_process: true,
            sample_rate: 48_000,
            channels: 2,
        }
    }
}

/// User-mode WASAPI loopback capture client.
pub struct LoopbackCapture {
    pub config: LoopbackConfig,
    pub is_capturing: bool,
}

impl LoopbackCapture {
    pub fn new(config: LoopbackConfig) -> Self {
        Self {
            config,
            is_capturing: false,
        }
    }

    pub fn start(&mut self) -> EngineResult<()> {
        self.is_capturing = true;
        Ok(())
    }

    pub fn stop(&mut self) -> EngineResult<()> {
        self.is_capturing = false;
        Ok(())
    }
}
