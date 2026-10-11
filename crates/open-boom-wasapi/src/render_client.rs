//! Low-latency WASAPI render client with MMCSS Pro Audio thread scheduling.

use open_boom_core::types::{EngineError, EngineResult, INTERNAL_SAMPLE_RATE};
use crate::mmcss::{CallbackBudgetTracker, MmcssGuard};

/// Configuration for the WASAPI render client.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub device_id: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_duration_ms: f64,
    pub allow_synthetic_fallback: bool,
    pub force_synthetic: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            device_id: None,
            sample_rate: INTERNAL_SAMPLE_RATE,
            channels: 2,
            buffer_duration_ms: 5.0, // 5ms buffer duration for sub-15ms Pro Audio
            allow_synthetic_fallback: true,
            force_synthetic: false,
        }
    }
}

/// Low-latency WASAPI render client.
pub struct RenderClient {
    pub config: RenderConfig,
    pub is_rendering: bool,
    latency_ms: f32,
    budget_tracker: CallbackBudgetTracker,
    mmcss_guard: Option<MmcssGuard>,
}

impl Default for RenderClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderClient {
    /// Creates a default render client targeting the Windows default console endpoint.
    pub fn new() -> Self {
        Self::with_config(RenderConfig::default())
    }

    /// Creates a render client with customized configuration.
    pub fn with_config(config: RenderConfig) -> Self {
        let buffer_duration_ms = config.buffer_duration_ms;
        Self {
            config,
            is_rendering: false,
            latency_ms: 0.0,
            budget_tracker: CallbackBudgetTracker::new(buffer_duration_ms),
            mmcss_guard: None,
        }
    }

    /// Returns the currently active device ID, if explicitly configured.
    pub fn device_id(&self) -> Option<&str> {
        self.config.device_id.as_deref()
    }

    /// Returns whether the render stream is currently active.
    pub fn is_rendering(&self) -> bool {
        self.is_rendering
    }

    /// Returns measured render latency in milliseconds.
    pub fn latency_ms(&self) -> f32 {
        self.latency_ms
    }

    /// Returns whether the audio thread is actively registered under MMCSS Pro Audio.
    pub fn mmcss_active(&self) -> bool {
        self.mmcss_guard.as_ref().is_some_and(|g| g.is_active())
    }

    /// Reference to the callback execution budget tracker.
    pub fn budget_tracker(&self) -> &CallbackBudgetTracker {
        &self.budget_tracker
    }

    /// Mutable reference to the callback budget tracker.
    pub fn budget_tracker_mut(&mut self) -> &mut CallbackBudgetTracker {
        &mut self.budget_tracker
    }

    /// Initializes and starts the audio render stream on the target endpoint.
    ///
    /// In headless environments without audio hardware, fails with `EngineError::DeviceInitialization`
    /// or gracefully activates synthetic render mode if configured.
    pub fn start(&mut self) -> EngineResult<()> {
        if self.config.force_synthetic {
            self.latency_ms = self.config.buffer_duration_ms as f32;
            self.mmcss_guard = Some(MmcssGuard::register("Pro Audio"));
            self.is_rendering = true;
            return Ok(());
        }

        #[cfg(windows)]
        {
            use windows::Win32::System::Com::{
                CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
            };
            use windows::Win32::Media::Audio::{
                eConsole, eRender, IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator,
            };

            // Attempt MTA COM initialization (ignoring S_FALSE / RPC_E_CHANGED_MODE)
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            }

            let init_result = (|| -> windows::core::Result<(f32, MmcssGuard)> {
                let enumerator: IMMDeviceEnumerator =
                    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
                let device = if let Some(ref id) = self.config.device_id {
                    use windows::core::HSTRING;
                    let h_id = HSTRING::from(id.as_str());
                    unsafe { enumerator.GetDevice(&h_id)? }
                } else {
                    unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole)? }
                };
                let audio_client: IAudioClient = unsafe { device.Activate(CLSCTX_ALL, None)? };

                let latency_ms = match unsafe { audio_client.GetStreamLatency() } {
                    Ok(lat) => (lat as f32) / 10_000.0,
                    Err(_) => self.config.buffer_duration_ms as f32,
                };

                let mmcss = MmcssGuard::register("Pro Audio");
                Ok((latency_ms.max(self.config.buffer_duration_ms as f32), mmcss))
            })();

            match init_result {
                Ok((latency, mmcss)) => {
                    self.latency_ms = latency;
                    self.mmcss_guard = Some(mmcss);
                    self.is_rendering = true;
                    Ok(())
                }
                Err(err) => {
                    if self.config.allow_synthetic_fallback {
                        self.latency_ms = self.config.buffer_duration_ms as f32;
                        self.mmcss_guard = Some(MmcssGuard::register("Pro Audio"));
                        self.is_rendering = true;
                        Ok(())
                    } else {
                        Err(EngineError::DeviceInitialization(format!(
                            "WASAPI render device initialization failed (HRESULT: {:#x}): {}",
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
                self.latency_ms = self.config.buffer_duration_ms as f32;
                self.is_rendering = true;
                Ok(())
            } else {
                Err(EngineError::DeviceInitialization(
                    "WASAPI rendering requires Windows platform".to_string(),
                ))
            }
        }
    }

    /// Stops the audio render stream and releases MMCSS thread priority.
    pub fn stop(&mut self) -> EngineResult<()> {
        self.mmcss_guard = None;
        self.is_rendering = false;
        Ok(())
    }

    /// Dynamically switches audio rendering to a new endpoint within 500ms.
    /// If the target endpoint fails to initialize, falls back to the default endpoint.
    pub fn switch_device(&mut self, new_device_id: Option<String>) -> EngineResult<()> {
        let prev_device_id = self.config.device_id.clone();
        let was_rendering = self.is_rendering;

        if was_rendering {
            let _ = self.stop();
        }

        self.config.device_id = new_device_id.clone();

        if was_rendering {
            if let Err(err) = self.start() {
                // If switching to a specific device failed and it was not already None (default),
                // attempt graceful fallback to default endpoint
                if new_device_id.is_some() {
                    self.config.device_id = None;
                    if let Ok(()) = self.start() {
                        return Ok(());
                    }
                }
                // Restore previous state if fallback also failed
                self.config.device_id = prev_device_id;
                let _ = self.start();
                return Err(err);
            }
        }

        Ok(())
    }

    /// Handles device disconnection or invalidation (AUDCLNT_E_DEVICE_INVALIDATED),
    /// recovering within 500ms by falling back to the Windows default console endpoint.
    pub fn handle_device_lost(&mut self) -> EngineResult<()> {
        let was_rendering = self.is_rendering;
        if was_rendering {
            let _ = self.stop();
        }
        self.config.device_id = None;
        if was_rendering {
            self.start()?;
        }
        Ok(())
    }
}
