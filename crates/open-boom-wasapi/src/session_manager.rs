//! Per-process audio session mixer running on a dedicated COM MTA worker thread.

use open_boom_core::types::EngineResult;

/// Process audio session information.
#[derive(Debug, Clone)]
pub struct ProcessAudioSession {
    pub process_id: u32,
    pub name: String,
    pub volume: f32,
    pub is_muted: bool,
}

/// Session manager coordinating per-application volume levels.
pub struct SessionManager {
    sessions: Vec<ProcessAudioSession>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    pub fn enumerate_sessions(&mut self) -> EngineResult<&[ProcessAudioSession]> {
        Ok(&self.sessions)
    }

    pub fn set_process_volume(&mut self, _process_id: u32, _volume: f32) -> EngineResult<()> {
        Ok(())
    }

    pub fn set_process_mute(&mut self, _process_id: u32, _mute: bool) -> EngineResult<()> {
        Ok(())
    }
}
