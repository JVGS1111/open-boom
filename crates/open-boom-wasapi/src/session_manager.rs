//! Per-process audio session mixer running on a dedicated COM MTA worker thread.

use open_boom_core::types::{EngineError, EngineResult};
pub use open_boom_core::types::ProcessAudioSession;
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

        #[cfg(windows)]
        {
            if force_synthetic {
                return Self {
                    sender: None,
                    worker_handle: None,
                    allow_synthetic_fallback,
                    force_synthetic: true,
                    synthetic_sessions,
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
                },
                Err(_) => Self {
                    sender: None,
                    worker_handle: None,
                    allow_synthetic_fallback,
                    force_synthetic: true,
                    synthetic_sessions,
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
            }
        }
    }

    /// Enumerates all active Windows Core Audio sessions.
    pub fn enumerate_sessions(&mut self) -> EngineResult<Vec<ProcessAudioSession>> {
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

    /// Sets the master volume for a specific process audio session (0.0 to 1.5).
    pub fn set_process_volume(&mut self, process_id: u32, volume: f32) -> EngineResult<()> {
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
                    Ok(res) => return res,
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

    /// Sets the mute state for a specific process audio session.
    pub fn set_process_mute(&mut self, process_id: u32, mute: bool) -> EngineResult<()> {
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
                    Ok(res) => return res,
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
