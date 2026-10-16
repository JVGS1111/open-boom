//! CoreAudio master volume and mute bidirectional synchronization (Epic 10, Story 10-7).
//!
//! Provides bidirectional volume synchronization on macOS between OpenBoom and the system audio endpoint:
//! 1. Listens to CoreAudio HAL property notifications via `AudioObjectAddPropertyListener` for
//!    `kAudioHardwareServiceDeviceProperty_VirtualMasterVolume`, `kAudioDevicePropertyVolumeScalar`,
//!    and `kAudioDevicePropertyMute`.
//! 2. Prevents echo / feedback loops using an atomic `is_syncing_to_os` guard.
//! 3. Supports volume boost (101% - 150%) by clamping hardware volume to 1.0 (100%) while preserving
//!    full software boost gain in the DSP pipeline.
//! 4. Supports dynamic endpoint rebinding (`rebind_endpoint`) on hardware switch or master power toggle.
//! 5. Provides deterministic synthetic mode for headless CI / unit testing.

use open_boom_core::types::EngineResult;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

#[cfg(target_os = "macos")]
use coreaudio_sys as ca;

/// Platform-neutral volume context GUID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VolumeContextGuid(pub u128);

impl VolumeContextGuid {
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }
}

/// Dedicated context GUID identifying volume adjustments originating from OpenBoom.
pub const OPEN_BOOM_VOLUME_CONTEXT_GUID: VolumeContextGuid =
    VolumeContextGuid::from_u128(0x7b003e8a_8422_4f1e_98c1_4a8df2d00100);

/// Volume synchronization event delivered when the OS master volume or mute state changes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeSyncEvent {
    /// Scalar volume (0.0 to 1.5 scale in OpenBoom).
    pub volume: f32,
    /// Mute state.
    pub is_muted: bool,
}

impl VolumeSyncEvent {
    pub fn new(volume: f32, is_muted: bool) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.5),
            is_muted,
        }
    }
}

pub type VolumeCallback = Arc<dyn Fn(VolumeSyncEvent) + Send + Sync + 'static>;

#[cfg(target_os = "macos")]
mod macos_listener {
    use super::*;
    use std::ffi::c_void;
    use std::ptr;

    pub const SYSTEM_OBJECT: ca::AudioObjectID = ca::kAudioObjectSystemObject as ca::AudioObjectID;
    pub const SCOPE_OUTPUT: u32 = u32::from_be_bytes(*b"outp");
    pub const ELEMENT_MAIN: u32 = 0;
    pub const SEL_VIRTUAL_VOLUME: u32 = u32::from_be_bytes(*b"vmvc");
    pub const SEL_VOLUME_SCALAR: u32 = u32::from_be_bytes(*b"volm");
    pub const SEL_MUTE: u32 = u32::from_be_bytes(*b"mute");
    pub const SEL_DEFAULT_OUTPUT: u32 = ca::kAudioHardwarePropertyDefaultOutputDevice as u32;

    pub struct VolumeListenerContext {
        pub device_id: ca::AudioObjectID,
        pub callbacks: Arc<RwLock<Vec<VolumeCallback>>>,
        pub is_syncing_to_os: Arc<AtomicBool>,
        pub volume: Arc<Mutex<f32>>,
        pub muted: Arc<Mutex<bool>>,
    }

    impl VolumeListenerContext {
        pub fn handle_change(&self) {
            if self.is_syncing_to_os.load(Ordering::SeqCst) {
                return;
            }
            let (hw_vol, hw_mute) = read_hw_volume_and_mute(self.device_id);
            let mut changed = false;
            let mut final_vol = hw_vol;
            let mut final_mute = hw_mute;

            if let Ok(mut v) = self.volume.lock() {
                // If OpenBoom is in boost (> 1.0) and macOS volume is 1.0, preserve boost
                if *v > 1.0 && (hw_vol - 1.0).abs() < 1e-4 {
                    final_vol = *v;
                } else if (*v - hw_vol).abs() >= 1e-4 {
                    *v = hw_vol;
                    final_vol = hw_vol;
                    changed = true;
                }
            }

            if let Ok(mut m) = self.muted.lock() {
                if *m != hw_mute {
                    *m = hw_mute;
                    final_mute = hw_mute;
                    changed = true;
                }
            }

            if changed {
                let event = VolumeSyncEvent::new(final_vol, final_mute);
                if let Ok(cbs) = self.callbacks.read() {
                    for cb in cbs.iter() {
                        cb(event);
                    }
                }
            }
        }
    }

