//! Unit and integration tests for AudioProcessor and SPSC command dispatcher.

use open_boom_core::dsp::pipeline::AudioProcessor;
use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{AudioCommand, CROSSFADE_SAMPLES, LIMITER_LOOKAHEAD_SAMPLES};

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

#[test]
fn test_volume_and_boost_perceptual_curve_and_snap() {
    let mut proc = AudioProcessor::new();

    // 1. Slider at 50% (0.5) -> p^2 perceptual curve = 0.25 (-12.04 dB)
    let slider_50 = 0.5f32;
    let vol_50 = slider_50 * slider_50;
    proc.handle_command(AudioCommand::SetMasterVolume {
        volume: vol_50,
        boost: 1.0,
    });
    assert_eq!(proc.master_volume(), 0.25);
    assert_eq!(proc.master_boost(), 1.0);
    let db_50 = 20.0 * (0.25f32).log10();
    assert!((db_50 - (-12.0412)).abs() < 0.01);

    // 2. Slider at 135% (1.35) -> volume 1.0, boost factor 1.35 (+2.61 dB)
    proc.handle_command(AudioCommand::SetMasterVolume {
        volume: 1.0,
        boost: 1.35,
    });
    assert_eq!(proc.master_volume(), 1.0);
    assert_eq!(proc.master_boost(), 1.35);
    let db_135 = 20.0 * (1.35f32).log10();
    assert!((db_135 - 2.6067).abs() < 0.01);

    // 3. Double-click snap to 100% (1.0 volume, 1.0 boost, 0.0 dB)
    proc.handle_command(AudioCommand::SetMasterVolume {
        volume: 1.0,
        boost: 1.0,
    });
    assert_eq!(proc.master_volume(), 1.0);
    assert_eq!(proc.master_boost(), 1.0);
}

#[test]
fn test_limiter_brickwall_ceiling_clamp_under_extreme_boost() {
    let mut proc = AudioProcessor::new();

    // Configure 150% volume boost (+3.52 dB) on 0 dBFS input
    proc.handle_command(AudioCommand::SetMasterVolume {
        volume: 1.0,
        boost: 1.5,
    });

    // Generate full-scale sinusoidal input (peaks at 1.0 = 0 dBFS)
    let mut input = [0.0f32; 512];
    for (i, sample) in input.iter_mut().enumerate() {
        *sample = ((i as f32) * 0.05).sin();
    }
    let mut output = [0.0f32; 512];

    proc.process_interleaved(&input, &mut output);

    let ceiling = 10.0f32.powf(-0.2 / 20.0); // ≈ 0.9772372
    let mut max_output: f32 = 0.0;

    for (i, s) in output.iter().enumerate() {
        assert!(
            s.abs() <= ceiling + 1e-5,
            "Sample at {} (value {}) exceeds -0.2 dBFS ceiling ({})",
            i,
            s.abs(),
            ceiling
        );
        assert!(
            s.abs() < 1.0,
            "Sample at {} (value {}) caused digital clipping (>= 1.0)",
            i,
            s.abs()
        );
        max_output = max_output.max(s.abs());
    }

    let max_output_dbfs = 20.0 * max_output.log10();
    assert!(
        max_output_dbfs <= -0.1,
        "Maximum peak output ({} dBFS) must be <= -0.1 dBFS",
        max_output_dbfs
    );

    let telemetry = proc.telemetry_snapshot();
    assert!(telemetry.limiter_active, "Limiter must be marked active during 150% boost");
    assert!(
        telemetry.limiter_gain_reduction_db < -3.0,
        "Limiter gain reduction ({} dB) must be around -3.7 dB",
        telemetry.limiter_gain_reduction_db
    );
    assert!(
        telemetry.limiter_headroom_db < 0.1,
        "Limiter headroom must be 0.0 dB when actively limiting"
    );
    assert_eq!(telemetry.sample_rate, 48_000);
    assert_eq!(telemetry.bit_depth, 32);
}

