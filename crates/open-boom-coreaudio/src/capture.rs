//! CoreAudio HAL capture client.
//!
//! `read_samples` is a pure drain of the SPSC ring that the capture IOProc publishes
//! into. Capture audio is never synthesized here; the only synthetic path is the
//! explicit forced-synthetic [`crate::hal::MockHal`] tone, which lives in the HAL boundary.

use std::sync::Arc;

use open_boom_core::types::{EngineResult, INTERNAL_SAMPLE_RATE};

use crate::hal::{AudioHal, HalStream, HalStreamConfig};
use crate::ring_buffer::SampleRing;

/// Configuration for the CoreAudio capture client.
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Capture device id, or `None` for the system default input endpoint.
    pub device_id: Option<String>,
    /// Sample rate in Hertz.
    pub sample_rate: u32,
    /// Interleaved channel count.
    pub channels: u16,
    /// Buffer size in frames.
    pub buffer_frames: u32,
    /// Whether synthetic fallback is permitted in headless/CI environments.
    pub allow_synthetic_fallback: bool,
    /// Whether to force deterministic synthetic capture.
    pub force_synthetic: bool,
}

impl Default for CaptureConfig {
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

/// CoreAudio input capture client.
pub struct CoreAudioCapture {
    hal: Arc<dyn AudioHal>,
    config: CaptureConfig,
    ring: Arc<SampleRing>,
    stream: Option<Box<dyn HalStream>>,
    is_capturing: bool,
}

impl CoreAudioCapture {
    /// Creates a capture client bound to the given HAL and configuration.
    pub fn new(hal: Arc<dyn AudioHal>, config: CaptureConfig) -> Self {
        Self {
            hal,
            config,
            ring: SampleRing::new(crate::DEFAULT_RING_CAPACITY_SAMPLES),
            stream: None,
            is_capturing: false,
        }
    }

    /// Returns the SPSC ring filled by the capture IOProc.
    pub fn ring(&self) -> Arc<SampleRing> {
        self.ring.clone()
    }

    /// Returns the capture configuration.
    pub fn config(&self) -> &CaptureConfig {
        &self.config
    }

    /// Returns whether capture is currently running.
    pub fn is_capturing(&self) -> bool {
        self.is_capturing
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

    /// Number of samples dropped because the capture ring was full.
    pub fn dropped_samples(&self) -> usize {
        self.ring.dropped_samples()
    }

    /// Capture-side latency implied by the configured buffer.
    pub fn latency_ms(&self) -> f32 {
        let rate = self.sample_rate().max(1) as f32;
        (self.buffer_frames() as f32 / rate) * 1000.0
    }

    /// Opens and starts the capture stream.
    pub fn start(&mut self) -> EngineResult<()> {
        if self.is_capturing {
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
        let mut stream = self.hal.open_capture(config, self.ring.clone())?;
        stream.start()?;
        self.stream = Some(stream);
        self.is_capturing = true;
        Ok(())
    }

    /// Stops the capture stream.
    pub fn stop(&mut self) -> EngineResult<()> {
        if let Some(mut stream) = self.stream.take() {
            let result = stream.stop();
            self.is_capturing = false;
            return result;
        }
        self.is_capturing = false;
        Ok(())
    }

    /// Drains interleaved Float32 samples published by the capture IOProc.
    ///
    /// Only whole frames are drained, so a caller can never observe a half frame.
    /// Returns the number of samples actually read; the unwritten tail of `out` is
    /// left untouched so callers never process stale or zero-filled frames by mistake.
    pub fn read_samples(&mut self, out: &mut [f32]) -> usize {
        let channels = self.config.channels.max(1) as usize;
        self.ring.pop_frames(out, channels) * channels
    }

    /// Headless verification hook: runs the capture IOProc for `frames` frames.
    ///
    /// Real CoreAudio streams are driven by the OS and return `Ok(0)`.
    pub fn drive_io_proc(&mut self, frames: usize) -> EngineResult<usize> {
        match self.stream.as_mut() {
            Some(stream) => stream.drive_io_proc(frames),
            None => Ok(0),
        }
    }
}