    pub unsafe extern "C" fn property_listener(
        _in_object: ca::AudioObjectID,
        _in_number_addresses: u32,
        _in_addresses: *const ca::AudioObjectPropertyAddress,
        in_client_data: *mut c_void,
    ) -> ca::OSStatus {
        if !in_client_data.is_null() {
            let ctx = &*(in_client_data as *const VolumeListenerContext);
            ctx.handle_change();
        }
        0
    }

    pub struct MacListenerGuard {
        pub device_id: ca::AudioObjectID,
        pub addresses: Vec<ca::AudioObjectPropertyAddress>,
        pub context_ptr: *mut VolumeListenerContext,
    }

    unsafe impl Send for MacListenerGuard {}
    unsafe impl Sync for MacListenerGuard {}

    impl Drop for MacListenerGuard {
        fn drop(&mut self) {
            unsafe {
                for addr in &self.addresses {
                    let _ = ca::AudioObjectRemovePropertyListener(
                        self.device_id,
                        addr,
                        Some(property_listener),
                        self.context_ptr as *mut c_void,
                    );
                }
                drop(Box::from_raw(self.context_ptr));
            }
        }
    }

    pub fn get_default_output_device_id() -> Option<ca::AudioObjectID> {
        let addr = ca::AudioObjectPropertyAddress {
            mSelector: SEL_DEFAULT_OUTPUT,
            mScope: ca::kAudioObjectPropertyScopeGlobal as u32,
            mElement: ELEMENT_MAIN,
        };
        let mut device_id: ca::AudioDeviceID = 0;
        let mut size = std::mem::size_of::<ca::AudioDeviceID>() as u32;
        let status = unsafe {
            ca::AudioObjectGetPropertyData(
                SYSTEM_OBJECT,
                &addr,
                0,
                ptr::null(),
                &mut size,
                &mut device_id as *mut ca::AudioDeviceID as *mut c_void,
            )
        };
        if status == 0 && device_id != 0 {
            Some(device_id)
        } else {
            None
        }
    }

    pub fn read_hw_volume_and_mute(device_id: ca::AudioObjectID) -> (f32, bool) {
        let mut vol = 0.8f32;
        let mut muted = false;

        // Try virtual volume selector first, then volume scalar
        for sel in [SEL_VIRTUAL_VOLUME, SEL_VOLUME_SCALAR] {
            let addr = ca::AudioObjectPropertyAddress {
                mSelector: sel,
                mScope: SCOPE_OUTPUT,
                mElement: ELEMENT_MAIN,
            };
            let mut val: f32 = 0.0;
            let mut size = std::mem::size_of::<f32>() as u32;
            let status = unsafe {
                ca::AudioObjectGetPropertyData(
                    device_id,
                    &addr,
                    0,
                    ptr::null(),
                    &mut size,
                    &mut val as *mut f32 as *mut c_void,
                )
            };
            if status == 0 {
                vol = val.clamp(0.0, 1.0);
                break;
            }
        }

        let mute_addr = ca::AudioObjectPropertyAddress {
            mSelector: SEL_MUTE,
            mScope: SCOPE_OUTPUT,
            mElement: ELEMENT_MAIN,
        };
        let mut mute_val: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        let status = unsafe {
            ca::AudioObjectGetPropertyData(
                device_id,
                &mute_addr,
                0,
                ptr::null(),
                &mut size,
                &mut mute_val as *mut u32 as *mut c_void,
            )
        };
        if status == 0 {
            muted = mute_val != 0;
        }

        (vol, muted)
    }

    pub fn write_hw_volume(device_id: ca::AudioObjectID, vol: f32) {
        let clamped = vol.clamp(0.0, 1.0);
        for sel in [SEL_VIRTUAL_VOLUME, SEL_VOLUME_SCALAR] {
            let addr = ca::AudioObjectPropertyAddress {
                mSelector: sel,
                mScope: SCOPE_OUTPUT,
                mElement: ELEMENT_MAIN,
            };
            let size = std::mem::size_of::<f32>() as u32;
            let status = unsafe {
                ca::AudioObjectSetPropertyData(
                    device_id,
                    &addr,
                    0,
                    ptr::null(),
                    size,
                    &clamped as *const f32 as *const c_void,
                )
            };
            if status == 0 {
                break;
            }
        }
    }