#[test]
fn test_limiter_extreme_pathological_transient_impulse() {
    let mut proc = AudioProcessor::new();

    // Instantaneous pathological impulse sample of amplitude 10.0 (+20 dBFS)
    let mut input = [0.0f32; 256];
    input[0] = 10.0;
    input[1] = 10.0;
    let mut output = [0.0f32; 256];

    proc.process_interleaved(&input, &mut output);

    let ceiling = 10.0f32.powf(-0.2 / 20.0);
    for (i, s) in output.iter().enumerate() {
        assert!(!s.is_nan(), "Sample {} is NaN", i);
        assert!(!s.is_infinite(), "Sample {} is infinite", i);
        assert!(
            s.abs() <= ceiling + 1e-5,
            "Pathological transient at {} (value {}) exceeded ceiling {}",
            i,
            s.abs(),
            ceiling
        );
        assert!(s.abs() < 1.0, "Pathological transient caused digital clipping");
    }
}

#[test]
fn test_limiter_lookahead_delay_and_transparent_release() {
    let mut proc = AudioProcessor::new();

    // 96 stereo frames lookahead delay = 192 interleaved samples at 48 kHz (2ms)
    // Inject a peak > ceiling (2.0) at frame 0
    let mut input = [0.0f32; 300];
    input[0] = 2.0; // Left impulse at frame 0
    input[1] = 2.0; // Right impulse at frame 0
    let mut output = [0.0f32; 300];

    proc.process_interleaved(&input, &mut output);

    // Frame 0 to 95 (indices 0 to 191) must be 0.0 due to 2ms lookahead delay
    for i in 0..192 {
        assert_eq!(
            output[i], 0.0,
            "Output sample at index {} before 96-frame lookahead delay must be 0.0",
            i
        );
    }

    let ceiling = 10.0f32.powf(-0.2 / 20.0);
    // Frame 96 (indices 192, 193) is where the impulse emerges after 2ms (96 frames)
    assert!(
        output[192].abs() <= ceiling + 1e-5,
        "Emerging impulse {} exceeds ceiling {}",
        output[192],
        ceiling
    );
    assert!(
        output[193].abs() <= ceiling + 1e-5,
        "Emerging impulse {} exceeds ceiling {}",
        output[193],
        ceiling
    );

    // Gain reduction must be triggered (< 0.0 dB)
    let t_attenuating = proc.telemetry_snapshot();
    assert!(
        t_attenuating.limiter_gain_reduction_db < 0.0,
        "Limiter gain reduction must be < 0.0 dB during peak attenuation (got {})",
        t_attenuating.limiter_gain_reduction_db
    );

    // After subsequent audio (>2400 frames = 50ms release), envelope recovers to 1.0 and GR returns to 0.0 dB
    let silence = [0.0f32; 256];
    let mut out_silence = [0.0f32; 256];
    // Advance 200 chunks of 128 frames (25,600 frames > 2400 frames)
    for _ in 0..200 {
        proc.process_interleaved(&silence, &mut out_silence);
    }
    let t_recovered = proc.telemetry_snapshot();
    assert_eq!(
        t_recovered.limiter_gain_reduction_db, 0.0,
        "Limiter gain reduction must recover to 0.0 dB after transparent release"
    );
    assert!(!t_recovered.limiter_active);
}

#[test]
fn test_limiter_odd_buffer_length_and_channel_alignment() {
    let mut proc = AudioProcessor::new();

    // 1. Process odd buffer length (len = 5: 2 stereo frames + 1 odd trailing sample)
    let input_odd = [0.1f32, -0.1f32, 0.2f32, -0.2f32, 3.0f32];
    let mut output_odd = [0.0f32; 5];
    proc.process_interleaved(&input_odd, &mut output_odd);

    let ceiling = 10.0f32.powf(-0.2 / 20.0);
    // Trailing sample at index 4 must be clamped to ceiling
    assert!(
        output_odd[4].abs() <= ceiling + 1e-5,
        "Trailing odd sample {} must be clamped to ceiling {}",
        output_odd[4],
        ceiling
    );
    assert!(output_odd[4].abs() < 1.0, "Trailing odd sample caused clipping");

    // 2. Verify channel alignment: subsequent stereo frames must NOT have left and right desynchronized
    // Flush the buffer first with silence
    let silence = [0.0f32; 4];
    let mut out_s = [0.0f32; 4];
    for _ in 0..60 {
        proc.process_interleaved(&silence, &mut out_s);
    }

    // Now send an impulse on Left channel ONLY: [0.7, 0.0]
    let mut impulse_buf = [0.0f32; 200];
    impulse_buf[0] = 0.7; // Left only
    impulse_buf[1] = 0.0; // Right is silent
    let mut impulse_out = [0.0f32; 200];
    proc.process_interleaved(&impulse_buf, &mut impulse_out);

    // Impulse emerges at frame 96 (indices 192, 193)
    // Left channel (index 192) must have the impulse (~0.7), and Right channel (index 193) must be 0.0!
    assert!(
        (impulse_out[192] - 0.7).abs() < 1e-3,
        "Left channel must retain impulse after lookahead delay (got {})",
        impulse_out[192]
    );
    assert_eq!(
        impulse_out[193], 0.0,
        "Right channel must remain exactly 0.0 without channel desynchronization"
    );
}

