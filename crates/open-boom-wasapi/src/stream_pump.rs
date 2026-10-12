//! Real-time continuous WASAPI audio stream pump.
//!
//! Spawns a dedicated MMCSS "Pro Audio" worker thread that captures audio from an
//! input endpoint (Virtual Audio Device / Loopback), routes buffers through the
//! internal DSP pipeline (`AudioEngineCoordinator::process_audio_chunk`), and renders
//! the enhanced audio to the active physical output endpoint with sub-15ms latency.

use open_boom_core::types::{EngineError, EngineResult};
use crate::coordinator::AudioEngineCoordinator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Configuration for the WASAPI audio streaming pump.
#[derive(Debug, Clone)]
pub struct StreamPumpConfig {
    /// Target physical render device ID (e.g. Headphones/Speakers). None = system default.
    pub target_device_id: Option<String>,
    /// Capture device ID (e.g. Boom Audio virtual endpoint). None = auto-detect virtual device.
    pub capture_device_id: Option<String>,
    /// Whether synthetic/mock streaming is allowed in headless test environments.
    pub allow_synthetic: bool,
    /// Whether WASAPI buffer guard (25ms safe buffer + anti-glitch margin) is enabled.
    pub buffer_guard_enabled: bool,
    /// Whether to force driverless WASAPI loopback capture on the physical device (Mode B),
    /// ignoring any installed virtual audio devices (e.g. Boom Audio).
    pub force_loopback: bool,
}

impl Default for StreamPumpConfig {
    fn default() -> Self {
        Self {
            target_device_id: None,
            capture_device_id: None,
            allow_synthetic: false,
            buffer_guard_enabled: true,
            force_loopback: false,
        }
    }
}

/// Active streaming pump handle managing the audio processing loop.
pub struct AudioStreamPump {
    is_running: Arc<AtomicBool>,
    worker_handle: Option<JoinHandle<()>>,
}

impl AudioStreamPump {
    /// Starts the real-time audio streaming pump worker thread.
    pub fn start(
        coordinator: Arc<Mutex<AudioEngineCoordinator>>,
        config: StreamPumpConfig,
    ) -> EngineResult<Self> {
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_clone = is_running.clone();

        #[cfg(windows)]
        let handle = std::thread::Builder::new()
            .name("open-boom-audio-pump".to_string())
            .spawn(move || {
                if config.allow_synthetic {
                    while is_running_clone.load(Ordering::Relaxed) {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    return;
                }
                Self::run_windows_pump(coordinator, config, is_running_clone);
            })
            .map_err(|e| {
                EngineError::DeviceInitialization(format!(
                    "Failed to spawn audio pump thread: {}",
                    e
                ))
            })?;

        #[cfg(not(windows))]
        let handle = std::thread::Builder::new()
            .name("open-boom-audio-pump".to_string())
            .spawn(move || {
                // Synthetic / headless loop for non-Windows platforms
                while is_running_clone.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            })
            .map_err(|e| {
                EngineError::DeviceInitialization(format!(
                    "Failed to spawn synthetic pump thread: {}",
                    e
                ))
            })?;

        Ok(Self {
            is_running,
            worker_handle: Some(handle),
        })
    }

    /// Stops the real-time audio pump and waits for the worker thread to exit.
    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }

    /// Returns whether the audio pump is actively running.
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    #[cfg(windows)]
    fn run_windows_pump(
        coordinator: Arc<Mutex<AudioEngineCoordinator>>,
        config: StreamPumpConfig,
        is_running: Arc<AtomicBool>,
    ) {
        use std::time::Duration;
        use windows::core::HSTRING;
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
            CoTaskMemFree, STGM_READ,
        };
        use windows::Win32::Media::Audio::{
            eRender, DEVICE_STATE_ACTIVE, IAudioCaptureClient, IAudioClient,
            IAudioRenderClient, IMMDevice, IMMDeviceEnumerator,
            MMDeviceEnumerator, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK,
            AUDCLNT_BUFFERFLAGS_SILENT, WAVEFORMATEX,
        };
        use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
        use crate::mmcss::MmcssGuard;

        let com_init = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let _mmcss = MmcssGuard::register("Pro Audio");

