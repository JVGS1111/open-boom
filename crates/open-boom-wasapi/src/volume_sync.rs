//! Real-time Windows Master Endpoint Volume bidirectional synchronization coordinator.
//!
//! Intercepts OS master volume adjustments, media keys (Vol+, Vol-, Mute), and taskbar slider
//! changes via `IAudioEndpointVolumeCallback` on a dedicated COM MTA worker thread.
//! Dispatches volume adjustments from OpenBoom to Windows via `IAudioEndpointVolume::SetMasterVolumeLevelScalar`
//! with strict feedback loop suppression using atomic guards and COM event context GUIDs.

use open_boom_core::types::{EngineError, EngineResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};

#[cfg(windows)]
use windows::Win32::Foundation::BOOL;
#[cfg(windows)]
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
};
#[cfg(windows)]
use windows::Win32::Media::Audio::{
    eConsole, eRender, AUDIO_VOLUME_NOTIFICATION_DATA, IMMDevice, IMMDeviceEnumerator,
    MMDeviceEnumerator,
};
#[cfg(windows)]
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};

/// Platform-neutral alias for the COM event context GUID carried by volume notifications.
///
/// On Windows this is `windows_core::GUID`; on other platforms a lightweight
/// equivalent keeps the crate compiling without pulling in `windows-rs`.
#[cfg(windows)]
pub use windows_core::GUID as VolumeContextGuid;

/// Platform-neutral equivalent of `windows_core::GUID` used off-Windows.
#[cfg(not(windows))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VolumeContextGuid(pub u128);

#[cfg(not(windows))]
impl VolumeContextGuid {
    /// Constructs a GUID-equivalent value from a 128-bit integer.
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }
}

/// Dedicated COM event context GUID identifying volume adjustments originating from OpenBoom.
/// Any `OnNotify` event matching this GUID is recognized as an internal echo and suppressed.
pub const OPEN_BOOM_VOLUME_CONTEXT_GUID: VolumeContextGuid =
    VolumeContextGuid::from_u128(0x7b003e8a_8422_4f1e_98c1_4a8df2d00100);

/// Volume synchronization event delivered when the OS master volume or mute state changes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeSyncEvent {
    /// Scalar volume (0.0 to 1.5 scale in OpenBoom; OS inputs are 0.0 to 1.0).
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

/// Worker commands sent to the dedicated COM MTA thread.
enum VolumeWorkerCommand {
    SetVolume(f32),
    SetMute(bool),
    GetState(Sender<(f32, bool)>),
    Rebind(Option<String>),
    Shutdown,
}

/// COM implementation of `IAudioEndpointVolumeCallback` registered on the default playback endpoint.
#[cfg(windows)]
#[windows_core::implement(IAudioEndpointVolumeCallback)]
struct EndpointVolumeCallback {
    callback_fn: Arc<RwLock<Vec<VolumeCallback>>>,
    is_syncing: Arc<AtomicBool>,
}

#[cfg(windows)]
impl IAudioEndpointVolumeCallback_Impl for EndpointVolumeCallback_Impl {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows_core::Result<()> {
        if pnotify.is_null() {
            return Ok(());
        }

        unsafe {
            let data = &*pnotify;

            // 1. Feedback Loop Suppression: Ignore notifications with OpenBoom context GUID
            if data.guidEventContext == OPEN_BOOM_VOLUME_CONTEXT_GUID {
                return Ok(());
            }

            // 2. Feedback Loop Suppression: Ignore notifications while actively syncing to Windows
            if self.is_syncing.load(Ordering::SeqCst) {
                return Ok(());
            }

            let raw_vol = data.fMasterVolume;
            let volume = if raw_vol.is_nan() {
                0.0
            } else {
                raw_vol.clamp(0.0, 1.5)
            };
            let is_muted = data.bMuted.as_bool();

            let event = VolumeSyncEvent { volume, is_muted };

            if let Ok(guard) = self.callback_fn.read() {
                for cb in guard.iter() {
                    cb(event);
                }
            }
        }

        Ok(())
    }
}