#[test]
fn test_limiter_headroom_metering() {
    let mut proc = AudioProcessor::new();

    // 1. Audio with peaks at -6.0 dBFS (amplitude 0.5)
    let input_6db = [0.5f32, -0.5f32, 0.5f32, -0.5f32];
    let mut output_6db = [0.0f32; 4];
    // Warm up lookahead buffer with this signal (needs >= 48 iterations of 2 frames to exit 96-frame delay)
    for _ in 0..60 {
        proc.process_interleaved(&input_6db, &mut output_6db);
    }
    let t_6db = proc.telemetry_snapshot();
    assert_eq!(t_6db.limiter_gain_reduction_db, 0.0);
    assert!(!t_6db.limiter_active);
    // Headroom = -0.2 - (-6.02) ≈ 5.8 dB
    assert!(
        (t_6db.limiter_headroom_db - 5.82).abs() < 0.1,
        "Headroom for -6.0 dBFS signal must be ≈ 5.8 dB (got {} dB)",
        t_6db.limiter_headroom_db
    );

    // 2. Silence: flush lookahead buffer with silence, headroom must be capped at 20.0 dB
    let silence = [0.0f32; 4];
    let mut out_silence = [0.0f32; 4];
    for _ in 0..60 {
        proc.process_interleaved(&silence, &mut out_silence);
    }
    let t_silent = proc.telemetry_snapshot();
    assert_eq!(t_silent.limiter_headroom_db, 20.0);

    // 3. Master power OFF: passthrough preserves peak metering informativeness
    proc.handle_command(AudioCommand::SetMasterPower(false));
    proc.process_interleaved(&input_6db, &mut output_6db);
    let t_off = proc.telemetry_snapshot();
    assert!(
        (t_off.limiter_headroom_db - 5.82).abs() < 0.1,
        "Headroom during power OFF passthrough must remain informative"
    );
    assert_eq!(t_off.limiter_gain_reduction_db, 0.0);
    assert!(!t_off.dsp_active);

    // 4. A/B Bypass engaged: passthrough preserves peak metering informativeness
    proc.handle_command(AudioCommand::SetMasterPower(true));
    proc.handle_command(AudioCommand::SetBypass(true));
    // Warm up lookahead buffer and complete 240-frame crossfade (>= 336 frames)
    for _ in 0..200 {
        proc.process_interleaved(&input_6db, &mut output_6db);
    }
    let t_bypassed = proc.telemetry_snapshot();
    assert!(
        (t_bypassed.limiter_headroom_db - 5.82).abs() < 0.1,
        "Headroom during bypass passthrough must remain informative"
    );
    assert_eq!(t_bypassed.limiter_gain_reduction_db, 0.0);
    assert!(!t_bypassed.dsp_active);
}

