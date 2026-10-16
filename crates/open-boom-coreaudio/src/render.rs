//! CoreAudio HAL render client.
//!
//! `write_samples` enqueues frames into the SPSC ring that the render IOProc drains
//! into the output device. Switching the render endpoint updates the OS default output
//! and rebinds the stream; device loss restores the default endpoint.

use std::sync::Arc;

use open_boom_core::types::{EngineError, EngineResult, INTERNAL_SAMPLE_RATE};

use crate::hal::{AudioHal, HalStream, HalStreamConfig};
use crate::ring_buffer::SampleRing;

/// Configuration for the CoreAudio render client.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Render device id, or `None` for the system default output endpoint.
    pub device_id: Option<String>,
    /// Sample rate in Hertz.
    pub sample_rate: u32,
    /// Interleaved channel count.
    pub channels: u16,
    /// Buffer size in frames.
    pub buffer_frames: u32,
    /// Whether synthetic fallback is permitted in headless/CI environments.
    pub allow_synthetic_fallback: bool,
    /// Whether to force deterministic synthetic rendering.
    pub force_synthetic: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            device_id: None,
            sample_rate: INTERNAL_SAMPLE_RATE,
            channels: crate::DEFAULT_STEREO_CHANNELS,
            buffer_frames: crate::DEFAULT_BUFFER_FRAMES,
            allow_synthetic_fallback: false,
            force_synthetic: false,
        }
    }
}

/// CoreAudio output render client.
pub struct CoreAudioRender {
    hal: Arc<dyn AudioHal>,
    config: RenderConfig,
    ring: Arc<SampleRing>,
    stream: Option<Box<dyn HalStream>>,
    is_rendering: bool,
}

impl CoreAudioRender {
    /// Creates a render client bound to the given HAL and configuration.
    pub fn new(hal: Arc<dyn AudioHal>, config: RenderConfig) -> Self {
        Self {
            hal,
            config,
            ring: SampleRing::new(crate::DEFAULT_RING_CAPACITY_SAMPLES),
            stream: None,
            is_rendering: false,
        }
    }

    /// Returns the SPSC ring drained by the render IOProc.
    pub fn ring(&self) -> Arc<SampleRing> {
        self.ring.clone()
    }

    /// Returns the render configuration.
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    /// Returns the currently pinned render device id, if any.
    pub fn device_id(&self) -> Option<&str> {
        self.config.device_id.as_deref()
    }

    /// Returns whether rendering is currently running.
    pub fn is_rendering(&self) -> bool {
        self.is_rendering
    }

    /// Negotiated sample rate (from the stream once started, otherwise the request).
    pub fn sample_rate(&self) -> u32 {
        self.stream
            .as_ref()
            .map(|s| s.sample_rate())
            .unwrap_or(self.config.sample_rate)
    }

    /// Negotiated buffer size in frames.
    pub fn buffer_frames(&self) -> u32 {
        self.stream
            .as_ref()
            .map(|s| s.buffer_frames())
            .unwrap_or(self.config.buffer_frames)
    }

    /// Number of samples dropped because the render ring was full.
    pub fn dropped_samples(&self) -> usize {
        self.ring.dropped_samples()
    }

    /// Render-side latency implied by the configured buffer.
    pub fn latency_ms(&self) -> f32 {
        let rate = self.sample_rate().max(1) as f32;
        (self.buffer_frames() as f32 / rate) * 1000.0
    }

    /// Opens and starts the render stream.
    pub fn start(&mut self) -> EngineResult<()> {
        if self.is_rendering {
            return Ok(());
        }

        // Drop any samples left from a previous run so a restart never replays stale audio.
        self.ring.clear();

        let config = HalStreamConfig {
            device_id: self.config.device_id.clone(),
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
            buffer_frames: self.config.buffer_frames,
        };
        let mut stream = self.hal.open_render(config, self.ring.clone())?;
        stream.start()?;
        self.stream = Some(stream);
        self.is_rendering = true;
        Ok(())
    }

    /// Stops the render stream.
    pub fn stop(&mut self) -> EngineResult<()> {
        if let Some(mut stream) = self.stream.take() {
            let result = stream.stop();
            self.is_rendering = false;
            return result;
        }
        self.is_rendering = false;
        Ok(())
    }

    /// Enqueues interleaved Float32 frames for the render IOProc to consume.
    ///
    /// Only whole frames are enqueued; samples that do not fit are reported through
    /// [`Self::dropped_samples`]. Returns the number of samples actually enqueued.
    pub fn write_samples(&mut self, samples: &[f32]) -> usize {
        let channels = self.config.channels.max(1) as usize;
        self.ring.push_frames(samples, channels) * channels
    }

    /// Headless verification hook: runs the render IOProc for `frames` frames.
    ///
    /// Real CoreAudio streams are driven by the OS and return `Ok(0)`.
    pub fn drive_io_proc(&mut self, frames: usize) -> EngineResult<usize> {
        match self.stream.as_mut() {
            Some(stream) => stream.drive_io_proc(frames),
            None => Ok(0),
        }
    }

    /// Sets the target render device endpoint ID without restarting the stream.
    pub fn set_device_id(&mut self, device_id: Option<String>) {
        self.config.device_id = device_id;
    }

    /// Switches the render endpoint, updating the OS default output device.
    ///
    /// `None` restores the system default endpoint. The target is validated and applied
    /// before the running stream is touched; if the rebind fails, the previous device id
    /// is restored and the client is left in a stopped (not "rendering") state.
    pub fn switch_device(&mut self, device_id: Option<String>) -> EngineResult<()> {
        self.switch_device_internal(device_id, true)
    }

    /// Switches the render endpoint without mutating the OS default output device
    /// (crucial in Mode A / Virtual Sink mode where BlackHole must remain the OS default).
    pub fn switch_device_endpoint_only(&mut self, device_id: Option<String>) -> EngineResult<()> {
        self.switch_device_internal(device_id, false)
    }

    /// Internal device switcher with configurable OS default update.
    pub fn switch_device_internal(&mut self, device_id: Option<String>, update_os_default: bool) -> EngineResult<()> {
        // Resolve and validate the target before mutating any state.
        let resolved: Option<String> = match device_id {
            Some(ref id) => {
                if self.hal.device_name(id)?.is_none() {
                    return Err(EngineError::DeviceInitialization(format!(
                        "cannot switch to unknown render device: {id}"
                    )));
                }
                Some(id.clone())
            }
            None => self.hal.default_output_device_id()?,
        };

        let previous_device_id = self.config.device_id.clone();
        let was_rendering = self.is_rendering;

        if update_os_default {
            if let Some(ref id) = resolved {
                self.hal.set_default_output_device(id)?;
            }
        }
        self.config.device_id = resolved;

        if was_rendering {
            if let Err(err) = self.stop() {
                self.config.device_id = previous_device_id;
                return Err(err);
            }
            if let Err(err) = self.start() {
                // Roll back the device id and try to bring the previous endpoint back.
                self.config.device_id = previous_device_id;
                let _ = self.start();
                return Err(err);
            }
        }
        Ok(())
    }

    /// Restores the system default output endpoint after active-device loss.
    pub fn handle_device_lost(&mut self) -> EngineResult<()> {
        let was_rendering = self.is_rendering;
        if was_rendering {
            let _ = self.stop();
        }

        // Detach from the lost endpoint so the stream rebinds to the host default.
        self.config.device_id = None;

        if was_rendering {
            self.start()?;
        }
        Ok(())
    }
}
