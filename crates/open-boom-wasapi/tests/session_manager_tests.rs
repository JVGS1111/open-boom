//! Tests for SessionManager and Windows Core Audio session coordination.

use open_boom_wasapi::{ProcessAudioSession, SessionManager};

#[test]
fn test_session_manager_synthetic_enumeration() {
    let mut manager = SessionManager::with_forced_synthetic(true);
    let sessions = manager.enumerate_sessions().expect("enumerate sessions");

    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[0].process_id, 1001);
    assert_eq!(sessions[0].name, "spotify.exe");
    assert!((sessions[0].volume - 0.85).abs() < 1e-4);
    assert!(!sessions[0].is_muted);
    assert!(sessions[0].is_active);

    assert_eq!(sessions[1].process_id, 1002);
    assert_eq!(sessions[1].name, "discord.exe");
    assert!((sessions[1].volume - 0.70).abs() < 1e-4);
    assert!(!sessions[1].is_muted);
    assert!(sessions[1].is_active);

    assert_eq!(sessions[2].process_id, 1003);
    assert_eq!(sessions[2].name, "chrome.exe");
    assert!(!sessions[2].is_active);
}

#[test]
fn test_session_manager_synthetic_volume_control() {
    let mut manager = SessionManager::with_forced_synthetic(true);

    // Set Spotify volume to 55%
    manager
        .set_process_volume(1001, 0.55)
        .expect("set spotify volume");

    let sessions = manager.enumerate_sessions().expect("enumerate sessions");
    let spotify = sessions.iter().find(|s| s.process_id == 1001).unwrap();
    assert!((spotify.volume - 0.55).abs() < 1e-4);

    // Set volume above 1.5 clamped to 1.5
    manager
        .set_process_volume(1001, 2.0)
        .expect("set volume clamped");
    let sessions2 = manager.enumerate_sessions().expect("enumerate sessions");
    let spotify2 = sessions2.iter().find(|s| s.process_id == 1001).unwrap();
    assert!((spotify2.volume - 1.5).abs() < 1e-4);

    // Set volume below 0.0 clamped to 0.0
    manager
        .set_process_volume(1001, -0.5)
        .expect("set volume clamped to zero");
    let sessions3 = manager.enumerate_sessions().expect("enumerate sessions");
    let spotify3 = sessions3.iter().find(|s| s.process_id == 1001).unwrap();
    assert!((spotify3.volume - 0.0).abs() < 1e-4);

    // Non-existent PID returns error
    let err = manager.set_process_volume(9999, 0.5);
    assert!(err.is_err());
}

#[test]
fn test_session_manager_synthetic_mute_control() {
    let mut manager = SessionManager::with_forced_synthetic(true);

    // Mute Discord
    manager
        .set_process_mute(1002, true)
        .expect("mute discord");

    let sessions = manager.enumerate_sessions().expect("enumerate sessions");
    let discord = sessions.iter().find(|s| s.process_id == 1002).unwrap();
    assert!(discord.is_muted);

    // Unmute Discord
    manager
        .set_process_mute(1002, false)
        .expect("unmute discord");

    let sessions2 = manager.enumerate_sessions().expect("enumerate sessions");
    let discord2 = sessions2.iter().find(|s| s.process_id == 1002).unwrap();
    assert!(!discord2.is_muted);

    // Non-existent PID returns error
    let err = manager.set_process_mute(9999, true);
    assert!(err.is_err());
}

#[test]
fn test_session_manager_process_lifecycle_pruning() {
    let mut manager = SessionManager::with_forced_synthetic(true);

    // Add a new dynamic game process session
    let game_session = ProcessAudioSession::new(2048, "cyberpunk2077.exe", 0.90, false, true);
    manager.add_synthetic_session(game_session);

    let sessions = manager.enumerate_sessions().expect("enumerate sessions");
    assert_eq!(sessions.len(), 4);
    assert!(sessions.iter().any(|s| s.process_id == 2048 && s.name == "cyberpunk2077.exe"));

    // Simulate process termination
    manager.remove_synthetic_session(2048);

    let sessions_after = manager.enumerate_sessions().expect("enumerate sessions");
    assert_eq!(sessions_after.len(), 3);
    assert!(!sessions_after.iter().any(|s| s.process_id == 2048));
}

#[test]
fn test_session_manager_thread_lifecycle_and_shutdown() {
    // Test creating with synthetic fallback (which initializes the worker thread on Windows)
    let mut manager = SessionManager::with_synthetic_fallback(true);

    // Enumerate sessions safely
    let sessions = manager.enumerate_sessions();
    assert!(sessions.is_ok());

    // Dropping manager should trigger thread join and cleanly terminate without panicking
    drop(manager);
}