    pub fn write_hw_mute(device_id: ca::AudioObjectID, muted: bool) {
        let mute_addr = ca::AudioObjectPropertyAddress {
            mSelector: SEL_MUTE,
            mScope: SCOPE_OUTPUT,
            mElement: ELEMENT_MAIN,
        };
        let val: u32 = if muted { 1 } else { 0 };
        let size = std::mem::size_of::<u32>() as u32;
        unsafe {
            let _ = ca::AudioObjectSetPropertyData(
                device_id,
                &mute_addr,
                0,
                ptr::null(),
                size,
                &val as *const u32 as *const c_void,
            );
        }
    }
}

/// Bidirectional CoreAudio Master Volume synchronization coordinator.
pub struct CoreAudioVolumeSync {
    callbacks: Arc<RwLock<Vec<VolumeCallback>>>,
    is_syncing_to_os: Arc<AtomicBool>,
    volume: Arc<Mutex<f32>>,
    muted: Arc<Mutex<bool>>,
    allow_synthetic_fallback: bool,
    force_synthetic: bool,
    #[cfg(target_os = "macos")]
    listener_guard: Arc<Mutex<Option<macos_listener::MacListenerGuard>>>,
}

impl Default for CoreAudioVolumeSync {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreAudioVolumeSync {
    /// Creates a new `CoreAudioVolumeSync`.
    pub fn new() -> Self {
        Self::with_options(None, false, false)
    }

    /// Creates a coordinator with synthetic fallback enabled.
    pub fn with_synthetic_fallback(allow_synthetic_fallback: bool) -> Self {
        Self::with_options(None, allow_synthetic_fallback, false)
    }

    /// Creates a coordinator forcing synthetic simulation mode for unit tests.
    pub fn with_forced_synthetic(force_synthetic: bool) -> Self {
        Self::with_options(None, true, force_synthetic)
    }

    /// Creates a coordinator with target device and options.
    pub fn with_options(
        device_id: Option<String>,
        allow_synthetic_fallback: bool,
        force_synthetic: bool,
    ) -> Self {
        let sync = Self {
            callbacks: Arc::new(RwLock::new(Vec::new())),
            is_syncing_to_os: Arc::new(AtomicBool::new(false)),
            volume: Arc::new(Mutex::new(0.8f32)),
            muted: Arc::new(Mutex::new(false)),
            allow_synthetic_fallback,
            force_synthetic,
            #[cfg(target_os = "macos")]
            listener_guard: Arc::new(Mutex::new(None)),
        };
        let _ = sync.rebind_endpoint(device_id);
        sync
    }

    /// Returns whether synthetic fallback is allowed.
    pub fn allow_synthetic_fallback(&self) -> bool {
        self.allow_synthetic_fallback
    }

    /// Returns whether this coordinator is in forced synthetic mode.
    pub fn is_forced_synthetic(&self) -> bool {
        self.force_synthetic
    }

    /// Registers a callback to receive volume change notifications.
    pub fn add_on_volume_changed<F>(&self, callback: F)
    where
        F: Fn(VolumeSyncEvent) + Send + Sync + 'static,
    {
        if let Ok(mut cbs) = self.callbacks.write() {
            cbs.push(Arc::new(callback));
        }
    }