/// Bidirectional Windows Master Endpoint Volume synchronization coordinator.
pub struct EndpointVolumeSync {
    sender: Option<Sender<VolumeWorkerCommand>>,
    worker_handle: Option<JoinHandle<()>>,
    callback: Arc<RwLock<Vec<VolumeCallback>>>,
    is_syncing_to_windows: Arc<AtomicBool>,
    force_synthetic: bool,
    allow_synthetic_fallback: bool,
    synthetic_volume: Arc<Mutex<f32>>,
    synthetic_muted: Arc<Mutex<bool>>,
    is_running_synthetic: Arc<AtomicBool>,
}

impl Default for EndpointVolumeSync {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EndpointVolumeSync {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(VolumeWorkerCommand::Shutdown);
        }
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl EndpointVolumeSync {
    /// Creates a new `EndpointVolumeSync` connecting to the active Windows default playback endpoint.
    pub fn new() -> Self {
        Self::with_options(None, false, false)
    }

    /// Creates a coordinator with synthetic fallback permitted if COM or hardware is unavailable.
    pub fn with_synthetic_fallback(allow_synthetic_fallback: bool) -> Self {
        Self::with_options(None, allow_synthetic_fallback, false)
    }

    /// Creates a coordinator forcing synthetic simulation mode for deterministic unit tests.
    pub fn with_forced_synthetic(force_synthetic: bool) -> Self {
        Self::with_options(None, true, force_synthetic)
    }

    /// Creates a coordinator with specific device target, synthetic fallback, and forced synthetic flags.
    pub fn with_options(
        device_id: Option<String>,
        allow_synthetic_fallback: bool,
        force_synthetic: bool,
    ) -> Self {
        let callback = Arc::new(RwLock::new(Vec::new()));
        let is_syncing_to_windows = Arc::new(AtomicBool::new(false));
        let synthetic_volume = Arc::new(Mutex::new(0.8f32));
        let synthetic_muted = Arc::new(Mutex::new(false));
        let is_running_synthetic = Arc::new(AtomicBool::new(force_synthetic));

        #[cfg(windows)]
        if !force_synthetic {
            let (tx, rx) = mpsc::channel();
            let cb_clone = callback.clone();
            let is_syncing_clone = is_syncing_to_windows.clone();
            let running_synth = is_running_synthetic.clone();
            let dev_id_clone = device_id.clone();

            let handle = thread::Builder::new()
                .name("open-boom-volume-sync".to_string())
                .spawn(move || {
                    Self::worker_loop(
                        rx,
                        cb_clone,
                        is_syncing_clone,
                        dev_id_clone,
                        running_synth,
                        allow_synthetic_fallback,
                    );
                });

            match handle {
                Ok(h) => {
                    return Self {
                        sender: Some(tx),
                        worker_handle: Some(h),
                        callback,
                        is_syncing_to_windows,
                        force_synthetic,
                        allow_synthetic_fallback,
                        synthetic_volume,
                        synthetic_muted,
                        is_running_synthetic,
                    };
                }
                Err(err) => {
                    if allow_synthetic_fallback {
                        is_running_synthetic.store(true, Ordering::SeqCst);
                    } else {
                        eprintln!("Failed to spawn volume sync worker thread: {}", err);
                    }
                }
            }
        }

        #[cfg(not(windows))]
        {
            let _ = device_id;
            is_running_synthetic.store(true, Ordering::SeqCst);
        }

        Self {
            sender: None,
            worker_handle: None,
            callback,
            is_syncing_to_windows,
            force_synthetic,
            allow_synthetic_fallback,
            synthetic_volume,
            synthetic_muted,
            is_running_synthetic,
        }
    }

    /// Registers a single listener callback invoked whenever OS master volume or mute state changes.
    pub fn set_on_volume_changed<F>(&self, callback_fn: F)
    where
        F: Fn(VolumeSyncEvent) + Send + Sync + 'static,
    {
        if let Ok(mut guard) = self.callback.write() {
            *guard = vec![Arc::new(callback_fn)];
        }
    }

    /// Adds an additional listener callback invoked whenever OS master volume or mute state changes.
    pub fn add_on_volume_changed<F>(&self, callback_fn: F)
    where
        F: Fn(VolumeSyncEvent) + Send + Sync + 'static,
    {
        if let Ok(mut guard) = self.callback.write() {
            guard.push(Arc::new(callback_fn));
        }
    }

    /// Synchronizes volume from OpenBoom to Windows master endpoint.
    ///
    /// Mapping:
    /// - 0% to 100% (0.0 to 1.0) maps 1:1 with Windows scalar master volume.
    /// - 101% to 150% (1.01 to 1.5) clamps Windows scalar master volume to 1.0 (100%).
    pub fn set_master_volume(&self, volume: f32) -> EngineResult<()> {
        let clamped = if volume.is_nan() {
            0.0
        } else {
            volume.clamp(0.0, 1.5)
        };
        let windows_scalar = clamped.min(1.0);

        if self.is_running_synthetic.load(Ordering::SeqCst) || self.force_synthetic {
            self.is_syncing_to_windows.store(true, Ordering::SeqCst);
            if let Ok(mut v) = self.synthetic_volume.lock() {
                *v = windows_scalar;
            }
            self.is_syncing_to_windows.store(false, Ordering::SeqCst);
            return Ok(());
        }

        if let Some(ref sender) = self.sender {
            self.is_syncing_to_windows.store(true, Ordering::SeqCst);
            let res = sender
                .send(VolumeWorkerCommand::SetVolume(windows_scalar))
                .map_err(|e| EngineError::DeviceInitialization(format!("Volume sync channel error: {}", e)));
            // Note: worker thread will also maintain is_syncing_to_windows around SetMasterVolumeLevelScalar
            self.is_syncing_to_windows.store(false, Ordering::SeqCst);
            res
        } else {
            Ok(())
        }
    }

    /// Synchronizes mute state from OpenBoom to Windows master endpoint.
    pub fn set_mute(&self, muted: bool) -> EngineResult<()> {
        if self.is_running_synthetic.load(Ordering::SeqCst) || self.force_synthetic {
            self.is_syncing_to_windows.store(true, Ordering::SeqCst);
            if let Ok(mut m) = self.synthetic_muted.lock() {
                *m = muted;
            }
            self.is_syncing_to_windows.store(false, Ordering::SeqCst);
            return Ok(());
        }

        if let Some(ref sender) = self.sender {
            self.is_syncing_to_windows.store(true, Ordering::SeqCst);
            let res = sender
                .send(VolumeWorkerCommand::SetMute(muted))
                .map_err(|e| EngineError::DeviceInitialization(format!("Volume sync channel error: {}", e)));
            self.is_syncing_to_windows.store(false, Ordering::SeqCst);
            res
        } else {
            Ok(())
        }
    }

    /// Queries the current Windows master volume (scalar 0.0 to 1.0).
    pub fn get_master_volume(&self) -> EngineResult<f32> {
        if self.is_running_synthetic.load(Ordering::SeqCst) || self.force_synthetic {
            return Ok(self.synthetic_volume.lock().map(|g| *g).unwrap_or(0.8));
        }

        if let Some(ref sender) = self.sender {
            let (tx, rx) = mpsc::channel();
            sender
                .send(VolumeWorkerCommand::GetState(tx))
                .map_err(|e| EngineError::DeviceInitialization(format!("Volume sync query error: {}", e)))?;
            let (vol, _) = rx
                .recv_timeout(std::time::Duration::from_millis(500))
                .map_err(|e| EngineError::DeviceInitialization(format!("Volume sync timeout: {}", e)))?;
            Ok(vol)
        } else {
            Ok(0.8)
        }
    }

    /// Queries the current Windows master mute state.
    pub fn is_muted(&self) -> EngineResult<bool> {
        if self.is_running_synthetic.load(Ordering::SeqCst) || self.force_synthetic {
            return Ok(self.synthetic_muted.lock().map(|g| *g).unwrap_or(false));
        }

        if let Some(ref sender) = self.sender {
            let (tx, rx) = mpsc::channel();
            sender
                .send(VolumeWorkerCommand::GetState(tx))
                .map_err(|e| EngineError::DeviceInitialization(format!("Volume sync query error: {}", e)))?;
            let (_, muted) = rx
                .recv_timeout(std::time::Duration::from_millis(500))
                .map_err(|e| EngineError::DeviceInitialization(format!("Volume sync timeout: {}", e)))?;
            Ok(muted)
        } else {
            Ok(false)
        }
    }

    /// Rebinds the synchronization listener to a new audio endpoint (or default).
    pub fn rebind_endpoint(&self, device_id: Option<String>) -> EngineResult<()> {
        if self.is_running_synthetic.load(Ordering::SeqCst) || self.force_synthetic {
            return Ok(());
        }

        if let Some(ref sender) = self.sender {
            sender
                .send(VolumeWorkerCommand::Rebind(device_id))
                .map_err(|e| EngineError::DeviceInitialization(format!("Rebind error: {}", e)))
        } else {
            Ok(())
        }
    }

    /// Indicates whether synthetic fallback is allowed.
    pub fn allows_synthetic_fallback(&self) -> bool {
        self.allow_synthetic_fallback
    }

    /// Indicates whether volume synchronization is currently executing an update to Windows.
    pub fn is_syncing(&self) -> bool {
        self.is_syncing_to_windows.load(Ordering::SeqCst)
    }

    /// Indicates whether the coordinator is operating in synthetic simulation mode.
    pub fn is_synthetic(&self) -> bool {
        self.is_running_synthetic.load(Ordering::SeqCst) || self.force_synthetic
    }

    /// Simulates an OS volume change event (for testing, feedback filtering verification, and synthetic mode).
    ///
    /// Returns `true` if the event was processed and delivered to the registered listener;
    /// returns `false` if the event was suppressed (e.g. echo from OpenBoom or active sync guard).
    pub fn simulate_os_volume_change(
        &self,
        volume: f32,
        is_muted: bool,
        context_guid: Option<VolumeContextGuid>,
    ) -> bool {
        // Feedback Suppression Rule 1: Check OpenBoom context GUID
        if let Some(guid) = context_guid {
            if guid == OPEN_BOOM_VOLUME_CONTEXT_GUID {
                return false;
            }
        }

        // Feedback Suppression Rule 2: Check active synchronization flag
        if self.is_syncing_to_windows.load(Ordering::SeqCst) {
            return false;
        }

        let clamped = if volume.is_nan() {
            0.0
        } else {
            volume.clamp(0.0, 1.5)
        };

        if let Ok(mut v) = self.synthetic_volume.lock() {
            *v = clamped.min(1.0);
        }
        if let Ok(mut m) = self.synthetic_muted.lock() {
            *m = is_muted;
        }

        let event = VolumeSyncEvent {
            volume: clamped,
            is_muted,
        };

        if let Ok(guard) = self.callback.read() {
            for cb in guard.iter() {
                cb(event);
            }
        }

        true
    }

    #[cfg(windows)]
    fn worker_loop(
        rx: Receiver<VolumeWorkerCommand>,
        callback: Arc<RwLock<Vec<VolumeCallback>>>,
        is_syncing: Arc<AtomicBool>,
        mut current_device_id: Option<String>,
        is_running_synthetic: Arc<AtomicBool>,
        allow_synthetic: bool,
    ) {
        let com_init = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

        let init_res = Self::init_endpoint_volume(current_device_id.as_deref());
        let (mut endpoint_volume, mut com_callback) = match init_res {
            Ok(ep) => {
                let cb: IAudioEndpointVolumeCallback = EndpointVolumeCallback {
                    callback_fn: callback.clone(),
                    is_syncing: is_syncing.clone(),
                }
                .into();

                let reg_res = unsafe { ep.RegisterControlChangeNotify(&cb) };
                if let Err(e) = reg_res {
                    eprintln!("Failed to register volume change callback: {}", e);
                    if allow_synthetic {
                        is_running_synthetic.store(true, Ordering::SeqCst);
                    }
                    (None, None)
                } else {
                    (Some(ep), Some(cb))
                }
            }
            Err(e) => {
                eprintln!("Failed to initialize IAudioEndpointVolume: {}", e);
                if allow_synthetic {
                    is_running_synthetic.store(true, Ordering::SeqCst);
                }
                (None, None)
            }
        };

        while let Ok(cmd) = rx.recv() {
            match cmd {
                VolumeWorkerCommand::SetVolume(vol) => {
                    if let Some(ref ep) = endpoint_volume {
                        is_syncing.store(true, Ordering::SeqCst);
                        let _ = unsafe {
                            ep.SetMasterVolumeLevelScalar(vol, &OPEN_BOOM_VOLUME_CONTEXT_GUID)
                        };
                        is_syncing.store(false, Ordering::SeqCst);
                    }
                }
                VolumeWorkerCommand::SetMute(muted) => {
                    if let Some(ref ep) = endpoint_volume {
                        is_syncing.store(true, Ordering::SeqCst);
                        let _ = unsafe {
                            ep.SetMute(BOOL::from(muted), &OPEN_BOOM_VOLUME_CONTEXT_GUID)
                        };
                        is_syncing.store(false, Ordering::SeqCst);
                    }
                }
                VolumeWorkerCommand::GetState(tx) => {
                    let mut vol = 0.8f32;
                    let mut muted = false;
                    if let Some(ref ep) = endpoint_volume {
                        if let Ok(v) = unsafe { ep.GetMasterVolumeLevelScalar() } {
                            vol = v;
                        }
                        if let Ok(m) = unsafe { ep.GetMute() } {
                            muted = m.as_bool();
                        }
                    }
                    let _ = tx.send((vol, muted));
                }
                VolumeWorkerCommand::Rebind(new_id) => {
                    // Unregister old callback
                    if let (Some(ref ep), Some(ref cb)) = (&endpoint_volume, &com_callback) {
                        let _ = unsafe { ep.UnregisterControlChangeNotify(cb) };
                    }
                    current_device_id = new_id;

                    match Self::init_endpoint_volume(current_device_id.as_deref()) {
                        Ok(new_ep) => {
                            let new_cb: IAudioEndpointVolumeCallback = EndpointVolumeCallback {
                                callback_fn: callback.clone(),
                                is_syncing: is_syncing.clone(),
                            }
                            .into();
                            let _ = unsafe { new_ep.RegisterControlChangeNotify(&new_cb) };
                            endpoint_volume = Some(new_ep);
                            com_callback = Some(new_cb);
                            is_running_synthetic.store(false, Ordering::SeqCst);
                        }
                        Err(e) => {
                            eprintln!("Failed to rebind endpoint volume: {}", e);
                            endpoint_volume = None;
                            com_callback = None;
                            if allow_synthetic {
                                is_running_synthetic.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                }
                VolumeWorkerCommand::Shutdown => {
                    break;
                }
            }
        }

        // Clean cleanup on thread termination
        if let (Some(ref ep), Some(ref cb)) = (&endpoint_volume, &com_callback) {
            let _ = unsafe { ep.UnregisterControlChangeNotify(cb) };
        }
        drop(com_callback);
        drop(endpoint_volume);

        if com_init.is_ok() {
            unsafe {
                CoUninitialize();
            }
        }
    }

    #[cfg(windows)]
    fn init_endpoint_volume(device_id: Option<&str>) -> EngineResult<IAudioEndpointVolume> {
        unsafe {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| EngineError::DeviceInitialization(format!("CoCreateInstance MMDeviceEnumerator failed: {}", e)))?;

            let device: IMMDevice = if let Some(id) = device_id {
                let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
                enumerator
                    .GetDevice(windows_core::PCWSTR::from_raw(wide.as_ptr()))
                    .map_err(|e| EngineError::DeviceInitialization(format!("GetDevice failed for {}: {}", id, e)))?
            } else {
                enumerator
                    .GetDefaultAudioEndpoint(eRender, eConsole)
                    .map_err(|e| EngineError::DeviceInitialization(format!("GetDefaultAudioEndpoint failed: {}", e)))?
            };

            let ep: IAudioEndpointVolume = device
                .Activate(CLSCTX_ALL, None)
                .map_err(|e| EngineError::DeviceInitialization(format!("Activate IAudioEndpointVolume failed: {}", e)))?;

            Ok(ep)
        }
    }
}
