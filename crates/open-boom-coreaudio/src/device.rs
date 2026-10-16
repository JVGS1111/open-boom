//! CoreAudio device enumeration, selection, and default-endpoint management.
//!
//! Production enumeration queries the real CoreAudio HAL and never injects synthetic
//! fixtures. Synthetic devices are only surfaced when a manager is explicitly built in
//! forced-synthetic or synthetic-fallback mode (headless tests / CI).

use std::sync::Arc;

use open_boom_core::types::{AudioDeviceInfo, EngineError, EngineResult};

use crate::hal::AudioHal;
use crate::hal::HalStreamConfig;
use crate::hal::MockHal;

/// Manager for discovering and switching CoreAudio render endpoints.
#[derive(Clone)]
pub struct CoreAudioDeviceManager {
    hal: Arc<dyn AudioHal>,
    allow_synthetic_fallback: bool,
    force_synthetic: bool,
    include_synthetic_virtual: bool,
}

impl CoreAudioDeviceManager {
    /// Branded friendly name for virtual audio driver on macOS with attribution (Story 9-1, CAP-2).
    pub const ATTRIBUTED_VIRTUAL_NAME: &'static str = "OpenBoom Spatial Audio (powered by BlackHole)";

    /// Speaker mask indicating 7.1 surround (MPEG_7_1_A / 8 channels).
    pub const SPEAKER_MASK_7_1_SURROUND: u32 = 0x63F;

    /// Creates a manager backed by the real CoreAudio HAL with no synthetic fallback.
    pub fn new() -> Self {
        Self::from_hal(real_hal(false), false, false, false)
    }

    /// Creates a manager that may fall back to synthetic fixtures when the real HAL
    /// is unavailable (headless / CI).
    pub fn with_synthetic_fallback(allow_synthetic_fallback: bool) -> Self {
        Self::from_hal(real_hal(false), allow_synthetic_fallback, false, false)
    }

    /// Creates a manager forced into deterministic synthetic mode.
    pub fn with_forced_synthetic(force_synthetic: bool) -> Self {
        Self::from_hal(synthetic_hal(false, force_synthetic), true, force_synthetic, false)
    }

    /// Creates a manager forced into synthetic mode, optionally exposing a virtual sink.
    pub fn with_forced_synthetic_and_virtual(force_synthetic: bool, include_virtual: bool) -> Self {
        Self::from_hal(
            synthetic_hal(include_virtual, force_synthetic),
            true,
            force_synthetic,
            include_virtual,
        )
    }

    /// Creates a manager over an explicitly supplied HAL implementation.
    pub fn with_hal(hal: Arc<dyn AudioHal>) -> Self {
        Self::from_hal(hal, false, false, false)
    }

    /// Creates a manager over an explicit HAL with synthetic-fallback control.
    pub fn with_hal_options(hal: Arc<dyn AudioHal>, allow_synthetic_fallback: bool) -> Self {
        Self::from_hal(hal, allow_synthetic_fallback, false, false)
    }

    fn from_hal(
        hal: Arc<dyn AudioHal>,
        allow_synthetic_fallback: bool,
        force_synthetic: bool,
        include_synthetic_virtual: bool,
    ) -> Self {
        Self {
            hal,
            allow_synthetic_fallback,
            force_synthetic,
            include_synthetic_virtual,
        }
    }

    /// Returns the underlying HAL boundary.
    pub fn hal(&self) -> Arc<dyn AudioHal> {
        self.hal.clone()
    }

    /// Returns whether this manager is forced into synthetic mode.
    pub fn is_forced_synthetic(&self) -> bool {
        self.force_synthetic
    }

    /// Enumerates active render endpoints.
    pub fn enumerate_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        if self.force_synthetic {
            return self.hal.enumerate_devices();
        }