#[test]
fn test_crossfade_duration_and_equal_power_ramp() {
    assert_eq!(CROSSFADE_SAMPLES, 240);
    // 240 samples at 48 kHz = 5ms
    let duration_ms = (CROSSFADE_SAMPLES as f32 / 48000.0) * 1000.0;
    assert!((duration_ms - 5.0).abs() < 1e-4);

    let mut proc = AudioProcessor::new();
    // Set volume to 0.5 so processed differs from dry (0.5 vs 1.0)
    proc.handle_command(AudioCommand::SetMasterVolume { volume: 0.5, boost: 1.0 });
    assert_eq!(proc.crossfade_pos(), 0.0);

    // Warm up lookahead delay with constant input 0.5 (below -0.2 dBFS limiter threshold)
    let in_dc = [0.5f32, 0.5f32];
    let mut out_dc = [0.0f32; 2];
    for _ in 0..120 {
        proc.process_interleaved(&in_dc, &mut out_dc);
    }
    // At start (alpha=0), output should be 0.5 * 0.5 = 0.25
    assert!((out_dc[0] - 0.25).abs() < 1e-3);

    // Engage A/B Bypass
    proc.handle_command(AudioCommand::SetBypass(true));

    // Process frame by frame for 240 frames
    let in_frame = [0.5f32, 0.5f32];
    let mut out_frame = [0.0f32; 2];

    for frame_idx in 0..CROSSFADE_SAMPLES {
        let expected_alpha = frame_idx as f32 / CROSSFADE_SAMPLES as f32;
        assert!(
            (proc.crossfade_pos() - expected_alpha).abs() < 1e-5,
            "Frame {} crossfade_pos was {} but expected {}",
            frame_idx,
            proc.crossfade_pos(),
            expected_alpha
        );

        // Verify equal-power invariant: g_a^2 + g_b^2 == 1.0
        let (g_b, g_a) = (proc.crossfade_pos() * std::f32::consts::FRAC_PI_2).sin_cos();
        let total_power = g_a * g_a + g_b * g_b;
        assert!(
            (total_power - 1.0).abs() < 1e-6,
            "Equal-power invariant violated at frame {}: power = {}",
            frame_idx,
            total_power
        );

        proc.process_interleaved(&in_frame, &mut out_frame);
    }

    // After exactly 240 frames, crossfade_pos must be 1.0
    assert_eq!(proc.crossfade_pos(), 1.0);

    // Drain lookahead buffer (96 frames) so full bypass (dry 0.5) emerges
    for _ in 0..100 {
        proc.process_interleaved(&in_frame, &mut out_frame);
    }
    assert!(
        (out_frame[0] - 0.5).abs() < 1e-3,
        "Full bypass must output uncolored dry level 0.5 (got {})",
        out_frame[0]
    );

    // Disengage A/B Bypass
    proc.handle_command(AudioCommand::SetBypass(false));

    // Ramp down for 240 frames
    for frame_idx in 0..CROSSFADE_SAMPLES {
        let expected_alpha = 1.0 - (frame_idx as f32 / CROSSFADE_SAMPLES as f32);
        assert!(
            (proc.crossfade_pos() - expected_alpha).abs() < 1e-5,
            "Disengage frame {} crossfade_pos was {} but expected {}",
            frame_idx,
            proc.crossfade_pos(),
            expected_alpha
        );

        let (g_b, g_a) = (proc.crossfade_pos() * std::f32::consts::FRAC_PI_2).sin_cos();
        let total_power = g_a * g_a + g_b * g_b;
        assert!((total_power - 1.0).abs() < 1e-6);

        proc.process_interleaved(&in_frame, &mut out_frame);
    }

    // After 240 frames, crossfade_pos must be 0.0
    assert_eq!(proc.crossfade_pos(), 0.0);

    // Drain lookahead buffer so processed level (0.25) emerges
    for _ in 0..100 {
        proc.process_interleaved(&in_frame, &mut out_frame);
    }
    assert!(
        (out_frame[0] - 0.25).abs() < 1e-3,
        "Processed audio must return to volume 0.25 (got {})",
        out_frame[0]
    );
}

#[test]
fn test_bypass_retains_2ms_lookahead_delay_and_phase_alignment() {
    let mut proc = AudioProcessor::new();
    // Set volume to 0.5 so processed level is 0.4 and dry is 0.8
    proc.handle_command(AudioCommand::SetMasterVolume { volume: 0.5, boost: 1.0 });
    proc.handle_command(AudioCommand::SetBypass(true));

    // Advance 240 frames to complete crossfade into full bypass
    let silence = [0.0f32; 2];
    let mut out = [0.0f32; 2];
    for _ in 0..CROSSFADE_SAMPLES {
        proc.process_interleaved(&silence, &mut out);
    }
    assert_eq!(proc.crossfade_pos(), 1.0);

    // Now inject a stereo impulse in bypass mode
    let mut impulse_buf = [0.0f32; 300];
    impulse_buf[0] = 0.8; // Left
    impulse_buf[1] = 0.8; // Right
    let mut impulse_out = [0.0f32; 300];

    proc.process_interleaved(&impulse_buf, &mut impulse_out);

    // First 96 frames (indices 0..191) must be 0.0 because dry audio also goes through 2ms lookahead delay!
    for i in 0..(LIMITER_LOOKAHEAD_SAMPLES * 2) {
        assert_eq!(
            impulse_out[i], 0.0,
            "Sample at index {} before 96-frame lookahead delay must be 0.0 in bypass mode",
            i
        );
    }

    // The impulse must emerge at frame 96 (indices 192, 193) as unattenuated dry level 0.8 (NOT 0.4!)
    assert!(
        (impulse_out[192] - 0.8).abs() < 1e-3,
        "Left channel must retain uncolored dry impulse after lookahead delay in bypass mode (got {})",
        impulse_out[192]
    );
    assert!(
        (impulse_out[193] - 0.8).abs() < 1e-3,
        "Right channel must retain uncolored dry impulse after lookahead delay in bypass mode (got {})",
        impulse_out[193]
    );
}

