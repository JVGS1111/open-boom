//! Integration tests for AudioEngineCoordinator, SPSC command dispatching,
//! and live telemetry synchronization.

use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::AudioCommand;
use open_boom_wasapi::{
    AudioEngineCoordinator, LoopbackConfig, RenderConfig, DEFAULT_COMMAND_QUEUE_CAPACITY,
};

#[test]
fn test_coordinator_with_spsc_command_queue_power_toggle() {
    let queue = SpscQueue::<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>::new();
    let mut coordinator = AudioEngineCoordinator::with_command_queue(
        LoopbackConfig {
            allow_synthetic_fallback: true,
            ..Default::default()
        },
        RenderConfig {
            allow_synthetic_fallback: true,
            ..Default::default()
        },
        queue.clone(),
    );

    // Initial state before start: dsp_active is false because engine is not running
    assert!(!coordinator.is_running());
    let mut telemetry = coordinator.telemetry_snapshot();
    assert!(!telemetry.dsp_active);

    // Start engine (runs in synthetic fallback if hardware unavailable in test env)
    assert!(coordinator.start().is_ok());
    assert!(coordinator.is_running());

    // Once running, processor defaults to master_power == true, so dsp_active is true
    telemetry = coordinator.telemetry_snapshot();
    assert!(telemetry.dsp_active);
    assert!(telemetry.round_trip_latency_ms < 15.0);

    // Send SetMasterPower(false) via SPSC queue
    assert!(queue.push(AudioCommand::SetMasterPower(false)).is_ok());

    // Drain commands and verify telemetry immediately reflects power OFF
    coordinator.drain_commands();
    telemetry = coordinator.telemetry_snapshot();
    assert!(!telemetry.dsp_active, "dsp_active must be false when master power is switched off");
    assert_eq!(telemetry.limiter_gain_reduction_db, 0.0);

    // Send SetMasterPower(true) via SPSC queue
    assert!(queue.push(AudioCommand::SetMasterPower(true)).is_ok());
    coordinator.drain_commands();
    telemetry = coordinator.telemetry_snapshot();
    assert!(telemetry.dsp_active, "dsp_active must return to true when master power is switched on");

    // Clean shutdown
    assert!(coordinator.stop().is_ok());
    assert!(!coordinator.is_running());
    telemetry = coordinator.telemetry_snapshot();
    assert!(!telemetry.dsp_active, "dsp_active must be false after stop");
}

#[test]
fn test_coordinator_process_audio_chunk_bit_perfect_passthrough() {
    let queue = SpscQueue::<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>::new();
    let mut coordinator = AudioEngineCoordinator::with_command_queue(
        LoopbackConfig::default(),
        RenderConfig::default(),
        queue.clone(),
    );

    let input = [0.25f32, -0.65f32, 0.95f32, -0.12f32];
    let mut output = [0.0f32; 4];

    // Toggle power OFF
    assert!(queue.push(AudioCommand::SetMasterPower(false)).is_ok());

    // Process chunk through coordinator (drains queue and calls processor)
    coordinator.process_audio_chunk(&input, &mut output);

    for i in 0..input.len() {
        assert_eq!(
            input[i].to_bits(),
            output[i].to_bits(),
            "Coordinator audio processing with power OFF must be bit-perfect passthrough"
        );
    }
}

#[test]
fn test_coordinator_sub_15ms_latency_guarantee() {
    let coordinator = AudioEngineCoordinator::default();
    let latency = coordinator.round_trip_latency_ms();
    assert!(
        latency < 15.0,
        "Total round-trip latency must be strictly sub-15ms (measured {}ms)",
        latency
    );
}

#[test]
fn test_coordinator_telemetry_headroom_sample_rate_and_bit_depth() {
    let coordinator = AudioEngineCoordinator::default();
    let telemetry = coordinator.telemetry_snapshot();

    // Verify sample rate and bit depth metadata propagation
    assert_eq!(telemetry.sample_rate, 48_000, "Sample rate must report 48 kHz");
    assert_eq!(telemetry.bit_depth, 32, "Bit depth must report 32-bit float");
    assert!(
        telemetry.limiter_headroom_db >= 0.0,
        "Headroom must be non-negative"
    );
    assert_eq!(
        telemetry.limiter_headroom_db, 20.0,
        "Silent initial state must report capped 20.0 dB headroom"
    );
}

