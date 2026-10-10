//! Audio endpoint enumeration and device management via Windows MMDeviceAPI.

use open_boom_core::types::{AudioDeviceInfo, EngineError, EngineResult};

/// Manager for discovering and enumerating Windows audio render endpoints.
#[derive(Debug, Clone, Default)]
pub struct AudioDeviceManager {
    allow_synthetic_fallback: bool,
    force_synthetic: bool,
    include_synthetic_virtual: bool,
}

impl AudioDeviceManager {
    /// Creates a new device manager with real hardware discovery enabled.
    pub fn new() -> Self {
        Self {
            allow_synthetic_fallback: false,
            force_synthetic: false,
            include_synthetic_virtual: false,
        }
    }

    /// Creates a device manager that allows synthetic fallback when no audio hardware is present.
    pub fn with_synthetic_fallback(allow_synthetic_fallback: bool) -> Self {
        Self {
            allow_synthetic_fallback,
            force_synthetic: false,
            include_synthetic_virtual: false,
        }
    }

    /// Creates a device manager that forces synthetic devices for testing and headless runs.
    pub fn with_forced_synthetic(force_synthetic: bool) -> Self {
        Self {
            allow_synthetic_fallback: true,
            force_synthetic,
            include_synthetic_virtual: false,
        }
    }

    /// Creates a device manager that forces synthetic devices including a synthetic virtual endpoint.
    pub fn with_forced_synthetic_and_virtual(force_synthetic: bool, include_virtual: bool) -> Self {
        Self {
            allow_synthetic_fallback: true,
            force_synthetic,
            include_synthetic_virtual: include_virtual,
        }
    }

    /// Sets whether synthetic enumeration includes a virtual audio sink.
    pub fn set_include_synthetic_virtual(&mut self, include_virtual: bool) {
        self.include_synthetic_virtual = include_virtual;
    }

    /// Returns a synthetic virtual device descriptor for testing.
    pub fn synthetic_virtual_device() -> AudioDeviceInfo {
        AudioDeviceInfo::new(
            "synthetic_boomvad",
            "Boom Audio Virtual Device",
            false,
        )
    }

    /// Returns a list of synthetic fallback audio devices for testing and headless environments.
    pub fn synthetic_devices() -> Vec<AudioDeviceInfo> {
        vec![
            AudioDeviceInfo::new(
                "synthetic_default",
                "Headphones (Synthetic Audio)",
                true,
            ),
            AudioDeviceInfo::new(
                "synthetic_speakers",
                "Speakers (Synthetic Audio)",
                false,
            ),
        ]
    }

    /// Enumerates all active audio render endpoints on the system.
    pub fn enumerate_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        if self.force_synthetic {
            let mut devices = Self::synthetic_devices();
            if self.include_synthetic_virtual {
                devices.push(Self::synthetic_virtual_device());
            }
            return Ok(devices);
        }

        #[cfg(windows)]
        {
            match self.enumerate_windows_devices() {
                Ok(devices) if !devices.is_empty() => Ok(devices),
                Ok(_) if self.allow_synthetic_fallback => Ok(self.synthetic_device_list()),
                Ok(devices) => Ok(devices),
                Err(err) => {
                    if self.allow_synthetic_fallback {
                        Ok(self.synthetic_device_list())
                    } else {
                        Err(err)
                    }
                }
            }
        }
        #[cfg(not(windows))]
        {
            if self.allow_synthetic_fallback {
                Ok(self.synthetic_device_list())
            } else {
                Err(EngineError::DeviceInitialization(
                    "WASAPI device enumeration requires Windows platform".to_string(),
                ))
            }
        }
    }

    fn synthetic_device_list(&self) -> Vec<AudioDeviceInfo> {
        let mut devices = Self::synthetic_devices();
        if self.include_synthetic_virtual {
            devices.push(Self::synthetic_virtual_device());
        }
        devices
    }

    /// Checks whether the given endpoint friendly name corresponds to a known
    /// virtual audio adapter or cable sink (e.g. boomvad, VB-Audio Cable, VAC).
    pub fn is_virtual_device_name(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.contains("boom")
            || lower.contains("vb-audio")
            || lower.contains("cable")
            || lower.contains("voicemeeter")
            || lower.contains("vac")
            || lower.contains("virtual audio")
            || lower.contains("virtual cable")
            || lower.contains("openboom")
    }

    /// Finds an active virtual audio device endpoint on the system, if present.
    pub fn find_virtual_device(&self) -> EngineResult<Option<AudioDeviceInfo>> {
        let devices = self.enumerate_devices()?;
        Ok(devices.into_iter().find(|d| Self::is_virtual_device_name(&d.name)))
    }

    /// Returns the default console audio render endpoint.
    pub fn get_default_device(&self) -> EngineResult<Option<AudioDeviceInfo>> {
        let devices = self.enumerate_devices()?;
        Ok(devices.into_iter().find(|d| d.is_default))
    }

    /// Finds an audio render endpoint by its unique device identifier.
    pub fn get_device_by_id(&self, id: &str) -> EngineResult<Option<AudioDeviceInfo>> {
        let devices = self.enumerate_devices()?;
        Ok(devices.into_iter().find(|d| d.id == id))
    }

    #[cfg(windows)]
    fn enumerate_windows_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
        };
        use windows::Win32::Media::Audio::{
            eConsole, eRender, DEVICE_STATE_ACTIVE, IMMDevice, IMMDeviceCollection,
            IMMDeviceEnumerator, MMDeviceEnumerator,
        };
        use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }

        let enumerator: IMMDeviceEnumerator = unsafe {
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| {
                EngineError::DeviceInitialization(format!(
                    "Failed to create MMDeviceEnumerator: {}",
                    e
                ))
            })?
        };

        // Determine default device ID to set is_default flag
        let default_id: Option<String> = unsafe {
            enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .ok()
                .and_then(|dev| dev.GetId().ok())
                .map(|p| p.to_string().unwrap_or_default())
        };

        let collection: IMMDeviceCollection = unsafe {
            enumerator
                .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
                .map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to enumerate active audio endpoints: {}",
                        e
                    ))
                })?
        };

        let count = unsafe {
            collection.GetCount().map_err(|e| {
                EngineError::DeviceInitialization(format!(
                    "Failed to get audio endpoint count: {}",
                    e
                ))
            })?
        };

        let mut devices = Vec::with_capacity(count as usize);

        for i in 0..count {
            let device: IMMDevice = match unsafe { collection.Item(i) } {
                Ok(dev) => dev,
                Err(_) => continue,
            };

            let id_str = match unsafe { device.GetId() } {
                Ok(pwstr) => unsafe { pwstr.to_string().unwrap_or_default() },
                Err(_) => continue,
            };

            let is_default = default_id.as_ref().is_some_and(|def| def == &id_str);

            let friendly_name = unsafe {
                match device.OpenPropertyStore(STGM_READ) {
                    Ok(props) => {
                        match props.GetValue(&PKEY_Device_FriendlyName) {
                            Ok(prop_var) => {
                                prop_var.to_string()
                            }
                            Err(_) => id_str.clone(),
                        }
                    }
                    Err(_) => id_str.clone(),
                }
            };

            let clean_name = if friendly_name.trim().is_empty() {
                format!("Audio Device {}", i + 1)
            } else {
                friendly_name
            };

            devices.push(AudioDeviceInfo::new(id_str, clean_name, is_default));
        }

        Ok(devices)
    }
}