#[test]
fn test_rapid_bypass_toggle_chattering_stability_zero_clicks() {
    let mut proc = AudioProcessor::new();

    // Configure 120% boost so processed != dry
    proc.handle_command(AudioCommand::SetMasterVolume { volume: 1.0, boost: 1.2 });

    let mut prev_out_l = 0.0f32;
    let mut prev_out_r = 0.0f32;
    let mut in_buffer = [0.0f32; 4];
    let mut out_buffer = [0.0f32; 4];

    // Toggle 100 times in rapid succession
    for i in 0..100 {
        let bypass = i % 2 == 1;
        proc.handle_command(AudioCommand::SetBypass(bypass));

        // Generate gentle 1 kHz sine wave
        for frame in 0..2 {
            let t = (i * 2 + frame) as f32 / 48000.0;
            let sample = (2.0 * std::f32::consts::PI * 1000.0 * t).sin() * 0.5;
            in_buffer[frame * 2] = sample;
            in_buffer[frame * 2 + 1] = sample;
        }

        proc.process_interleaved(&in_buffer, &mut out_buffer);

        assert!(
            proc.crossfade_pos() >= 0.0 && proc.crossfade_pos() <= 1.0,
            "crossfade_pos {} out of bounds [0.0, 1.0]",
            proc.crossfade_pos()
        );

        for frame in 0..2 {
            let s_l = out_buffer[frame * 2];
            let s_r = out_buffer[frame * 2 + 1];

            assert!(!s_l.is_nan() && !s_l.is_infinite());
            assert!(!s_r.is_nan() && !s_r.is_infinite());

            // Check that consecutive samples do not jump abruptly (zero clicks/pops)
            if i > 50 { // After lookahead delay filled
                let diff_l = (s_l - prev_out_l).abs();
                let diff_r = (s_r - prev_out_r).abs();
                assert!(
                    diff_l < 0.25,
                    "Discontinuity detected on Left channel: delta = {}",
                    diff_l
                );
                assert!(
                    diff_r < 0.25,
                    "Discontinuity detected on Right channel: delta = {}",
                    diff_r
                );
            }

            prev_out_l = s_l;
            prev_out_r = s_r;
        }
    }
}

#[test]
fn test_master_power_off_precedence_and_crossfade_state_restoration() {
    let mut proc = AudioProcessor::new();

    // Start bypass transition
    proc.handle_command(AudioCommand::SetBypass(true));

    // Process 120 frames (halfway = 0.5)
    let silence = [0.0f32; 2];
    let mut out = [0.0f32; 2];
    for _ in 0..120 {
        proc.process_interleaved(&silence, &mut out);
    }
    assert!((proc.crossfade_pos() - 0.5).abs() < 1e-4);

    // Turn master power OFF
    proc.handle_command(AudioCommand::SetMasterPower(false));
    assert!(!proc.master_power());

    // Process audio during power OFF: bit-perfect passthrough takes precedence
    let input = [0.12f32, -0.34f32, 0.56f32, -0.78f32];
    let mut output = [0.0f32; 4];
    proc.process_interleaved(&input, &mut output);

    for idx in 0..input.len() {
        assert_eq!(
            input[idx].to_bits(),
            output[idx].to_bits(),
            "Power OFF must take precedence with bit-perfect passthrough"
        );
    }

    // Crossfade position is preserved during power OFF
    assert!((proc.crossfade_pos() - 0.5).abs() < 1e-4);

    // Turn master power back ON
    proc.handle_command(AudioCommand::SetMasterPower(true));
    assert!(proc.master_power());

    // Resume remaining 120 frames towards bypass
    for _ in 0..120 {
        proc.process_interleaved(&silence, &mut out);
    }

    // Now fully bypassed at 1.0
    assert_eq!(proc.crossfade_pos(), 1.0);
}



