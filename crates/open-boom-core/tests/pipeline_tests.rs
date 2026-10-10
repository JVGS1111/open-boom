//! Unit and integration tests for AudioProcessor and SPSC command dispatcher.

use open_boom_core::dsp::pipeline::AudioProcessor;
use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::AudioCommand;

#[test]
fn test_audio_processor_initial_state() {
    let proc = AudioProcessor::new();
    assert!(proc.master_power());
    assert!(proc.is_dsp_active());
    assert_eq!(proc.master_volume(), 1.0);
    assert_eq!(proc.master_boost(), 1.0);
    assert!(!proc.is_muted());
    assert!(!proc.is_bypassed());

    let telemetry = proc.telemetry_snapshot();
    assert!(telemetry.dsp_active);
    assert_eq!(telemetry.limiter_gain_reduction_db, 0.0);
}

#[test]
fn test_master_power_toggle_and_bit_perfect_passthrough() {
    let mut proc = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 16>::new();

    // Generate test audio frames
    let input = [0.15f32, -0.42f32, 0.88f32, -0.05f32, 0.99f32, -0.73f32];
    let mut output = [0.0f32; 6];

    // Toggle power OFF via SPSC queue command
    assert!(queue.push(AudioCommand::SetMasterPower(false)).is_ok());
    proc.drain_commands(&queue);

    assert!(!proc.master_power());
    assert!(!proc.is_dsp_active());

    // Process audio in OFF state
    proc.process_interleaved(&input, &mut output);

    // Verify bit-for-bit passthrough equality
    for i in 0..input.len() {
        assert_eq!(
            input[i].to_bits(),
            output[i].to_bits(),
            "Sample at index {} must match bit-for-bit in passthrough mode",
            i
        );
    }

    // Telemetry must report DSP inactive
    let telemetry = proc.telemetry_snapshot();
    assert!(!telemetry.dsp_active);
    assert_eq!(telemetry.limiter_gain_reduction_db, 0.0);

    // Toggle power back ON via SPSC queue
    assert!(queue.push(AudioCommand::SetMasterPower(true)).is_ok());
    proc.drain_commands(&queue);

    assert!(proc.master_power());
    assert!(proc.is_dsp_active());
    let telemetry_on = proc.telemetry_snapshot();
    assert!(telemetry_on.dsp_active);
}

#[test]
fn test_lock_free_spsc_command_drain_fifo_order() {
    let mut proc = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 16>::new();

    // Push sequential commands
    assert!(queue.push(AudioCommand::SetMasterVolume { volume: 0.75, boost: 1.25 }).is_ok());
    assert!(queue.push(AudioCommand::SetMute(true)).is_ok());
    assert!(queue.push(AudioCommand::SetBypass(true)).is_ok());

    proc.drain_commands(&queue);

    assert!((proc.master_volume() - 0.75).abs() < 1e-6);
    assert!((proc.master_boost() - 1.25).abs() < 1e-6);
    assert!(proc.is_muted());
    assert!(proc.is_bypassed());
    assert!(!proc.is_dsp_active());

    // Next buffer: unmute and disable bypass
    assert!(queue.push(AudioCommand::SetMute(false)).is_ok());
    assert!(queue.push(AudioCommand::SetBypass(false)).is_ok());

    proc.drain_commands(&queue);

    assert!(!proc.is_muted());
    assert!(!proc.is_bypassed());
    assert!(proc.is_dsp_active());
}

#[test]
fn test_mute_produces_perfect_silence() {
    let mut proc = AudioProcessor::new();
    proc.handle_command(AudioCommand::SetMute(true));

    let input = [0.5f32, 0.5f32, -0.5f32, -0.5f32];
    let mut output = [999.0f32; 4];

    proc.process_interleaved(&input, &mut output);

    for s in output {
        assert_eq!(s, 0.0, "Muted output must be exactly 0.0");
    }
}

#[test]
fn test_volume_and_boost_clamping() {
    let mut proc = AudioProcessor::new();

    // Test extreme values
    proc.handle_command(AudioCommand::SetMasterVolume { volume: -10.0, boost: 99.0 });
    assert_eq!(proc.master_volume(), 0.0);
    assert_eq!(proc.master_boost(), 1.5);

    proc.handle_command(AudioCommand::SetMasterVolume { volume: 5.0, boost: -5.0 });
    assert_eq!(proc.master_volume(), 1.0);
    assert_eq!(proc.master_boost(), 1.0);
}

#[test]
fn test_continuous_processing_with_intermittent_commands() {
    let mut proc = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 32>::new();

    let mut in_buffer = [0.1f32; 128];
    let mut out_buffer = [0.0f32; 128];

    for cycle in 0..100 {
        if cycle % 10 == 0 {
            let power = cycle % 20 == 0;
            let _ = queue.push(AudioCommand::SetMasterPower(power));
        }

        // Fill input with non-zero test signal
        for (i, val) in in_buffer.iter_mut().enumerate() {
            *val = ((cycle * 128 + i) as f32 * 0.01).sin();
        }

        proc.drain_commands(&queue);
        proc.process_interleaved(&in_buffer, &mut out_buffer);

        for s in out_buffer {
            assert!(!s.is_nan(), "Output sample must not be NaN");
            assert!(!s.is_infinite(), "Output sample must not be infinite");
        }
    }
}

#[test]
fn test_volume_and_boost_nan_sanitization() {
    let mut proc = AudioProcessor::new();

    // Passing NaN or Inf should safely fall back to nominal 1.0
    proc.handle_command(AudioCommand::SetMasterVolume {
        volume: f32::NAN,
        boost: f32::NAN,
    });
    assert_eq!(proc.master_volume(), 1.0);
    assert_eq!(proc.master_boost(), 1.0);

    proc.handle_command(AudioCommand::SetMasterVolume {
        volume: f32::INFINITY,
        boost: f32::NEG_INFINITY,
    });
    assert_eq!(proc.master_volume(), 1.0);
    assert_eq!(proc.master_boost(), 1.0);
}

#[test]
fn test_empty_and_mismatched_buffers() {
    let mut proc = AudioProcessor::new();

    // Empty buffers do not panic
    let empty_in: [f32; 0] = [];
    let mut empty_out: [f32; 0] = [];
    proc.process_interleaved(&empty_in, &mut empty_out);

    // Mismatched buffers process the common prefix without panicking
    let input = [0.1f32, 0.2f32, 0.3f32];
    let mut output = [0.0f32; 2];
    proc.process_interleaved(&input, &mut output);
    assert!(!output[0].is_nan());
    assert!(!output[1].is_nan());
}
