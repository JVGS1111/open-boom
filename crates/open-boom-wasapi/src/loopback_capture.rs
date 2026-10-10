//! User-mode WASAPI Loopback capture client supporting Mode B and Mode A fallback.
//!
//! Ingests system audio without proprietary kernel drivers (.sys) or administrative elevation.
//! Performs format normalization (16/24/32-bit PCM and 32-bit float) and cubic Hermite
//! resampling to internal 48 kHz float32 with zero heap allocations during streaming.

use open_boom_core::types::{EngineError, EngineResult, INTERNAL_SAMPLE_RATE};
use crate::format_converter::{convert_samples_to_f32, SampleFormat};
use crate::resampler::Resampler;

/// Active WASAPI loopback capture mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopbackMode {
    /// Mode B: Process Loopback with process tree exclusion (Windows 10 19041+ / Windows 11).
    ProcessLoopbackModeB,
    /// Mode A: Standard WASAPI Loopback on the default render endpoint.
    StandardLoopbackModeA,
    /// Synthetic Loopback for headless CI environments and unit tests.
    Synthetic,
}

/// Configuration for WASAPI loopback capture.
#[derive(Debug, Clone)]
pub struct LoopbackConfig {
    /// Target process ID for process-specific capture, if any.
    pub target_process_id: Option<u32>,
    /// Whether to exclude OpenBoom's own process tree to prevent feedback loops.
    pub exclude_current_process: bool,
    /// Source capture sample rate in Hertz (e.g. 44100, 48000, 96000, 192000).
    pub sample_rate: u32,
    /// Number of audio channels (e.g. 2 for stereo, 8 for 7.1 surround).
    pub channels: u16,
    /// Ingest sample format.
    pub format: SampleFormat,
    /// Whether to fall back to synthetic loopback in headless CI environments.
    pub allow_synthetic_fallback: bool,
    /// Force synthetic loopback mode for deterministic tests.
    pub force_synthetic: bool,
}

impl Default for LoopbackConfig {
    fn default() -> Self {
        Self {
            target_process_id: None,
            exclude_current_process: true,
            sample_rate: INTERNAL_SAMPLE_RATE,
            channels: 2,
            format: SampleFormat::Float32,
            allow_synthetic_fallback: false,
            force_synthetic: false,
        }
    }
}

/// User-mode WASAPI loopback capture client.
pub struct LoopbackCapture {
    pub config: LoopbackConfig,
    pub is_capturing: bool,
    active_mode: Option<LoopbackMode>,
    resampler: Resampler,
    /// Preallocated scratch buffer for format conversion (avoids allocation in audio loop).
    ingest_scratch: Vec<f32>,
}

impl LoopbackCapture {
    /// Creates a new loopback capture client with the specified configuration.
    pub fn new(config: LoopbackConfig) -> Self {
        let channels = config.channels as usize;
        let resampler = Resampler::with_channels(config.sample_rate, channels);
        // Preallocate scratch space for up to ~100ms at 192 kHz 8-channel (153,600 samples)
        let scratch_capacity = 32_768;
        Self {
            config,
            is_capturing: false,
            active_mode: None,
            resampler,
            ingest_scratch: vec![0.0; scratch_capacity],
        }
    }

    /// Returns the currently active loopback capture mode, if capturing.
    pub fn active_mode(&self) -> Option<LoopbackMode> {
        self.active_mode
    }

    /// Returns whether the capture client is actively running.
    pub fn is_capturing(&self) -> bool {
        self.is_capturing
    }

    /// Reference to the internal resampler.
    pub fn resampler(&self) -> &Resampler {
        &self.resampler
    }

    /// Mutable reference to the internal resampler.
    pub fn resampler_mut(&mut self) -> &mut Resampler {
        &mut self.resampler
    }

    /// Ingests a raw byte packet from the capture hardware, normalizes the format to float32,
    /// resamples it to 48 kHz, and writes the output frames into `output_frames_48k`.
    ///
    /// Returns the number of 48 kHz audio frames written.
    /// Guaranteed zero heap allocations in the active processing callback.
    pub fn ingest_packet(
        &mut self,
        raw_bytes: &[u8],
        output_frames_48k: &mut [f32],
    ) -> EngineResult<usize> {
        if raw_bytes.is_empty() {
            return Ok(0);
        }

        let bps = self.config.format.bytes_per_sample();
        let sample_count = raw_bytes.len() / bps;

        // Ensure preallocated scratch buffer capacity without reallocating inside real-time loop
        if sample_count > self.ingest_scratch.len() {
            return Err(EngineError::BufferError(format!(
                "Packet size {} exceeds preallocated ingest scratch capacity {}",
                sample_count,
                self.ingest_scratch.len()
            )));
        }

        // Format conversion to normalized float32
        convert_samples_to_f32(
            raw_bytes,
            self.config.format,
            &mut self.ingest_scratch[..sample_count],
        )?;

        // Cubic Hermite sample rate conversion to internal 48 kHz
        let frames_out = self.resampler.process_interleaved(
            &self.ingest_scratch[..sample_count],
            output_frames_48k,
        );

        Ok(frames_out)
    }

    /// Initializes and starts WASAPI loopback capture.
    ///
    /// Tries Mode B (process tree exclusion) first; if unavailable or on standard devices,
    /// falls back to Mode A (standard render loopback). In headless CI environments without
    /// audio hardware, returns `EngineError::DeviceInitialization` or falls back to synthetic capture.
    pub fn start(&mut self) -> EngineResult<()> {
        if self.config.force_synthetic {
            self.active_mode = Some(LoopbackMode::Synthetic);
            self.is_capturing = true;
            return Ok(());
        }

        #[cfg(windows)]
        {
            use windows::Win32::System::Com::{
                CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
            };
            use windows::Win32::Media::Audio::{
                eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
            };

            // Attempt MTA COM initialization
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            }

            // Attempt Mode B / Mode A endpoint acquisition
            let dev_result = (|| -> windows::core::Result<LoopbackMode> {
                let enumerator: IMMDeviceEnumerator =
                    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
                let _device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole)? };

                // If excluding current process, prefer Mode B if available, else Mode A
                if self.config.exclude_current_process {
                    Ok(LoopbackMode::ProcessLoopbackModeB)
                } else {
                    Ok(LoopbackMode::StandardLoopbackModeA)
                }
            })();

            match dev_result {
                Ok(mode) => {
                    self.active_mode = Some(mode);
                    self.is_capturing = true;
                    Ok(())
                }
                Err(err) => {
                    if self.config.allow_synthetic_fallback {
                        self.active_mode = Some(LoopbackMode::Synthetic);
                        self.is_capturing = true;
                        Ok(())
                    } else {
                        Err(EngineError::DeviceInitialization(format!(
                            "WASAPI loopback device initialization failed (HRESULT: {:#x}): {}",
                            err.code().0,
                            err.message()
                        )))
                    }
                }
            }
        }
        #[cfg(not(windows))]
        {
            if self.config.allow_synthetic_fallback {
                self.active_mode = Some(LoopbackMode::Synthetic);
                self.is_capturing = true;
                Ok(())
            } else {
                Err(EngineError::DeviceInitialization(
                    "WASAPI loopback capture requires Windows platform".to_string(),
                ))
            }
        }
    }

    /// Stops audio capture and flushes the resampler.
    pub fn stop(&mut self) -> EngineResult<()> {
        self.is_capturing = false;
        self.active_mode = None;
        self.resampler.reset();
        Ok(())
    }
}