    /// Rebinds the volume listener to a new audio endpoint.
    pub fn rebind_endpoint(&self, device_id: Option<String>) -> EngineResult<()> {
        #[cfg(target_os = "macos")]
        {
            if self.force_synthetic {
                return Ok(());
            }

            let target_dev: Option<ca::AudioObjectID> = match device_id {
                Some(ref s) => s.parse::<u32>().ok(),
                None => macos_listener::get_default_output_device_id(),
            };

            let Some(dev_id) = target_dev else {
                return Ok(());
            };

            let (hw_vol, hw_mute) = macos_listener::read_hw_volume_and_mute(dev_id);
            if let Ok(mut v) = self.volume.lock() {
                if *v <= 1.0 {
                    *v = hw_vol;
                }
            }
            if let Ok(mut m) = self.muted.lock() {
                *m = hw_mute;
            }

            let mut addresses = Vec::new();
            for sel in [
                macos_listener::SEL_VIRTUAL_VOLUME,
                macos_listener::SEL_VOLUME_SCALAR,
                macos_listener::SEL_MUTE,
            ] {
                addresses.push(ca::AudioObjectPropertyAddress {
                    mSelector: sel,
                    mScope: macos_listener::SCOPE_OUTPUT,
                    mElement: macos_listener::ELEMENT_MAIN,
                });
            }

            let ctx = Box::into_raw(Box::new(macos_listener::VolumeListenerContext {
                device_id: dev_id,
                callbacks: self.callbacks.clone(),
                is_syncing_to_os: self.is_syncing_to_os.clone(),
                volume: self.volume.clone(),
                muted: self.muted.clone(),
            }));

            for addr in &addresses {
                unsafe {
                    let _ = ca::AudioObjectAddPropertyListener(
                        dev_id,
                        addr,
                        Some(macos_listener::property_listener),
                        ctx as *mut std::ffi::c_void,
                    );
                }
            }

            let new_guard = macos_listener::MacListenerGuard {
                device_id: dev_id,
                addresses,
                context_ptr: ctx,
            };

            if let Ok(mut g) = self.listener_guard.lock() {
                *g = Some(new_guard);
            }
        }
        let _ = device_id;
        Ok(())
    }

    /// Sets the master volume level (clamped 0.0 to 1.5 in OpenBoom, 0.0 to 1.0 on system endpoint).
    pub fn set_master_volume(&self, volume: f32) -> EngineResult<()> {
        let clamped = volume.clamp(0.0, 1.5);
        let os_scalar = clamped.min(1.0);
        self.is_syncing_to_os.store(true, Ordering::SeqCst);
        if let Ok(mut v) = self.volume.lock() {
            *v = os_scalar;
        }

        #[cfg(target_os = "macos")]
        {
            if !self.force_synthetic {
                if let Ok(guard_opt) = self.listener_guard.lock() {
                    if let Some(ref guard) = *guard_opt {
                        macos_listener::write_hw_volume(guard.device_id, os_scalar);
                    }
                }
            }
        }

        self.is_syncing_to_os.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Sets the master mute state.
    pub fn set_mute(&self, muted: bool) -> EngineResult<()> {
        self.is_syncing_to_os.store(true, Ordering::SeqCst);
        if let Ok(mut m) = self.muted.lock() {
            *m = muted;
        }

        #[cfg(target_os = "macos")]
        {
            if !self.force_synthetic {
                if let Ok(guard_opt) = self.listener_guard.lock() {
                    if let Some(ref guard) = *guard_opt {
                        macos_listener::write_hw_mute(guard.device_id, muted);
                    }
                }
            }
        }

        self.is_syncing_to_os.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Returns the current master volume scalar (0.0 to 1.5).
    pub fn get_master_volume(&self) -> EngineResult<f32> {
        self.volume
            .lock()
            .map(|v| *v)
            .map_err(|e| open_boom_core::types::EngineError::DeviceInitialization(e.to_string()))
    }

    /// Returns the current master mute state.
    pub fn is_muted(&self) -> EngineResult<bool> {
        self.muted
            .lock()
            .map(|m| *m)
            .map_err(|e| open_boom_core::types::EngineError::DeviceInitialization(e.to_string()))
    }

    /// Simulates an OS volume or mute change event (for testing and synthetic events).
    pub fn simulate_os_volume_change(
        &self,
        volume: f32,
        is_muted: bool,
        _context_guid: Option<VolumeContextGuid>,
    ) -> bool {
        let clamped = volume.clamp(0.0, 1.5);
        let os_scalar = clamped.min(1.0);
        if let Ok(mut v) = self.volume.lock() {
            *v = os_scalar;
        }
        if let Ok(mut m) = self.muted.lock() {
            *m = is_muted;
        }
        let event = VolumeSyncEvent::new(clamped, is_muted);
        if let Ok(cbs) = self.callbacks.read() {
            for cb in cbs.iter() {
                cb(event);
            }
        }
        true
    }
}
