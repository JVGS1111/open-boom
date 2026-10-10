//! Audio engine coordinator managing the unified lifecycle of WASAPI capture and render.

use open_boom_core::types::{EngineResult, TelemetrySnapshot};
use crate::loopback_capture::{LoopbackCapture, LoopbackConfig};
use crate::render_client::{RenderClient, RenderConfig};

/// High-level audio engine coordinator managing user-mode capture and output.
pub struct AudioEngineCoordinator {
    capture: LoopbackCapture,
    render: RenderClient,
    is_running: bool,
}

impl Default for AudioEngineCoordinator {
    fn default() -> Self {
        Self::new(LoopbackConfig::default(), RenderConfig::default())
    }
}

impl AudioEngineCoordinator {
    /// Creates a new audio engine coordinator with the specified capture and render configs.
    pub fn new(capture_config: LoopbackConfig, render_config: RenderConfig) -> Self {
        Self {
            capture: LoopbackCapture::new(capture_config),
            render: RenderClient::with_config(render_config),
            is_running: false,
        }
    }

    /// Reference to the loopback capture client.
    pub fn capture(&self) -> &LoopbackCapture {
        &self.capture
    }

    /// Mutable reference to the loopback capture client.
    pub fn capture_mut(&mut self) -> &mut LoopbackCapture {
        &mut self.capture
    }

    /// Reference to the render client.
    pub fn render(&self) -> &RenderClient {
        &self.render
    }

    /// Mutable reference to the render client.
    pub fn render_mut(&mut self) -> &mut RenderClient {
        &mut self.render
    }

    /// Returns whether the audio engine is actively running.
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Calculates total round-trip audio latency in milliseconds.
    pub fn round_trip_latency_ms(&self) -> f32 {
        let capture_latency = 10.0; // Standard 10ms capture buffer
        let render_latency = self.render.latency_ms().max(5.0);
        capture_latency + render_latency
    }

    /// Generates a telemetry snapshot of engine state and measured latency.
    pub fn telemetry_snapshot(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            limiter_gain_reduction_db: 0.0,
            round_trip_latency_ms: self.round_trip_latency_ms(),
            peak_output_l_dbfs: -100.0,
            peak_output_r_dbfs: -100.0,
            limiter_active: false,
            dsp_active: self.is_running,
        }
    }

    /// Starts both capture and render clients in the proper initialization order.
    pub fn start(&mut self) -> EngineResult<()> {
        self.render.start()?;
        if let Err(err) = self.capture.start() {
            let _ = self.render.stop();
            return Err(err);
        }
        self.is_running = true;
        Ok(())
    }

    /// Stops both audio streams cleanly.
    pub fn stop(&mut self) -> EngineResult<()> {
        let _ = self.capture.stop();
        let _ = self.render.stop();
        self.is_running = false;
        Ok(())
    }
}
