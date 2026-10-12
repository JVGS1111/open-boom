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
            allow_synthetic_fallback: true,
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
        let channels = (config.channels as usize).clamp(1, crate::resampler::MAX_RESAMPLER_CHANNELS);
        let resampler = Resampler::with_channels(config.sample_rate, channels);
        // Preallocate scratch space for up to ~100ms at 192 kHz 8-channel (153,600 samples)
        let scratch_capacity = 163_840;
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
        if !self.is_capturing {
            return Err(EngineError::BufferError(
                "Capture client is not running".to_string(),
            ));
        }

        if self.config.channels == 0
            || self.config.channels as usize > crate::resampler::MAX_RESAMPLER_CHANNELS
        {
            return Err(EngineError::UnsupportedFormat(format!(
                "Channels {} outside supported range 1..={}",
                self.config.channels,
                crate::resampler::MAX_RESAMPLER_CHANNELS
            )));
        }

        if raw_bytes.is_empty() {
            return Ok(0);
        }

        let bps = self.config.format.bytes_per_sample();
        let sample_count = raw_bytes.len() / bps;
        let channels = self.config.channels as usize;
        let input_frames = sample_count / channels;

        let required_frames = self.resampler.required_output_frames(input_frames);
        let required_samples = required_frames * channels;
        if output_frames_48k.len() < required_samples {
            return Err(EngineError::BufferError(format!(
                "Output buffer capacity {} too small for required {} samples ({} frames)",
                output_frames_48k.len(),
                required_samples,
                required_frames
            )));
        }

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
        if self.config.channels == 0
            || self.config.channels as usize > crate::resampler::MAX_RESAMPLER_CHANNELS
        {
            return Err(EngineError::UnsupportedFormat(format!(
                "Channels {} outside supported range 1..={}",
                self.config.channels,
                crate::resampler::MAX_RESAMPLER_CHANNELS
            )));
        }

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
            use windows::Win32::System::Threading::GetCurrentProcessId;

            // Attempt MTA COM initialization
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            }

            // Attempt Mode B (Process Loopback with process tree exclusion) or Mode A endpoint acquisition
            let dev_result = (|| -> windows::core::Result<LoopbackMode> {
                let enumerator: IMMDeviceEnumerator =
                    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
                let _device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole)? };

                // If excluding current process, verify Mode B process loopback activation succeeds
                if self.config.exclude_current_process {
                    let target_pid = self
                        .config
                        .target_process_id
                        .unwrap_or_else(|| unsafe { GetCurrentProcessId() });
                    if activate_process_loopback_client(target_pid).is_ok() {
                        return Ok(LoopbackMode::ProcessLoopbackModeB);
                    }
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

#[cfg(windows)]
#[windows_core::implement(windows::Win32::Media::Audio::IActivateAudioInterfaceCompletionHandler)]
struct ProcessLoopbackCompletionHandler {
    tx: std::sync::Mutex<
        Option<
            std::sync::mpsc::Sender<
                windows::core::Result<windows::Win32::Media::Audio::IAudioClient>,
            >,
        >,
    >,
}

#[cfg(windows)]
impl windows::Win32::Media::Audio::IActivateAudioInterfaceCompletionHandler_Impl
    for ProcessLoopbackCompletionHandler_Impl
{
    fn ActivateCompleted(
        &self,
        activateoperation: Option<
            &windows::Win32::Media::Audio::IActivateAudioInterfaceAsyncOperation,
        >,
    ) -> windows::core::Result<()> {
        use windows::core::Interface;
        use windows::Win32::Media::Audio::IAudioClient;

        let result = (|| -> windows::core::Result<IAudioClient> {
            let op = activateoperation.ok_or_else(|| {
                windows::core::Error::from_hresult(windows::core::HRESULT(0x80004003u32 as i32))
            })?;
            let mut activate_hr = windows::core::HRESULT(0);
            let mut activated_unk: Option<windows::core::IUnknown> = None;
            unsafe {
                op.GetActivateResult(&mut activate_hr, &mut activated_unk)?;
            }
            activate_hr.ok()?;
            let unk = activated_unk.ok_or_else(|| {
                windows::core::Error::from_hresult(windows::core::HRESULT(0x80004005u32 as i32))
            })?;
            unk.cast::<IAudioClient>()
        })();

        if let Ok(mut guard) = self.tx.lock() {
            if let Some(sender) = guard.take() {
                let _ = sender.send(result);
            }
        }
        Ok(())
    }
}

/// Activates a Windows 10 (build 20348+ / 19041+) / Windows 11 Process Loopback `IAudioClient`
/// configured with `PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE` for `exclude_process_id`.
///
/// This guarantees at the Windows Audio Engine (`audiodg.exe`) mixer level that audio rendered
/// by `exclude_process_id` (OpenBoom's own process tree) is excluded from the captured loopback
/// stream, eliminating positive acoustic feedback loops and self-oscillation in Mode B.
#[cfg(windows)]
pub fn activate_process_loopback_client(
    exclude_process_id: u32,
) -> windows::core::Result<windows::Win32::Media::Audio::IAudioClient> {
    use std::mem::{size_of, ManuallyDrop};
    use std::time::Duration;
    use windows::core::Interface;
    use windows::Win32::Media::Audio::{
        ActivateAudioInterfaceAsync, IActivateAudioInterfaceCompletionHandler, IAudioClient,
        AUDIOCLIENT_ACTIVATION_PARAMS, AUDIOCLIENT_ACTIVATION_PARAMS_0,
        AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK, AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS,
        PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE, VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK,
    };

    let mut activation_params = AUDIOCLIENT_ACTIVATION_PARAMS {
        ActivationType: AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK,
        Anonymous: AUDIOCLIENT_ACTIVATION_PARAMS_0 {
            ProcessLoopbackParams: AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS {
                TargetProcessId: exclude_process_id,
                ProcessLoopbackMode: PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE,
            },
        },
    };

    // VT_BLOB = 65. Wrap in ManuallyDrop so PropVariantClear never attempts CoTaskMemFree on stack memory.
    let raw_propvariant = windows_core::imp::PROPVARIANT {
        Anonymous: windows_core::imp::PROPVARIANT_0 {
            Anonymous: windows_core::imp::PROPVARIANT_0_0 {
                vt: 65u16,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: windows_core::imp::PROPVARIANT_0_0_0 {
                    blob: windows_core::imp::BLOB {
                        cbSize: size_of::<AUDIOCLIENT_ACTIVATION_PARAMS>() as u32,
                        pBlobData: &mut activation_params as *mut _ as *mut u8,
                    },
                },
            },
        },
    };
    let propvariant =
        ManuallyDrop::new(unsafe { windows_core::PROPVARIANT::from_raw(raw_propvariant) });

    let (tx, rx) = std::sync::mpsc::channel();
    let handler: IActivateAudioInterfaceCompletionHandler = ProcessLoopbackCompletionHandler {
        tx: std::sync::Mutex::new(Some(tx)),
    }
    .into();

    let _operation = unsafe {
        ActivateAudioInterfaceAsync(
            VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK,
            &IAudioClient::IID,
            Some(&*propvariant as *const windows_core::PROPVARIANT),
            &handler,
        )?
    };

    rx.recv_timeout(Duration::from_secs(3)).map_err(|_| {
        windows::core::Error::from_hresult(windows::core::HRESULT(0x800705B4u32 as i32))
    })?
}

