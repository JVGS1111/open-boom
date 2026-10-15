//! Hexagonal platform audio host boundary (ADR-037).
//!
//! Every platform audio adapter (Windows WASAPI, macOS CoreAudio, Linux PipeWire,
//! headless mock) implements [`AudioHostBackend`] so the application and UI layers can
//! drive capture, DSP and render without knowing which native audio API is underneath.
//!
//! The trait is intentionally small and platform-agnostic: it exposes the lifecycle,
//! the interleaved Float32 stereo chunk boundary, and device enumeration/switching.
//! Latency and the active device id are exposed as provided accessors so backends can
//! surface their real buffer configuration without forcing every implementation to
//! invent a value.

use crate::types::{AudioDeviceInfo, EngineResult};

/// Platform-agnostic audio host adapter boundary.
pub trait AudioHostBackend {
    /// Starts capture and render streams in a healthy initialization order.
    fn start(&mut self) -> EngineResult<()>;

    /// Stops all audio streams and releases device resources.
    fn stop(&mut self) -> EngineResult<()>;

    /// Returns whether the backend is currently streaming audio.
    fn is_running(&self) -> bool;

    /// Processes one interleaved Float32 stereo chunk through the DSP pipeline.
    ///
    /// When master power is disabled the implementation must fall back to a
    /// bit-perfect passthrough copy with zero DSP arithmetic.
    fn process_chunk(&mut self, input: &[f32], output: &mut [f32]);

    /// Enumerates the audio render endpoints exposed by the host.
    fn enumerate_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>>;

    /// Switches the active render endpoint, or restores the host default when `None`.
    fn switch_device(&mut self, device_id: Option<String>) -> EngineResult<()>;

    /// Reports the configured round-trip latency in milliseconds.
    ///
    /// Implementations must derive this from their actual capture/render buffer
    /// configuration plus the terminal DSP lookahead. There is no universal
    /// sub-15 ms guarantee: only the documented default configuration is expected
    /// to stay below that budget.
    fn round_trip_latency_ms(&self) -> f32 {
        0.0
    }

    /// Returns the currently pinned render device id, or `None` when using the host default.
    fn current_device_id(&self) -> Option<String> {
        None
    }
}
