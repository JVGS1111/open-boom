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

    /// Enumerates only physical audio render endpoints, filtering out any virtual devices/sinks.
    pub fn enumerate_physical_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        let all_devices = self.enumerate_devices()?;
        let mut physical: Vec<AudioDeviceInfo> = all_devices
            .into_iter()
            .filter(|d| !Self::is_virtual_device_name(&d.name))
            .collect();

        // If no physical device is marked default (e.g. because Windows default is currently a virtual sink),
        // mark the first physical device as default.
        if !physical.is_empty() && !physical.iter().any(|d| d.is_default) {
            physical[0].is_default = true;
        }

        Ok(physical)
    }

    /// Returns the active physical default audio render endpoint.
    pub fn get_physical_default_device(&self) -> EngineResult<Option<AudioDeviceInfo>> {
        let physical = self.enumerate_physical_devices()?;
        Ok(physical.into_iter().find(|d| d.is_default))
    }

    /// Sets the Windows default audio playback endpoint (eConsole and eMultimedia roles).
    pub fn set_default_device(&self, device_id: &str) -> EngineResult<()> {
        if self.force_synthetic {
            return Ok(());
        }

        #[cfg(windows)]
        {
            Self::set_windows_default_device(device_id)
        }
        #[cfg(not(windows))]
        {
            let _ = device_id;
            Ok(())
        }
    }

    /// Sets the Windows default audio playback endpoint via COM IPolicyConfig on an isolated thread.
    #[cfg(windows)]
    pub fn set_windows_default_device(device_id: &str) -> EngineResult<()> {
        let id_string = device_id.to_string();
        std::thread::Builder::new()
            .name("open-boom-set-default-device".to_string())
            .spawn(move || Self::set_windows_default_device_worker(&id_string))
            .map_err(|e| {
                EngineError::DeviceInitialization(format!(
                    "Failed to spawn set-default-device worker thread: {}",
                    e
                ))
            })?
            .join()
            .map_err(|_| {
                EngineError::DeviceInitialization(
                    "Set-default-device worker thread panicked".to_string(),
                )
            })?
    }

    #[cfg(windows)]
    fn set_windows_default_device_worker(device_id: &str) -> EngineResult<()> {
        use windows::core::{Interface, IUnknown, GUID, HRESULT, PCWSTR};
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
        };
        use windows::Win32::Media::Audio::ERole;

        const CLSID_POLICY_CONFIG: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);
        const IID_IPOLICY_CONFIG: GUID = GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8);

        unsafe {
            let com_init = CoInitializeEx(None, COINIT_MULTITHREADED);
            let unk: IUnknown = CoCreateInstance(&CLSID_POLICY_CONFIG, None, CLSCTX_ALL)
                .map_err(|e| EngineError::DeviceInitialization(format!("Failed to create PolicyConfig: {}", e)))?;

            let mut policy_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let qi_hr = unk.query(&IID_IPOLICY_CONFIG, &mut policy_ptr);
            if qi_hr.is_err() || policy_ptr.is_null() {
                if com_init.is_ok() {
                    CoUninitialize();
                }
                return Err(EngineError::DeviceInitialization("Failed to query IPolicyConfig interface".to_string()));
            }

            let _policy_guard = IUnknown::from_raw(policy_ptr);
            let vtable_ptr = *(policy_ptr as *const *const usize);
            let set_default_func_ptr = *vtable_ptr.add(13);
            let set_default_func: unsafe extern "system" fn(
                this: *mut std::ffi::c_void,
                wsz_device_id: PCWSTR,
                role: ERole,
            ) -> HRESULT = std::mem::transmute(set_default_func_ptr);

            let wide_id: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
            let pcwstr = PCWSTR::from_raw(wide_id.as_ptr());

            let hr_console = set_default_func(policy_ptr, pcwstr, windows::Win32::Media::Audio::eConsole);
            let hr_multi = set_default_func(policy_ptr, pcwstr, windows::Win32::Media::Audio::eMultimedia);

            if com_init.is_ok() {
                CoUninitialize();
            }

            if hr_console.is_err() || hr_multi.is_err() {
                return Err(EngineError::DeviceInitialization(format!(
                    "SetDefaultEndpoint failed: {:?}, {:?}",
                    hr_console, hr_multi
                )));
            }

            Ok(())
        }
    }

    #[cfg(windows)]
    fn enumerate_windows_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        // Run Windows Core Audio MMDeviceAPI enumeration on an isolated COM worker thread.
        // If run directly on the caller thread, initializing COINIT_MULTITHREADED permanently
        // sets the thread to MTA mode. When the caller is the main GUI thread, Tao's
        // subsequent OleInitialize call will fail with RPC_E_CHANGED_MODE and panic.
        std::thread::Builder::new()
            .name("open-boom-device-enum".to_string())
            .spawn(|| Self::enumerate_windows_devices_isolated())
            .map_err(|e| {
                EngineError::DeviceInitialization(format!(
                    "Failed to spawn device enumeration worker thread: {}",
                    e
                ))
            })?
            .join()
            .map_err(|_| {
                EngineError::DeviceInitialization(
                    "Device enumeration worker thread panicked".to_string(),
                )
            })?
    }

    #[cfg(windows)]
    fn enumerate_windows_devices_isolated() -> EngineResult<Vec<AudioDeviceInfo>> {
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
            STGM_READ,
        };
        use windows::Win32::Media::Audio::{
            eConsole, eRender, DEVICE_STATE_ACTIVE, IMMDevice, IMMDeviceCollection,
            IMMDeviceEnumerator, MMDeviceEnumerator,
        };
        use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;

        let com_init = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

        let result = (|| -> EngineResult<Vec<AudioDeviceInfo>> {
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
                        Ok(props) => match props.GetValue(&PKEY_Device_FriendlyName) {
                            Ok(prop_var) => prop_var.to_string(),
                            Err(_) => id_str.clone(),
                        },
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
        })();

        if com_init.is_ok() {
            unsafe {
                CoUninitialize();
            }
        }

        result
    }
}
