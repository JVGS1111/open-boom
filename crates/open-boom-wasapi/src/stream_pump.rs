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
#[derive(Debug, Clone, Default)]
pub struct StreamPumpConfig {
    /// Target physical render device ID (e.g. Headphones/Speakers). None = system default.
    pub target_device_id: Option<String>,
    /// Capture device ID (e.g. Boom Audio virtual endpoint). None = auto-detect virtual device.
    pub capture_device_id: Option<String>,
    /// Whether synthetic/mock streaming is allowed in headless test environments.
    pub allow_synthetic: bool,
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
            eConsole, eRender, DEVICE_STATE_ACTIVE, IAudioCaptureClient, IAudioClient,
            IAudioRenderClient, IMMDevice, IMMDeviceEnumerator,
            MMDeviceEnumerator, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK,
            AUDCLNT_BUFFERFLAGS_SILENT, WAVEFORMATEX,
        };
        use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
        use crate::mmcss::MmcssGuard;

        unsafe {
            let com_init = CoInitializeEx(None, COINIT_MULTITHREADED);
            let _mmcss = MmcssGuard::register("Pro Audio");

            let enumerator_res: windows::core::Result<IMMDeviceEnumerator> =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);

            let enumerator = match enumerator_res {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("AudioStreamPump: Failed to create MMDeviceEnumerator: {:?}", err);
                    if com_init.is_ok() {
                        CoUninitialize();
                    }
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

            let render_init_res = render_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                0,
                100_000, // 10ms buffer
                0,
                render_fmt_ptr,
                None,
            );

            if render_init_res.is_err() {
                CoTaskMemFree(Some(render_fmt_ptr as _));
                if com_init.is_ok() {
                    CoUninitialize();
                }
                return;
            }

            let render_svc: IAudioRenderClient = match render_client.GetService() {
                Ok(s) => s,
                Err(_) => {
                    CoTaskMemFree(Some(render_fmt_ptr as _));
                    if com_init.is_ok() {
                        CoUninitialize();
                    }
                    return;
                }
            };

            let render_buffer_size = render_client.GetBufferSize().unwrap_or(480);
            let _ = render_client.Start();

            // 2. Resolve Capture Endpoint (Prioritize Virtual Audio Sink, e.g. Boom Audio)
            let mut capture_device_opt: Option<IMMDevice> = None;
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

            // If no virtual device found, capture from the default render device (loopback)
            let capture_device = capture_device_opt.unwrap_or_else(|| {
                enumerator.GetDefaultAudioEndpoint(eRender, eConsole).unwrap_or(render_device.clone())
            });

            let capture_client: IAudioClient = match capture_device.Activate(CLSCTX_ALL, None) {
                Ok(c) => c,
                Err(_) => {
                    let _ = render_client.Stop();
                    CoTaskMemFree(Some(render_fmt_ptr as _));
                    if com_init.is_ok() {
                        CoUninitialize();
                    }
                    return;
                }
            };

            let capture_fmt_ptr: *mut WAVEFORMATEX = match capture_client.GetMixFormat() {
                Ok(f) => f,
                Err(_) => {
                    let _ = render_client.Stop();
                    CoTaskMemFree(Some(render_fmt_ptr as _));
                    if com_init.is_ok() {
                        CoUninitialize();
                    }
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
                if com_init.is_ok() {
                    CoUninitialize();
                }
                return;
            }

            let capture_svc: IAudioCaptureClient = match capture_client.GetService() {
                Ok(s) => s,
                Err(_) => {
                    let _ = render_client.Stop();
                    CoTaskMemFree(Some(render_fmt_ptr as _));
                    CoTaskMemFree(Some(capture_fmt_ptr as _));
                    if com_init.is_ok() {
                        CoUninitialize();
                    }
                    return;
                }
            };

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
                    if frames_read > 0 && !p_data.is_null() {
                        let sample_count = (frames_read as usize) * 2; // Stereo interleaved f32
                        if sample_count <= scratch_capacity {
                            if (flags & (AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)) != 0 {
                                input_scratch[..sample_count].fill(0.0);
                            } else {
                                let incoming = std::slice::from_raw_parts(
                                    p_data as *const f32,
                                    sample_count,
                                );
                                input_scratch[..sample_count].copy_from_slice(incoming);
                            }

                            // Process audio frames through DSP pipeline
                            // Non-blocking try_lock ensures audio thread never stalls on UI
                            if let Ok(mut coord) = coordinator.try_lock() {
                                coord.process_audio_chunk(
                                    &input_scratch[..sample_count],
                                    &mut output_scratch[..sample_count],
                                );
                            } else {
                                // Passthrough fallback if coordinator momentarily locked by telemetry
                                output_scratch[..sample_count]
                                    .copy_from_slice(&input_scratch[..sample_count]);
                            }

                            // Write processed audio frames to physical output DAC with retry loop
                            for _ in 0..10 {
                                let padding = render_client.GetCurrentPadding().unwrap_or(0);
                                let available = render_buffer_size.saturating_sub(padding);

                                if available >= frames_read {
                                    if let Ok(render_ptr) = render_svc.GetBuffer(frames_read) {
                                        let render_slice = std::slice::from_raw_parts_mut(
                                            render_ptr as *mut f32,
                                            sample_count,
                                        );
                                        render_slice.copy_from_slice(&output_scratch[..sample_count]);
                                        let _ = render_svc.ReleaseBuffer(frames_read, 0);
                                    }
                                    break;
                                }
                                std::thread::sleep(Duration::from_micros(500));
                            }
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
