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

    /// Official third-party attribution name for the virtual audio endpoint (Story 9-1, CAP-1).
    pub const ATTRIBUTED_VIRTUAL_NAME: &'static str = "OpenBoom Spatial Audio (powered by VB-Cable)";

    /// Standard Microsoft 7.1 Surround speaker physical configuration mask (Story 9-2, CAP-2):
    /// FL (0x1) | FR (0x2) | FC (0x4) | LFE (0x8) | BL (0x10) | BR (0x20) | SL (0x200) | SR (0x400) = 0x63F (1599).
    pub const SPEAKER_MASK_7_1_SURROUND: u32 = 0x63F;

    /// Returns a synthetic virtual device descriptor for testing.
    pub fn synthetic_virtual_device() -> AudioDeviceInfo {
        AudioDeviceInfo::new(
            "synthetic_virtual_sink",
            Self::ATTRIBUTED_VIRTUAL_NAME,
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

    /// Checks if a compatible virtual driver is installed and returns its presence and endpoint info (Story 9-3, CAP-4).
    pub fn check_virtual_driver_presence(&self) -> EngineResult<(bool, Option<AudioDeviceInfo>)> {
        let dev = self.find_virtual_device()?;
        let has_virtual = dev.is_some();
        Ok((has_virtual, dev))
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

    /// Extracts the endpoint GUID `{xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}` from a WASAPI device ID.
    pub fn extract_endpoint_guid(device_id: &str) -> Option<String> {
        let start = device_id.rfind('{')?;
        let end = device_id[start..].find('}')?;
        let candidate = &device_id[start..=start + end];
        if candidate.len() == 38 {
            Some(candidate.to_string())
        } else {
            None
        }
    }

    /// Normalizes virtual endpoint friendly names to the compliant branded format with attribution.
    pub fn normalize_endpoint_friendly_name(friendly_name: &str) -> String {
        let trimmed = friendly_name.trim();
        if trimmed.is_empty() {
            "Audio Device".to_string()
        } else if Self::is_virtual_device_name(trimmed) && !trimmed.to_lowercase().contains("openboom") {
            Self::ATTRIBUTED_VIRTUAL_NAME.to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Renames or configures the friendly name of the virtual audio endpoint with official third-party attribution (Story 9-1).
    pub fn rename_virtual_endpoint(&self, target_name: Option<&str>) -> EngineResult<String> {
        let trimmed = target_name.map(|s| s.trim()).filter(|s| !s.is_empty());
        let desired_name = match trimmed {
            Some(custom) => {
                // Ensure the required third-party trademark attribution is preserved (CAP-1 constraint)
                if custom.contains("(powered by VB-Cable)") {
                    custom.to_string()
                } else {
                    format!("{} (powered by VB-Cable)", custom)
                }
            }
            None => Self::ATTRIBUTED_VIRTUAL_NAME.to_string(),
        };

        if self.force_synthetic {
            return Ok(desired_name);
        }

        #[cfg(windows)]
        {
            self.set_windows_endpoint_friendly_name(&desired_name)
        }
        #[cfg(not(windows))]
        {
            Ok(desired_name)
        }
    }

    /// Writes the attributed friendly name to Windows registry properties for the virtual endpoint.
    #[cfg(windows)]
    pub fn set_windows_endpoint_friendly_name(&self, name: &str) -> EngineResult<String> {
        let virtual_device = self.find_virtual_device()?;
        let endpoint = match virtual_device {
            Some(dev) => dev,
            None => {
                if self.allow_synthetic_fallback {
                    return Ok(name.to_string());
                }
                return Err(EngineError::DeviceInitialization(
                    "No virtual audio device endpoint found to rename".to_string(),
                ));
            }
        };

        let guid = match Self::extract_endpoint_guid(&endpoint.id) {
            Some(g) => g,
            None => {
                if self.allow_synthetic_fallback {
                    return Ok(name.to_string());
                }
                return Err(EngineError::DeviceInitialization(format!(
                    "Invalid virtual device ID format: {}",
                    endpoint.id
                )));
            }
        };

        let reg_path = format!(
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render\{}\Properties",
            guid
        );

        let mut needs_elev = false;
        for pid in ["14", "2"] {
            let mut cmd = std::process::Command::new("reg");
            cmd.args([
                "add",
                &reg_path,
                "/v",
                &format!("{{a45c254e-df1c-4efd-8020-67d146a850e0}},{}", pid),
                "/t",
                "REG_SZ",
                "/d",
                name,
                "/f",
            ]);
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            if let Ok(out) = cmd.output() {
                if !out.status.success() {
                    needs_elev = true;
                }
            } else {
                needs_elev = true;
            }
        }

        if needs_elev {
            let ps_cmd = format!(
                "Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\MMDevices\\Audio\\Render\\{}\\Properties' -Name '{{a45c254e-df1c-4efd-8020-67d146a850e0}},2' -Value '{}' -Type String -Force; Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\MMDevices\\Audio\\Render\\{}\\Properties' -Name '{{a45c254e-df1c-4efd-8020-67d146a850e0}},14' -Value '{}' -Type String -Force; Restart-Service audiosrv -Force -ErrorAction SilentlyContinue",
                guid, name, guid, name
            );
            let mut elev_cmd = std::process::Command::new("powershell");
            elev_cmd.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("Start-Process powershell -ArgumentList '-NoProfile -Command \"{}\"' -Verb RunAs -Wait", ps_cmd),
            ]);
            use std::os::windows::process::CommandExt;
            elev_cmd.creation_flags(0x08000000);
            let _ = elev_cmd.output();
        }

        Ok(name.to_string())
    }

    /// Injects the 7.1 Surround physical speaker mask (0x63F) into the virtual endpoint (Story 9-2, CAP-2).
    pub fn configure_virtual_endpoint_7_1(&self, custom_mask: Option<u32>) -> EngineResult<u32> {
        let mask = custom_mask.unwrap_or(Self::SPEAKER_MASK_7_1_SURROUND);
        if self.force_synthetic {
            return Ok(mask);
        }

        #[cfg(windows)]
        {
            self.set_windows_endpoint_speaker_mask(mask)
        }
        #[cfg(not(windows))]
        {
            Ok(mask)
        }
    }

    /// Queries the physical speaker mask from the virtual endpoint property store.
    pub fn get_virtual_endpoint_speaker_mask(&self) -> EngineResult<u32> {
        if self.force_synthetic {
            return Ok(Self::SPEAKER_MASK_7_1_SURROUND);
        }

        #[cfg(windows)]
        {
            self.get_windows_endpoint_speaker_mask()
        }
        #[cfg(not(windows))]
        {
            Ok(Self::SPEAKER_MASK_7_1_SURROUND)
        }
    }

    /// Writes PKEY_AudioEndpoint_PhysicalSpeakers = mask ({1da5d803-d346-45ab-9f40-8c2f1083a47e},3) as DWORD into Windows registry.
    #[cfg(windows)]
    pub fn set_windows_endpoint_speaker_mask(&self, mask: u32) -> EngineResult<u32> {
        let virtual_device = self.find_virtual_device()?;
        let endpoint = match virtual_device {
            Some(dev) => dev,
            None => {
                if self.allow_synthetic_fallback {
                    return Ok(mask);
                }
                return Err(EngineError::DeviceInitialization(
                    "No virtual audio device endpoint found to configure speaker mask".to_string(),
                ));
            }
        };

        let guid = match Self::extract_endpoint_guid(&endpoint.id) {
            Some(g) => g,
            None => {
                if self.allow_synthetic_fallback {
                    return Ok(mask);
                }
                return Err(EngineError::DeviceInitialization(format!(
                    "Invalid virtual device ID format: {}",
                    endpoint.id
                )));
            }
        };

        let reg_path = format!(
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render\{}\Properties",
            guid
        );

        let mut cmd = std::process::Command::new("reg");
        cmd.args([
            "add",
            &reg_path,
            "/v",
            "{1da5d803-d346-45ab-9f40-8c2f1083a47e},3",
            "/t",
            "REG_DWORD",
            "/d",
            &format!("{}", mask),
            "/f",
        ]);
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let needs_elev = if let Ok(out) = cmd.output() {
            !out.status.success()
        } else {
            true
        };

        if needs_elev {
            let ps_cmd = format!(
                "Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\MMDevices\\Audio\\Render\\{}\\Properties' -Name '{{1da5d803-d346-45ab-9f40-8c2f1083a47e}},3' -Value {} -Type DWord -Force; Restart-Service audiosrv -Force -ErrorAction SilentlyContinue",
                guid, mask
            );
            let mut elev_cmd = std::process::Command::new("powershell");
            elev_cmd.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("Start-Process powershell -ArgumentList '-NoProfile -Command \"{}\"' -Verb RunAs -Wait", ps_cmd),
            ]);
            use std::os::windows::process::CommandExt;
            elev_cmd.creation_flags(0x08000000);
            let _ = elev_cmd.output();
        }

        Ok(mask)
    }

    /// Reads PKEY_AudioEndpoint_PhysicalSpeakers from the registry or returns default.
    #[cfg(windows)]
    pub fn get_windows_endpoint_speaker_mask(&self) -> EngineResult<u32> {
        let virtual_device = self.find_virtual_device()?;
        let endpoint = match virtual_device {
            Some(dev) => dev,
            None => {
                if self.allow_synthetic_fallback {
                    return Ok(Self::SPEAKER_MASK_7_1_SURROUND);
                }
                return Err(EngineError::DeviceInitialization(
                    "No virtual audio device endpoint found to query speaker mask".to_string(),
                ));
            }
        };

        let guid = match Self::extract_endpoint_guid(&endpoint.id) {
            Some(g) => g,
            None => {
                if self.allow_synthetic_fallback {
                    return Ok(Self::SPEAKER_MASK_7_1_SURROUND);
                }
                return Err(EngineError::DeviceInitialization(format!(
                    "Invalid virtual device ID format: {}",
                    endpoint.id
                )));
            }
        };

        let reg_path = format!(
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render\{}\Properties",
            guid
        );

        let mut cmd = std::process::Command::new("reg");
        cmd.args([
            "query",
            &reg_path,
            "/v",
            "{1da5d803-d346-45ab-9f40-8c2f1083a47e},3",
        ]);
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("{1da5d803-d346-45ab-9f40-8c2f1083a47e},3") {
                        if let Some(val_str) = line.split_whitespace().last() {
                            if let Ok(val) = u32::from_str_radix(val_str.trim_start_matches("0x"), 16) {
                                return Ok(val);
                            } else if let Ok(val) = val_str.parse::<u32>() {
                                return Ok(val);
                            }
                        }
                    }
                }
            }
        }

        Ok(Self::SPEAKER_MASK_7_1_SURROUND)
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
        use windows::Win32::Media::Audio::{eCommunications, eConsole, eMultimedia, ERole};

        const CLSID_POLICY_CONFIG: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);
        const IID_IPOLICY_CONFIG: GUID = GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8);

        let com_init = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

        let result = unsafe {
            (|| -> EngineResult<()> {
                let unk: IUnknown = CoCreateInstance(&CLSID_POLICY_CONFIG, None, CLSCTX_ALL)
                    .map_err(|e| {
                        EngineError::DeviceInitialization(format!(
                            "Failed to create PolicyConfig: {}",
                            e
                        ))
                    })?;

                let mut policy_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                let qi_hr = unk.query(&IID_IPOLICY_CONFIG, &mut policy_ptr);
                if qi_hr.is_err() || policy_ptr.is_null() {
                    return Err(EngineError::DeviceInitialization(
                        "Failed to query IPolicyConfig interface".to_string(),
                    ));
                }

                let _policy_guard = IUnknown::from_raw(policy_ptr);
                let vtable_ptr = *(policy_ptr as *const *const usize);
                let set_default_func_ptr = *vtable_ptr.add(13);
                let set_default_func: unsafe extern "system" fn(
                    this: *mut std::ffi::c_void,
                    wsz_device_id: PCWSTR,
                    role: ERole,
                ) -> HRESULT = std::mem::transmute(set_default_func_ptr);

                let wide_id: Vec<u16> =
                    device_id.encode_utf16().chain(std::iter::once(0)).collect();
                let pcwstr = PCWSTR::from_raw(wide_id.as_ptr());

                let hr_console = set_default_func(policy_ptr, pcwstr, eConsole);
                let hr_multi = set_default_func(policy_ptr, pcwstr, eMultimedia);
                let _hr_comm = set_default_func(policy_ptr, pcwstr, eCommunications);

                if hr_console.is_err() || hr_multi.is_err() {
                    return Err(EngineError::DeviceInitialization(format!(
                        "SetDefaultEndpoint failed: {:?}, {:?}",
                        hr_console, hr_multi
                    )));
                }

                Ok(())
            })()
        };

        if com_init.is_ok() {
            unsafe {
                CoUninitialize();
            }
        }

        result
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

                let clean_name = Self::normalize_endpoint_friendly_name(&friendly_name);

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
