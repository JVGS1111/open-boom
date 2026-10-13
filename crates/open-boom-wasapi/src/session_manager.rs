//! Per-process audio session mixer running on a dedicated COM MTA worker thread.

use open_boom_core::types::{EngineError, EngineResult};
pub use open_boom_core::types::{AppVolumeProfile, ProcessAudioSession};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

#[cfg(windows)]
use windows::core::Interface;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, BOOL};
#[cfg(windows)]
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, AudioSessionStateActive, AudioSessionStateExpired,
    IAudioSessionControl, IAudioSessionControl2, IAudioSessionEnumerator, IAudioSessionManager2,
    IMMDevice, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
};
#[cfg(windows)]
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};
#[cfg(windows)]
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};

/// Internal command messages sent to the dedicated COM MTA worker thread.
enum SessionWorkerCommand {
    Enumerate(Sender<EngineResult<Vec<ProcessAudioSession>>>),
    SetVolume {
        process_id: u32,
        volume: f32,
        responder: Sender<EngineResult<()>>,
    },
    SetMute {
        process_id: u32,
        mute: bool,
        responder: Sender<EngineResult<()>>,
    },
    Shutdown,
}

/// Session manager coordinating per-application volume levels.
pub struct SessionManager {
    sender: Option<Sender<SessionWorkerCommand>>,
    worker_handle: Option<JoinHandle<()>>,
    allow_synthetic_fallback: bool,
    force_synthetic: bool,
    synthetic_sessions: Vec<ProcessAudioSession>,
    stored_profiles: HashMap<String, AppVolumeProfile>,
    restored_pids: HashSet<u32>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(SessionWorkerCommand::Shutdown);
        }
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl SessionManager {
    /// Creates a new SessionManager with a dedicated background COM MTA thread.
    pub fn new() -> Self {
        Self::create_internal(false, false)
    }

    /// Creates a SessionManager with synthetic fallback enabled for tests and headless setups.
    pub fn with_synthetic_fallback(allow_synthetic_fallback: bool) -> Self {
        Self::create_internal(allow_synthetic_fallback, false)
    }

    /// Creates a SessionManager with forced synthetic sessions for isolated unit tests.
    pub fn with_forced_synthetic(force_synthetic: bool) -> Self {
        Self::create_internal(true, force_synthetic)
    }

    fn default_synthetic_sessions() -> Vec<ProcessAudioSession> {
        vec![
            ProcessAudioSession::new(1001, "spotify.exe", 0.85, false, true),
            ProcessAudioSession::new(1002, "discord.exe", 0.70, false, true),
            ProcessAudioSession::new(1003, "chrome.exe", 1.00, false, false),
        ]
    }

    fn create_internal(allow_synthetic_fallback: bool, force_synthetic: bool) -> Self {
        let synthetic_sessions = Self::default_synthetic_sessions();
        let stored_profiles = HashMap::new();
        let restored_pids = HashSet::new();

        #[cfg(windows)]
        {
            if force_synthetic {
                return Self {
                    sender: None,
                    worker_handle: None,
                    allow_synthetic_fallback,
                    force_synthetic: true,
                    synthetic_sessions,
                    stored_profiles,
                    restored_pids,
                };
            }

            let (tx, rx) = mpsc::channel();
            let handle = thread::Builder::new()
                .name("open-boom-session-mgr".to_string())
                .spawn(move || {
                    Self::run_worker_thread(rx);
                });

            match handle {
                Ok(worker_handle) => Self {
                    sender: Some(tx),
                    worker_handle: Some(worker_handle),
                    allow_synthetic_fallback,
                    force_synthetic: false,
                    synthetic_sessions,
                    stored_profiles,
                    restored_pids,
                },
                Err(_) => Self {
                    sender: None,
                    worker_handle: None,
                    allow_synthetic_fallback,
                    force_synthetic: true,
                    synthetic_sessions,
                    stored_profiles,
                    restored_pids,
                },
            }
        }

        #[cfg(not(windows))]
        {
            Self {
                sender: None,
                worker_handle: None,
                allow_synthetic_fallback,
                force_synthetic: true,
                synthetic_sessions,
                stored_profiles,
                restored_pids,
            }
        }
    }

    /// Enumerates all active Windows Core Audio sessions.
    /// Automatically applies remembered volume and mute profiles to newly appearing sessions (FR23, UJ-4).
    pub fn enumerate_sessions(&mut self) -> EngineResult<Vec<ProcessAudioSession>> {
        let mut sessions = self.raw_enumerate_sessions()?;
        self.apply_stored_profiles(&mut sessions);
        Ok(sessions)
    }

    fn raw_enumerate_sessions(&mut self) -> EngineResult<Vec<ProcessAudioSession>> {
        if self.force_synthetic {
            return Ok(self.synthetic_sessions.clone());
        }

        if let Some(ref sender) = self.sender {
            let (resp_tx, resp_rx) = mpsc::channel();
            if sender.send(SessionWorkerCommand::Enumerate(resp_tx)).is_ok() {
                match resp_rx.recv() {
                    Ok(Ok(sessions)) => return Ok(sessions),
                    Ok(Err(e)) => {
                        if self.allow_synthetic_fallback {
                            return Ok(self.synthetic_sessions.clone());
                        }
                        return Err(e);
                    }
                    Err(_) => {
                        if self.allow_synthetic_fallback {
                            return Ok(self.synthetic_sessions.clone());
                        }
                        return Err(EngineError::DeviceInitialization(
                            "Session worker thread communication failed".to_string(),
                        ));
                    }
                }
            }
        }

        if self.allow_synthetic_fallback {
            Ok(self.synthetic_sessions.clone())
        } else {
            Err(EngineError::DeviceInitialization(
                "Audio session worker is not active".to_string(),
            ))
        }
    }

    fn dispatch_set_volume(&mut self, process_id: u32, volume: f32) -> EngineResult<()> {
        let clamped_vol = volume.clamp(0.0, 1.5);

        if self.force_synthetic {
            if let Some(session) = self.synthetic_sessions.iter_mut().find(|s| s.process_id == process_id) {
                session.volume = clamped_vol;
                return Ok(());
            }
            return Err(EngineError::DeviceInitialization(format!(
                "Synthetic process session {} not found",
                process_id
            )));
        }

        if let Some(ref sender) = self.sender {
            let (resp_tx, resp_rx) = mpsc::channel();
            if sender
                .send(SessionWorkerCommand::SetVolume {
                    process_id,
                    volume: clamped_vol,
                    responder: resp_tx,
                })
                .is_ok()
            {
                match resp_rx.recv() {
                    Ok(Ok(())) => return Ok(()),
                    Ok(Err(err)) => {
                        if self.allow_synthetic_fallback {
                            if let Some(session) =
                                self.synthetic_sessions.iter_mut().find(|s| s.process_id == process_id)
                            {
                                session.volume = clamped_vol;
                                return Ok(());
                            }
                        }
                        return Err(err);
                    }
                    Err(_) => {
                        if self.allow_synthetic_fallback {
                            if let Some(session) =
                                self.synthetic_sessions.iter_mut().find(|s| s.process_id == process_id)
                            {
                                session.volume = clamped_vol;
                                return Ok(());
                            }
                        }
                        return Err(EngineError::DeviceInitialization(
                            "Session worker thread communication failed".to_string(),
                        ));
                    }
                }
            }
        }

        if self.allow_synthetic_fallback {
            if let Some(session) = self.synthetic_sessions.iter_mut().find(|s| s.process_id == process_id) {
                session.volume = clamped_vol;
                Ok(())
            } else {
                Err(EngineError::DeviceInitialization(format!(
                    "Process session {} not found",
                    process_id
                )))
            }
        } else {
            Err(EngineError::DeviceInitialization(
                "Audio session worker is not active".to_string(),
            ))
        }
    }

    fn dispatch_set_mute(&mut self, process_id: u32, mute: bool) -> EngineResult<()> {
        if self.force_synthetic {
            if let Some(session) = self.synthetic_sessions.iter_mut().find(|s| s.process_id == process_id) {
                session.is_muted = mute;
                return Ok(());
            }
            return Err(EngineError::DeviceInitialization(format!(
                "Synthetic process session {} not found",
                process_id
            )));
        }

        if let Some(ref sender) = self.sender {
            let (resp_tx, resp_rx) = mpsc::channel();
            if sender
                .send(SessionWorkerCommand::SetMute {
                    process_id,
                    mute,
                    responder: resp_tx,
                })
                .is_ok()
            {
                match resp_rx.recv() {
                    Ok(Ok(())) => return Ok(()),
                    Ok(Err(err)) => {
                        if self.allow_synthetic_fallback {
                            if let Some(session) =
                                self.synthetic_sessions.iter_mut().find(|s| s.process_id == process_id)
                            {
                                session.is_muted = mute;
                                return Ok(());
                            }
                        }
                        return Err(err);
                    }
                    Err(_) => {
                        if self.allow_synthetic_fallback {
                            if let Some(session) =
                                self.synthetic_sessions.iter_mut().find(|s| s.process_id == process_id)
                            {
                                session.is_muted = mute;
                                return Ok(());
                            }
                        }
                        return Err(EngineError::DeviceInitialization(
                            "Session worker thread communication failed".to_string(),
                        ));
                    }
                }
            }
        }

        if self.allow_synthetic_fallback {
            if let Some(session) = self.synthetic_sessions.iter_mut().find(|s| s.process_id == process_id) {
                session.is_muted = mute;
                Ok(())
            } else {
                Err(EngineError::DeviceInitialization(format!(
                    "Process session {} not found",
                    process_id
                )))
            }
        } else {
            Err(EngineError::DeviceInitialization(
                "Audio session worker is not active".to_string(),
            ))
        }
    }

    /// Sets the master volume for a specific process audio session (0.0 to 1.5).
    /// Updates remembered profile cache for automatic session recovery.
    pub fn set_process_volume(&mut self, process_id: u32, volume: f32) -> EngineResult<()> {
        let clamped_vol = volume.clamp(0.0, 1.5);
        self.dispatch_set_volume(process_id, clamped_vol)?;

        // Update stored profile cache
        if let Some(name) = self.get_process_name(process_id) {
            let key = name.trim().to_lowercase();
            let is_muted = self.get_process_mute(process_id).unwrap_or(false);
            self.stored_profiles.insert(
                key,
                AppVolumeProfile::new(name, clamped_vol, is_muted),
            );
        }
        self.restored_pids.insert(process_id);
        Ok(())
    }

    /// Sets the mute state for a specific process audio session.
    /// Updates remembered profile cache for automatic session recovery.
    pub fn set_process_mute(&mut self, process_id: u32, mute: bool) -> EngineResult<()> {
        self.dispatch_set_mute(process_id, mute)?;

        // Update stored profile cache
        if let Some(name) = self.get_process_name(process_id) {
            let key = name.trim().to_lowercase();
            let volume = self.get_process_volume(process_id).unwrap_or(1.0);
            self.stored_profiles.insert(
                key,
                AppVolumeProfile::new(name, volume, mute),
            );
        }
        self.restored_pids.insert(process_id);
        Ok(())
    }

    /// Resolves the process name for a given process ID.
    pub fn get_process_name(&self, process_id: u32) -> Option<String> {
        if let Some(session) = self.synthetic_sessions.iter().find(|s| s.process_id == process_id) {
            return Some(session.name.clone());
        }
        #[cfg(windows)]
        {
            if process_id == 0 {
                return Some("System Sounds".to_string());
            }
            let name = Self::query_process_name(process_id);
            if !name.starts_with("Process (") {
                return Some(name);
            }
        }
        None
    }

    /// Queries the current volume level of a process session if cached.
    pub fn get_process_volume(&self, process_id: u32) -> Option<f32> {
        if let Some(session) = self.synthetic_sessions.iter().find(|s| s.process_id == process_id) {
            return Some(session.volume);
        }
        if let Some(name) = self.get_process_name(process_id) {
            if let Some(profile) = self.stored_profiles.get(&name.to_lowercase()) {
                return Some(profile.volume);
            }
        }
        None
    }

    /// Queries the current mute state of a process session if cached.
    pub fn get_process_mute(&self, process_id: u32) -> Option<bool> {
        if let Some(session) = self.synthetic_sessions.iter().find(|s| s.process_id == process_id) {
            return Some(session.is_muted);
        }
        if let Some(name) = self.get_process_name(process_id) {
            if let Some(profile) = self.stored_profiles.get(&name.to_lowercase()) {
                return Some(profile.is_muted);
            }
        }
        None
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

    /// Retrieves a stored profile by application executable name.
    pub fn get_stored_profile(&self, process_name: &str) -> Option<&AppVolumeProfile> {
        self.stored_profiles.get(&process_name.trim().to_lowercase())
    }

    /// Removes a remembered application volume profile.
    pub fn remove_stored_profile(&mut self, process_name: &str) -> bool {
        self.stored_profiles.remove(&process_name.trim().to_lowercase()).is_some()
    }

    /// Returns all remembered profiles.
    pub fn get_stored_profiles(&self) -> Vec<AppVolumeProfile> {
        let mut list: Vec<AppVolumeProfile> = self.stored_profiles.values().cloned().collect();
        list.sort_by(|a, b| a.process_name.to_lowercase().cmp(&b.process_name.to_lowercase()));
        list
    }

    /// Applies remembered profiles to newly appearing process sessions (FR23, UJ-4).
    fn apply_stored_profiles(&mut self, sessions: &mut [ProcessAudioSession]) {
        let current_pids: HashSet<u32> = sessions.iter().map(|s| s.process_id).collect();
        // Prune terminated process IDs so if they restart later, they get restored
        self.restored_pids.retain(|pid| current_pids.contains(pid));

        for session in sessions.iter_mut() {
            if self.restored_pids.contains(&session.process_id) {
                continue;
            }

            let key = session.name.trim().to_lowercase();
            if let Some(profile) = self.stored_profiles.get(&key).cloned() {
                let _ = self.dispatch_set_volume(session.process_id, profile.volume);
                let _ = self.dispatch_set_mute(session.process_id, profile.is_muted);

                session.volume = profile.volume;
                session.is_muted = profile.is_muted;
            } else if session.is_muted && session.process_id != 0 {
                // If Windows ducked or muted this session during driver installation or endpoint migration,
                // and the user has not explicitly saved a mute profile for it, automatically unmute it.
                let _ = self.dispatch_set_mute(session.process_id, false);
                session.is_muted = false;
            }
            self.restored_pids.insert(session.process_id);
        }
    }

    /// Adds a synthetic process session (for test simulation).
    pub fn add_synthetic_session(&mut self, session: ProcessAudioSession) {
        self.synthetic_sessions.push(session);
    }

    /// Removes a synthetic process session by PID (simulating process termination).
    pub fn remove_synthetic_session(&mut self, process_id: u32) {
        self.synthetic_sessions.retain(|s| s.process_id != process_id);
    }

    #[cfg(windows)]
    fn run_worker_thread(rx: Receiver<SessionWorkerCommand>) {
        let com_init = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if com_init.is_err() {
            eprintln!(
                "[open-boom-session-mgr] CoInitializeEx failed: {:?}",
                com_init
            );
            return;
        }

        while let Ok(cmd) = rx.recv() {
            match cmd {
                SessionWorkerCommand::Enumerate(responder) => {
                    let result = Self::windows_enumerate_sessions();
                    let _ = responder.send(result);
                }
                SessionWorkerCommand::SetVolume {
                    process_id,
                    volume,
                    responder,
                } => {
                    let result = Self::windows_set_volume(process_id, volume);
                    let _ = responder.send(result);
                }
                SessionWorkerCommand::SetMute {
                    process_id,
                    mute,
                    responder,
                } => {
                    let result = Self::windows_set_mute(process_id, mute);
                    let _ = responder.send(result);
                }
                SessionWorkerCommand::Shutdown => {
                    break;
                }
            }
        }

        unsafe {
            CoUninitialize();
        }
    }

    #[cfg(windows)]
    fn windows_enumerate_sessions() -> EngineResult<Vec<ProcessAudioSession>> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to create MMDeviceEnumerator: {:?}",
                        e
                    ))
                })?;

            let device: IMMDevice = enumerator
                .GetDefaultAudioEndpoint(eRender, eMultimedia)
                .map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to get default audio endpoint: {:?}",
                        e
                    ))
                })?;

            let session_manager: IAudioSessionManager2 =
                device.Activate(CLSCTX_ALL, None).map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to activate IAudioSessionManager2: {:?}",
                        e
                    ))
                })?;

            let session_enum: IAudioSessionEnumerator = session_manager
                .GetSessionEnumerator()
                .map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to get session enumerator: {:?}",
                        e
                    ))
                })?;

            let count = session_enum.GetCount().map_err(|e| {
                EngineError::DeviceInitialization(format!(
                    "Failed to get session count: {:?}",
                    e
                ))
            })?;

            let mut sessions = Vec::with_capacity(count as usize);

            for i in 0..count {
                let control: IAudioSessionControl = match session_enum.GetSession(i) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let control2: IAudioSessionControl2 = match control.cast() {
                    Ok(c2) => c2,
                    Err(_) => continue,
                };

                // Check session state
                let state = match control2.GetState() {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                // Prune expired sessions
                if state == AudioSessionStateExpired {
                    continue;
                }

                let is_active = state == AudioSessionStateActive;

                // Check if system sounds session
                let is_system_sounds = control2.IsSystemSoundsSession().is_ok();

                let pid = if is_system_sounds {
                    0
                } else {
                    control2.GetProcessId().unwrap_or(0)
                };

                let name = if is_system_sounds {
                    "System Sounds".to_string()
                } else if pid != 0 {
                    Self::query_process_name(pid)
                } else {
                    "System".to_string()
                };

                // Query volume & mute
                let (volume, is_muted) = if let Ok(simple_vol) = control.cast::<ISimpleAudioVolume>() {
                    let vol = simple_vol.GetMasterVolume().unwrap_or(1.0);
                    let mute = simple_vol.GetMute().map(|b| b.as_bool()).unwrap_or(false);
                    (vol, mute)
                } else {
                    (1.0, false)
                };

                sessions.push(ProcessAudioSession::new(
                    pid, name, volume, is_muted, is_active,
                ));
            }

            Ok(sessions)
        }
    }

    #[cfg(windows)]
    fn query_process_name(pid: u32) -> String {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
            if let Ok(h_process) = handle {
                let mut buffer = [0u16; 1024];
                let mut size = buffer.len() as u32;
                let res = QueryFullProcessImageNameW(
                    h_process,
                    PROCESS_NAME_FORMAT(0),
                    windows::core::PWSTR::from_raw(buffer.as_mut_ptr()),
                    &mut size,
                );
                let _ = CloseHandle(h_process);

                if res.is_ok() && size > 0 {
                    let path = String::from_utf16_lossy(&buffer[..size as usize]);
                    if let Some(file_name) = std::path::Path::new(&path).file_name() {
                        return file_name.to_string_lossy().to_string();
                    }
                    return path;
                }
            }
        }
        format!("Process ({})", pid)
    }

    #[cfg(windows)]
    fn windows_set_volume(process_id: u32, volume: f32) -> EngineResult<()> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to create MMDeviceEnumerator: {:?}",
                        e
                    ))
                })?;

            let device: IMMDevice = enumerator
                .GetDefaultAudioEndpoint(eRender, eMultimedia)
                .map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to get default audio endpoint: {:?}",
                        e
                    ))
                })?;

            let session_manager: IAudioSessionManager2 =
                device.Activate(CLSCTX_ALL, None).map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to activate IAudioSessionManager2: {:?}",
                        e
                    ))
                })?;

            let session_enum: IAudioSessionEnumerator = session_manager
                .GetSessionEnumerator()
                .map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to get session enumerator: {:?}",
                        e
                    ))
                })?;

            let count = session_enum.GetCount().map_err(|e| {
                EngineError::DeviceInitialization(format!(
                    "Failed to get session count: {:?}",
                    e
                ))
            })?;

            for i in 0..count {
                if let Ok(control) = session_enum.GetSession(i) {
                    if let Ok(control2) = control.cast::<IAudioSessionControl2>() {
                        let is_system = control2.IsSystemSoundsSession().is_ok();
                        let pid = if is_system { 0 } else { control2.GetProcessId().unwrap_or(0) };

                        if pid == process_id {
                            if let Ok(simple_vol) = control.cast::<ISimpleAudioVolume>() {
                                simple_vol
                                    .SetMasterVolume(volume, std::ptr::null())
                                    .map_err(|e| {
                                        EngineError::DeviceInitialization(format!(
                                            "Failed to set master volume: {:?}",
                                            e
                                        ))
                                    })?;
                                return Ok(());
                            }
                        }
                    }
                }
            }

            Err(EngineError::DeviceInitialization(format!(
                "Audio session for process {} not found",
                process_id
            )))
        }
    }

    #[cfg(windows)]
    fn windows_set_mute(process_id: u32, mute: bool) -> EngineResult<()> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to create MMDeviceEnumerator: {:?}",
                        e
                    ))
                })?;

            let device: IMMDevice = enumerator
                .GetDefaultAudioEndpoint(eRender, eMultimedia)
                .map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to get default audio endpoint: {:?}",
                        e
                    ))
                })?;

            let session_manager: IAudioSessionManager2 =
                device.Activate(CLSCTX_ALL, None).map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to activate IAudioSessionManager2: {:?}",
                        e
                    ))
                })?;

            let session_enum: IAudioSessionEnumerator = session_manager
                .GetSessionEnumerator()
                .map_err(|e| {
                    EngineError::DeviceInitialization(format!(
                        "Failed to get session enumerator: {:?}",
                        e
                    ))
                })?;

            let count = session_enum.GetCount().map_err(|e| {
                EngineError::DeviceInitialization(format!(
                    "Failed to get session count: {:?}",
                    e
                ))
            })?;

            for i in 0..count {
                if let Ok(control) = session_enum.GetSession(i) {
                    if let Ok(control2) = control.cast::<IAudioSessionControl2>() {
                        let is_system = control2.IsSystemSoundsSession().is_ok();
                        let pid = if is_system { 0 } else { control2.GetProcessId().unwrap_or(0) };

                        if pid == process_id {
                            if let Ok(simple_vol) = control.cast::<ISimpleAudioVolume>() {
                                simple_vol
                                    .SetMute(BOOL::from(mute), std::ptr::null())
                                    .map_err(|e| {
                                        EngineError::DeviceInitialization(format!(
                                            "Failed to set mute: {:?}",
                                            e
                                        ))
                                    })?;
                                return Ok(());
                            }
                        }
                    }
                }
            }

            Err(EngineError::DeviceInitialization(format!(
                "Audio session for process {} not found",
                process_id
            )))
        }
    }
}