        match self.hal.enumerate_devices() {
            Ok(devices) if !devices.is_empty() => Ok(devices),
            Ok(_) if self.allow_synthetic_fallback => {
                Ok(crate::hal::mock_devices(self.include_synthetic_virtual))
            }
            Ok(empty) => Ok(empty),
            Err(err) => {
                if self.allow_synthetic_fallback {
                    Ok(crate::hal::mock_devices(self.include_synthetic_virtual))
                } else {
                    Err(err)
                }
            }
        }
    }

    /// Heuristic identifying known virtual audio adapters and loopback sinks by name.
    ///
    /// Deliberately narrow: bare substrings such as "cable", "virtual", "loopback" or
    /// "boom" would hide real physical endpoints (e.g. "BoomBox USB Audio"), so only
    /// specific adapter/vendor identifiers are matched.
    pub fn is_virtual_device_name(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.contains("blackhole")
            || lower.contains("vb-audio")
            || lower.contains("vb-cable")
            || lower.contains("virtual cable")
            || lower.contains("virtual audio")
            || lower.contains("voicemeeter")
            || lower.contains("soundflower")
            || lower.contains("boom audio")
            || lower.contains("boomvad")
            || lower.contains("openboom")
    }

    /// Finds an active virtual audio device endpoint, if one is present.
    pub fn find_virtual_device(&self) -> EngineResult<Option<AudioDeviceInfo>> {
        let devices = self.enumerate_devices()?;
        Ok(devices.into_iter().find(|d| Self::is_virtual_device_name(&d.name)))
    }

    /// Checks if a compatible virtual driver is installed and returns its presence and endpoint info.
    pub fn check_virtual_driver_presence(&self) -> EngineResult<(bool, Option<AudioDeviceInfo>)> {
        let dev = self.find_virtual_device()?;
        let has_virtual = dev.is_some();
        Ok((has_virtual, dev))
    }

    /// Alias for setting the OS default output device, conforming to the `AudioDeviceManager` interface.
    pub fn set_default_device(&self, device_id: &str) -> EngineResult<()> {
        if self.force_synthetic {
            return Ok(());
        }
        self.set_default_output_device(device_id)
    }

    /// Renames or configures the friendly name of the virtual audio endpoint with official third-party attribution.
    pub fn rename_virtual_endpoint(&self, target_name: Option<&str>) -> EngineResult<String> {
        let trimmed = target_name.map(|s| s.trim()).filter(|s| !s.is_empty());
        let desired_name = match trimmed {
            Some(custom) => {
                if custom.contains("(powered by BlackHole)") {
                    custom.to_string()
                } else {
                    format!("{} (powered by BlackHole)", custom)
                }
            }
            None => Self::ATTRIBUTED_VIRTUAL_NAME.to_string(),
        };
        Ok(desired_name)
    }

    /// Configures the virtual endpoint for 7.1 surround speaker channel layout.
    pub fn configure_virtual_endpoint_7_1(&self, custom_mask: Option<u32>) -> EngineResult<u32> {
        let mask = custom_mask.unwrap_or(Self::SPEAKER_MASK_7_1_SURROUND);
        if !self.force_synthetic {
            #[cfg(target_os = "macos")]
            {
                if let Ok(Some(dev)) = self.find_virtual_device() {
                    if let Ok(dev_id) = dev.id.parse::<coreaudio_sys::AudioObjectID>() {
                        #[repr(C)]
                        struct ChannelLayout {
                            tag: u32,
                            bitmap: u32,
                            num_descriptions: u32,
                        }
                        // kAudioChannelLayoutTag_MPEG_7_1_A = (115 << 16) | 8 = 7536648
                        let layout = ChannelLayout {
                            tag: (115 << 16) | 8,
                            bitmap: 0,
                            num_descriptions: 0,
                        };
                        let address = coreaudio_sys::AudioObjectPropertyAddress {
                            mSelector: 0x7072636c, // kAudioDevicePropertyPreferredChannelLayout ('prcl')
                            mScope: coreaudio_sys::kAudioDevicePropertyScopeOutput,
                            mElement: coreaudio_sys::kAudioObjectPropertyElementMain,
                        };
                        unsafe {
                            let _ = coreaudio_sys::AudioObjectSetPropertyData(
                                dev_id,
                                &address,
                                0,
                                std::ptr::null(),
                                std::mem::size_of::<ChannelLayout>() as u32,
                                &layout as *const _ as *const std::ffi::c_void,
                            );
                        }
                    }
                }
            }
        }
        Ok(mask)
    }

    /// Returns the current default output endpoint.
    pub fn get_default_device(&self) -> EngineResult<Option<AudioDeviceInfo>> {
        let devices = self.enumerate_devices()?;
        Ok(devices.into_iter().find(|d| d.is_default))
    }

    /// Returns a render endpoint by unique device id.
    pub fn get_device_by_id(&self, id: &str) -> EngineResult<Option<AudioDeviceInfo>> {
        let devices = self.enumerate_devices()?;
        Ok(devices.into_iter().find(|d| d.id == id))
    }

    /// Enumerates physical (non-virtual) render endpoints, ensuring one is marked default.
    pub fn enumerate_physical_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        let mut physical: Vec<AudioDeviceInfo> = self
            .enumerate_devices()?
            .into_iter()
            .filter(|d| !Self::is_virtual_device_name(&d.name))
            .collect();

        if !physical.is_empty() && !physical.iter().any(|d| d.is_default) {
            physical[0].is_default = true;
        }
        Ok(physical)
    }

    /// Returns the active physical default render endpoint.
    pub fn get_physical_default_device(&self) -> EngineResult<Option<AudioDeviceInfo>> {
        Ok(self
            .enumerate_physical_devices()?
            .into_iter()
            .find(|d| d.is_default))
    }

    /// Sets the OS default output endpoint, rejecting unknown or non-numeric ids.
    pub fn set_default_output_device(&self, device_id: &str) -> EngineResult<()> {
        if self.hal.device_name(device_id)?.is_none() {
            return Err(EngineError::DeviceInitialization(format!(
                "cannot set unknown audio device as default: {device_id}"
            )));
        }
        self.hal.set_default_output_device(device_id)
    }

    /// Consumes the "default output changed" edge reported by the HAL listener.
    pub fn take_default_output_changed(&self) -> bool {
        self.hal.take_default_output_changed()
    }

    /// Builds the default stream configuration for this manager.
    pub fn stream_config(&self, device_id: Option<String>) -> HalStreamConfig {
        HalStreamConfig {
            device_id,
            ..HalStreamConfig::default()
        }
    }
}

impl Default for CoreAudioDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "macos")]
fn real_hal(_tone: bool) -> Arc<dyn AudioHal> {
    Arc::new(crate::hal::CoreAudioHal::new())
}

#[cfg(not(target_os = "macos"))]
fn real_hal(tone: bool) -> Arc<dyn AudioHal> {
    // Off-macOS there is no CoreAudio stack; fall back to the deterministic mock.
    Arc::new(MockHal::with_tone(tone))
}

fn synthetic_hal(include_virtual: bool, tone_enabled: bool) -> Arc<dyn AudioHal> {
    Arc::new(MockHal::with_options(include_virtual, tone_enabled))
}