        unsafe {
            let _ = (|| {
                let enumerator_res: windows::core::Result<IMMDeviceEnumerator> =
                    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);

                let enumerator = match enumerator_res {
                    Ok(e) => e,
                    Err(err) => {
                        eprintln!("AudioStreamPump: Failed to create MMDeviceEnumerator: {:?}", err);
                        return;
                    }
                };

            // 1. Resolve Target Output Render Endpoint (Must be physical, never virtual sink)
            let render_device: IMMDevice = if let Some(ref target_id) = config.target_device_id {
                let h_id = HSTRING::from(target_id.as_str());
                match enumerator.GetDevice(&h_id) {
                    Ok(dev) => dev,
                    Err(_) => match Self::find_physical_render_device(&enumerator) {
                        Some(dev) => dev,
                        None => return,
                    },
                }
            } else {
                match Self::find_physical_render_device(&enumerator) {
                    Some(dev) => dev,
                    None => return,
                }
            };

            let render_client: IAudioClient = match render_device.Activate(CLSCTX_ALL, None) {
                Ok(c) => c,
                Err(_) => return,
            };

            let render_fmt_ptr: *mut WAVEFORMATEX = match render_client.GetMixFormat() {
                Ok(f) => f,
                Err(_) => return,
            };

            let buffer_duration_hns = if config.buffer_guard_enabled {
                250_000 // 25ms safe buffer preventing dropouts under heavy bass
            } else {
                100_000 // 10ms aggressive low-latency buffer
            };

            let render_init_res = render_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                0,
                buffer_duration_hns,
                0,
                render_fmt_ptr,
                None,
            );

            if render_init_res.is_err() {
                CoTaskMemFree(Some(render_fmt_ptr as _));
                return;
            }

            let render_svc: IAudioRenderClient = match render_client.GetService() {
                Ok(s) => s,
                Err(_) => {
                    CoTaskMemFree(Some(render_fmt_ptr as _));
                    return;
                }
            };

            let render_buffer_size = render_client.GetBufferSize().unwrap_or(480);
            let _ = render_client.Start();

            // 2. Resolve Capture Endpoint (Prioritize Virtual Audio Sink unless force_loopback is set)
            let mut capture_device_opt: Option<IMMDevice> = None;
            if !config.force_loopback {
                if let Some(ref cap_id) = config.capture_device_id {
                    let h_id = HSTRING::from(cap_id.as_str());
                    capture_device_opt = enumerator.GetDevice(&h_id).ok();
                }

                if capture_device_opt.is_none() {
                    // Auto-detect virtual device
                    if let Ok(collection) = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) {
                        if let Ok(count) = collection.GetCount() {
                            for i in 0..count {
                                if let Ok(dev) = collection.Item(i) {
                                    if let Ok(props) = dev.OpenPropertyStore(STGM_READ) {
                                        if let Ok(name_var) = props.GetValue(&PKEY_Device_FriendlyName) {
                                            let name = name_var.to_string();
                                            if crate::device::AudioDeviceManager::is_virtual_device_name(&name) {
                                                capture_device_opt = Some(dev);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // If no virtual device found or force_loopback is active, capture directly from the physical render device (loopback)
            let capture_device = capture_device_opt.unwrap_or_else(|| render_device.clone());

            let capture_client: IAudioClient = match capture_device.Activate(CLSCTX_ALL, None) {
                Ok(c) => c,
                Err(_) => {
                    let _ = render_client.Stop();
                    CoTaskMemFree(Some(render_fmt_ptr as _));
                    return;
                }
            };

            let capture_fmt_ptr: *mut WAVEFORMATEX = match capture_client.GetMixFormat() {
                Ok(f) => f,
                Err(_) => {
                    let _ = render_client.Stop();
                    CoTaskMemFree(Some(render_fmt_ptr as _));
                    return;
                }
            };

            let cap_init_res = capture_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK,
                100_000,
                0,
                capture_fmt_ptr,
                None,
            );

            if cap_init_res.is_err() {
                let _ = render_client.Stop();
                CoTaskMemFree(Some(render_fmt_ptr as _));
                CoTaskMemFree(Some(capture_fmt_ptr as _));
                return;
            }

            let capture_svc: IAudioCaptureClient = match capture_client.GetService() {
                Ok(s) => s,
                Err(_) => {
                    let _ = render_client.Stop();
                    CoTaskMemFree(Some(render_fmt_ptr as _));
                    CoTaskMemFree(Some(capture_fmt_ptr as _));
                    return;
                }
            };

            let render_channels = (*render_fmt_ptr).nChannels.max(1) as usize;
            let render_sample_rate = (*render_fmt_ptr).nSamplesPerSec;
            let capture_channels = (*capture_fmt_ptr).nChannels.max(1) as usize;

            if let Ok(mut coord) = coordinator.lock() {
                coord.processor_mut().set_sample_rate(render_sample_rate);
            }

            let _ = capture_client.Start();

            // Preallocate scratch buffers (zero heap allocations in processing loop, AD-1)
            let scratch_capacity = 16384;
            let mut input_scratch = vec![0.0f32; scratch_capacity];
            let mut output_scratch = vec![0.0f32; scratch_capacity];

            // Real-Time Audio Streaming Pump Loop
            while is_running.load(Ordering::Relaxed) {
                let packet_size = capture_svc.GetNextPacketSize().unwrap_or(0);
                if packet_size == 0 {
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }

                let mut p_data: *mut u8 = std::ptr::null_mut();
                let mut frames_read = 0u32;
                let mut flags = 0u32;

                if capture_svc
                    .GetBuffer(&mut p_data, &mut frames_read, &mut flags, None, None)
                    .is_ok()
                {
                    let frames = frames_read as usize;
                    let stereo_sample_count = frames * 2;
                    if frames > 0 && !p_data.is_null() && stereo_sample_count <= scratch_capacity {
                        if (flags & (AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)) != 0 {
                            input_scratch[..stereo_sample_count].fill(0.0);
                        } else if capture_channels == 2 {
                            let incoming = std::slice::from_raw_parts(
                                p_data as *const f32,
                                stereo_sample_count,
                            );
                            input_scratch[..stereo_sample_count].copy_from_slice(incoming);
                        } else if capture_channels >= 8 {
                            let incoming = std::slice::from_raw_parts(
                                p_data as *const f32,
                                frames * capture_channels,
                            );
                            for f in 0..frames {
                                let base = f * capture_channels;
                                let fl = incoming[base];
                                let fr = incoming[base + 1];
                                let c = incoming[base + 2] * std::f32::consts::FRAC_1_SQRT_2;
                                let lfe = incoming[base + 3] * 0.5;
                                let rl = incoming[base + 4] * std::f32::consts::FRAC_1_SQRT_2;
                                let rr = incoming[base + 5] * std::f32::consts::FRAC_1_SQRT_2;
                                let sl = incoming[base + 6] * std::f32::consts::FRAC_1_SQRT_2;
                                let sr = incoming[base + 7] * std::f32::consts::FRAC_1_SQRT_2;
                                input_scratch[f * 2] = fl + c + lfe + rl + sl;
                                input_scratch[f * 2 + 1] = fr + c + lfe + rr + sr;
                            }
                        } else if capture_channels == 1 {
                            let incoming = std::slice::from_raw_parts(
                                p_data as *const f32,
                                frames,
                            );
                            for f in 0..frames {
                                let mono = incoming[f];
                                input_scratch[f * 2] = mono;
                                input_scratch[f * 2 + 1] = mono;
                            }
                        } else {
                            let incoming = std::slice::from_raw_parts(
                                p_data as *const f32,
                                frames * capture_channels,
                            );
                            for f in 0..frames {
                                let base = f * capture_channels;
                                input_scratch[f * 2] = incoming[base];
                                input_scratch[f * 2 + 1] = incoming[base + 1];
                            }
                        }

                        // Process audio frames through DSP pipeline
                        // Spin-wait briefly if coordinator lock momentarily contended by UI telemetry,
                        // never dropping DSP to passthrough during active playback
                        let mut processed = false;
                        for _ in 0..16 {
                            if let Ok(mut coord) = coordinator.try_lock() {
                                coord.process_audio_chunk(
                                    &input_scratch[..stereo_sample_count],
                                    &mut output_scratch[..stereo_sample_count],
                                );
                                processed = true;
                                break;
                            }
                            std::hint::spin_loop();
                        }
                        if !processed {
                            if let Ok(mut coord) = coordinator.lock() {
                                coord.process_audio_chunk(
                                    &input_scratch[..stereo_sample_count],
                                    &mut output_scratch[..stereo_sample_count],
                                );
                            } else {
                                output_scratch[..stereo_sample_count]
                                    .copy_from_slice(&input_scratch[..stereo_sample_count]);
                            }
                        }

                        // Write processed audio frames to physical output DAC with retry loop
                        let max_retries = if config.buffer_guard_enabled { 25 } else { 12 };
                        for _ in 0..max_retries {
                            let padding = render_client.GetCurrentPadding().unwrap_or(0);
                            let available = render_buffer_size.saturating_sub(padding);

                            if available >= frames_read {
                                if let Ok(render_ptr) = render_svc.GetBuffer(frames_read) {
                                    let render_slice = std::slice::from_raw_parts_mut(
                                        render_ptr as *mut f32,
                                        frames * render_channels,
                                    );
                                    if render_channels == 2 {
                                        render_slice
                                            .copy_from_slice(&output_scratch[..stereo_sample_count]);
                                    } else if render_channels == 1 {
                                        for f in 0..frames {
                                            render_slice[f] = 0.5
                                                * (output_scratch[f * 2]
                                                    + output_scratch[f * 2 + 1]);
                                        }
                                    } else {
                                        for f in 0..frames {
                                            let base = f * render_channels;
                                            render_slice[base] = output_scratch[f * 2];
                                            render_slice[base + 1] = output_scratch[f * 2 + 1];
                                            for c in 2..render_channels {
                                                render_slice[base + c] = 0.0;
                                            }
                                        }
                                    }
                                    let _ = render_svc.ReleaseBuffer(frames_read, 0);
                                }
                                break;
                            }
                            std::thread::sleep(Duration::from_micros(400));
                        }
                    }
                    let _ = capture_svc.ReleaseBuffer(frames_read);
                }
            }

            // Clean shutdown
            let _ = capture_client.Stop();
            let _ = render_client.Stop();
            CoTaskMemFree(Some(render_fmt_ptr as _));
            CoTaskMemFree(Some(capture_fmt_ptr as _));
        })();
        }

        unsafe {
            if com_init.is_ok() {
                CoUninitialize();
            }
        }
    }

    #[cfg(windows)]
    fn find_physical_render_device(
        enumerator: &windows::Win32::Media::Audio::IMMDeviceEnumerator,
    ) -> Option<windows::Win32::Media::Audio::IMMDevice> {
        use windows::Win32::Media::Audio::{eConsole, eRender, DEVICE_STATE_ACTIVE};
        use windows::Win32::System::Com::STGM_READ;
        use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;

        // Check if default render endpoint is non-virtual
        if let Ok(dev) = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole) } {
            if let Ok(props) = unsafe { dev.OpenPropertyStore(STGM_READ) } {
                if let Ok(name_var) = unsafe { props.GetValue(&PKEY_Device_FriendlyName) } {
                    let name = name_var.to_string();
                    if !crate::device::AudioDeviceManager::is_virtual_device_name(&name) {
                        return Some(dev);
                    }
                }
            }
        }

        // If default is virtual (e.g. Boom Audio), find first active non-virtual render endpoint
        if let Ok(collection) = unsafe { enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) } {
            if let Ok(count) = unsafe { collection.GetCount() } {
                for i in 0..count {
                    if let Ok(dev) = unsafe { collection.Item(i) } {
                        if let Ok(props) = unsafe { dev.OpenPropertyStore(STGM_READ) } {
                            if let Ok(name_var) = unsafe { props.GetValue(&PKEY_Device_FriendlyName) } {
                                let name = name_var.to_string();
                                if !crate::device::AudioDeviceManager::is_virtual_device_name(&name) {
                                    return Some(dev);
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

impl Drop for AudioStreamPump {
    fn drop(&mut self) {
        self.stop();
    }
}
