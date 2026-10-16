//! Interim CoreAudio process audio session manager stub (Epic 10, Story 10-2).
//!
//! Provides an API-compatible stub for per-application volume mixing on macOS.
//! On macOS 14.2+, Story 10-5 will implement real HAL process taps (`AudioHardwareCreateProcessTap`).
//! In Story 10-2, this interim manager gracefully returns an empty session list in production
//! and deterministic synthetic sessions in test mode, preventing any crashes or panics.

use open_boom_core::types::{AppVolumeProfile, EngineResult, ProcessAudioSession};
use std::collections::HashMap;

/// Manager coordinating per-application audio session volumes on macOS.
pub struct CoreAudioSessionManager {
    allow_synthetic_fallback: bool,
    force_synthetic: bool,
    synthetic_sessions: Vec<ProcessAudioSession>,
    stored_profiles: HashMap<String, AppVolumeProfile>,
}

impl Default for CoreAudioSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreAudioSessionManager {
    /// Creates a new `CoreAudioSessionManager`.
    pub fn new() -> Self {
        Self::with_options(false, false)
    }

    /// Creates a manager with synthetic fallback enabled for tests and headless setups.
    pub fn with_synthetic_fallback(allow_synthetic_fallback: bool) -> Self {
        Self::with_options(allow_synthetic_fallback, false)
    }

    /// Creates a manager with forced synthetic sessions for isolated unit tests.
    pub fn with_forced_synthetic(force_synthetic: bool) -> Self {
        Self::with_options(true, force_synthetic)
    }

    fn default_synthetic_sessions() -> Vec<ProcessAudioSession> {
        vec![
            ProcessAudioSession::new(1001, "spotify", 0.85, false, true),
            ProcessAudioSession::new(1002, "discord", 0.70, false, true),
            ProcessAudioSession::new(1003, "chrome", 1.00, false, false),
        ]
    }

    pub fn with_options(allow_synthetic_fallback: bool, force_synthetic: bool) -> Self {
        Self {
            allow_synthetic_fallback,
            force_synthetic,
            synthetic_sessions: Self::default_synthetic_sessions(),
            stored_profiles: HashMap::new(),
        }
    }

    /// Returns whether synthetic fallback is allowed.
    pub fn allow_synthetic_fallback(&self) -> bool {
        self.allow_synthetic_fallback
    }

    /// Returns whether this manager is forced into synthetic mode.
    pub fn is_forced_synthetic(&self) -> bool {
        self.force_synthetic
    }

    /// Enumerates active application audio sessions.
    ///
    /// In production on macOS (Story 10-2), returns `Ok(vec![])` until Story 10-5
    /// implements native ProcessTap sessions. In forced synthetic test mode, returns
    /// synthetic mock sessions.
    pub fn enumerate_sessions(&mut self) -> EngineResult<Vec<ProcessAudioSession>> {
        if self.force_synthetic {
            let mut sessions = self.synthetic_sessions.clone();
            self.apply_stored_profiles(&mut sessions);
            return Ok(sessions);
        }

        // Interim 10-2 behavior: empty list gracefully
        Ok(Vec::new())
    }

    fn apply_stored_profiles(&mut self, sessions: &mut [ProcessAudioSession]) {
        for session in sessions.iter_mut() {
            let key = session.name.trim().to_lowercase();
            if let Some(profile) = self.stored_profiles.get(&key) {
                session.volume = profile.volume;
                session.is_muted = profile.is_muted;
            }
        }
    }

    /// Sets the volume for a specific process audio session (0.0 to 1.5).
    pub fn set_process_volume(&mut self, process_id: u32, volume: f32) -> EngineResult<()> {
        let clamped = volume.clamp(0.0, 1.5);
        if let Some(session) = self
            .synthetic_sessions
            .iter_mut()
            .find(|s| s.process_id == process_id)
        {
            session.volume = clamped;
            let name = session.name.clone();
            let is_muted = session.is_muted;
            self.stored_profiles
                .insert(name.to_lowercase(), AppVolumeProfile::new(name, clamped, is_muted));
        }
        Ok(())
    }

    /// Sets the mute state for a specific process audio session.
    pub fn set_process_mute(&mut self, process_id: u32, mute: bool) -> EngineResult<()> {
        if let Some(session) = self
            .synthetic_sessions
            .iter_mut()
            .find(|s| s.process_id == process_id)
        {
            session.is_muted = mute;
            let name = session.name.clone();
            let volume = session.volume;
            self.stored_profiles
                .insert(name.to_lowercase(), AppVolumeProfile::new(name, volume, mute));
        }
        Ok(())
    }

    /// Resolves the process name for a given process ID.
    pub fn get_process_name(&self, process_id: u32) -> Option<String> {
        self.synthetic_sessions
            .iter()
            .find(|s| s.process_id == process_id)
            .map(|s| s.name.clone())
    }

    /// Gets volume for a process ID.
    pub fn get_process_volume(&self, process_id: u32) -> Option<f32> {
        self.synthetic_sessions
            .iter()
            .find(|s| s.process_id == process_id)
            .map(|s| s.volume)
    }

    /// Gets mute state for a process ID.
    pub fn get_process_mute(&self, process_id: u32) -> Option<bool> {
        self.synthetic_sessions
            .iter()
            .find(|s| s.process_id == process_id)
            .map(|s| s.is_muted)
    }

    /// Registers or updates stored volume profiles (e.g. loaded from disk).
    pub fn register_stored_profiles<I>(&mut self, profiles: I)
    where
        I: IntoIterator<Item = (String, AppVolumeProfile)>,
    {
        for (k, v) in profiles {
            self.stored_profiles.insert(k.to_lowercase(), v);
        }
    }

    /// Sets or updates a single stored application volume profile.
    pub fn set_stored_profile(&mut self, profile: AppVolumeProfile) {
        self.stored_profiles
            .insert(profile.process_name.to_lowercase(), profile);
    }

    /// Retrieves a stored profile by application name.
    pub fn get_stored_profile(&self, name: &str) -> Option<&AppVolumeProfile> {
        self.stored_profiles.get(&name.trim().to_lowercase())
    }

    /// Removes a stored profile by application name.
    pub fn remove_stored_profile(&mut self, name: &str) -> bool {
        self.stored_profiles.remove(&name.trim().to_lowercase()).is_some()
    }
}
