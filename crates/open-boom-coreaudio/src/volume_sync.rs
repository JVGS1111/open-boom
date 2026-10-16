//! Interim CoreAudio master volume synchronization stub (Epic 10, Story 10-2).
//!
//! Provides an API-compatible bidirectional volume synchronization coordinator on macOS.
//! Story 10-7 will implement full CoreAudio HAL property listeners via
//! `kAudioHardwareServiceDeviceProperty_VirtualMasterVolume` and `AudioObjectAddPropertyListener`.
//! In Story 10-2, this interim adapter manages in-memory volume and mute state, provides
//! `add_on_volume_changed`, and performs graceful no-op `rebind_endpoint` calls.

use open_boom_core::types::EngineResult;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

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

/// Bidirectional CoreAudio Master Volume synchronization coordinator (Interim Stub).
pub struct CoreAudioVolumeSync {
    callbacks: Arc<RwLock<Vec<VolumeCallback>>>,
    is_syncing_to_os: Arc<AtomicBool>,
    volume: Arc<Mutex<f32>>,
    muted: Arc<Mutex<bool>>,
    allow_synthetic_fallback: bool,
    force_synthetic: bool,
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
        _device_id: Option<String>,
        allow_synthetic_fallback: bool,
        force_synthetic: bool,
    ) -> Self {
        Self {
            callbacks: Arc::new(RwLock::new(Vec::new())),
            is_syncing_to_os: Arc::new(AtomicBool::new(false)),
            volume: Arc::new(Mutex::new(0.8f32)),
            muted: Arc::new(Mutex::new(false)),
            allow_synthetic_fallback,
            force_synthetic,
        }
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

    /// Rebinds the volume listener to a new audio endpoint (graceful no-op in Story 10-2).
    pub fn rebind_endpoint(&self, _device_id: Option<String>) -> EngineResult<()> {
        Ok(())
    }

    /// Sets the master volume level (clamped 0.0 to 1.0 for system endpoint).
    pub fn set_master_volume(&self, volume: f32) -> EngineResult<()> {
        let clamped = volume.clamp(0.0, 1.0);
        self.is_syncing_to_os.store(true, Ordering::SeqCst);
        if let Ok(mut v) = self.volume.lock() {
            *v = clamped;
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
        self.is_syncing_to_os.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Returns the current master volume scalar (0.0 to 1.0).
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
        if let Ok(mut v) = self.volume.lock() {
            *v = if clamped > 1.0 { 1.0 } else { clamped };
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
