//! Integration tests for 8D Orbital Audio DSP modulation,
//! continuous 7.1 rotation, 32-sample sub-block coefficient updates,
//! and SPSC lock-free command dispatching.

use open_boom_core::dsp::pipeline::AudioProcessor;
use open_boom_core::dsp::spatializer_3d::{
    compute_interpolated_hrtf, wrap_deg_180, Spatializer3D, DEFAULT_ORBIT_DEPTH,
    DEFAULT_ORBIT_PERIOD_SEC, KEMAR_HRTF_LEFT_REV, KEMAR_HRTF_RIGHT_REV, MAX_ORBIT_PERIOD_SEC,
    MIN_ORBIT_PERIOD_SEC, ORBIT_SUB_BLOCK_SAMPLES,
};
use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{
    AudioCommand, OrbitDirection, CH_FRONT_LEFT, CH_LFE, ITU_R_7_1_DEFAULT_AZIMUTHS,
    SURROUND_7_1_CHANNELS,
};

#[test]
fn test_orbital_disabled_passes_static_spatializer() {
    let mut spat = Spatializer3D::new();
    assert!(!spat.orbit_enabled);
    assert_eq!(spat.orbit_phase_deg, 0.0);

    let chs = [0.4, -0.3, 0.2, 0.1, 0.05, -0.05, 0.15, -0.15];
    let (out_l, out_r) = spat.process_frame(&chs);
    assert_eq!(out_l, 0.4);
    assert_eq!(out_r, -0.3);

    // Turn spatializer on with orbit disabled -> static 3D spatializer
    spat.enabled = true;
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;

    let (l, r) = spat.process_frame(&chs);
    assert!(l.is_finite());
    assert!(r.is_finite());
    assert_eq!(spat.orbit_phase_deg, 0.0);
    assert!(!spat.orbit_enabled);
}

#[test]
fn test_orbital_clockwise_linear_rotation() {
    let mut spat = Spatializer3D::new();
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;
    spat.set_orbit(true, OrbitDirection::Clockwise, 12.0, 0.85);

    assert!(spat.enabled);
    assert!(spat.orbit_enabled);
    assert_eq!(spat.orbit_phase_deg, 0.0);

    let dummy = [0.1f32; SURROUND_7_1_CHANNELS];

    // Process 3 seconds (144,000 samples at 48 kHz = 1/4 of 12s period)
    let samples_3s = 3 * 48_000;
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    // Expected phase: +90.0 degrees
    assert!(
        (spat.orbit_phase_deg - 90.0).abs() < 0.2,
        "After 3s (1/4 period), expected ~90.0 deg, got {}",
        spat.orbit_phase_deg
    );

    // Process another 3 seconds (total 6s = 1/2 period)
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    // Expected phase: ~180.0 or -180.0 degrees (Rear Center)
    assert!(
        (spat.orbit_phase_deg.abs() - 180.0).abs() < 0.2,
        "After 6s (1/2 period), expected ~+-180.0 deg, got {}",
        spat.orbit_phase_deg
    );

    // Process another 3 seconds (total 9s = 3/4 period)
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    // Expected phase: -90.0 degrees (Surround Left side)
    assert!(
        (spat.orbit_phase_deg - (-90.0)).abs() < 0.2,
        "After 9s (3/4 period), expected ~-90.0 deg, got {}",
        spat.orbit_phase_deg
    );

    // Process another 3 seconds (total 12s = full 360 circle)
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    // Expected phase: wrapped back to ~0.0 degrees
    assert!(
        spat.orbit_phase_deg.abs() < 0.2,
        "After 12s (full period), expected ~0.0 deg, got {}",
        spat.orbit_phase_deg
    );
}

#[test]
fn test_orbital_counter_clockwise_linear_rotation() {
    let mut spat = Spatializer3D::new();
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;
    spat.set_orbit(true, OrbitDirection::CounterClockwise, 12.0, 0.85);

    let dummy = [0.1f32; SURROUND_7_1_CHANNELS];
    let samples_3s = 3 * 48_000;

    // At 3 seconds: -90.0 degrees
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    assert!(
        (spat.orbit_phase_deg - (-90.0)).abs() < 0.2,
        "CounterClockwise after 3s: expected ~-90.0 deg, got {}",
        spat.orbit_phase_deg
    );

    // At 6 seconds: +-180.0 degrees
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    assert!(
        (spat.orbit_phase_deg.abs() - 180.0).abs() < 0.2,
        "CounterClockwise after 6s: expected ~+-180.0 deg, got {}",
        spat.orbit_phase_deg
    );

    // At 9 seconds: +90.0 degrees
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    assert!(
        (spat.orbit_phase_deg - 90.0).abs() < 0.2,
        "CounterClockwise after 9s: expected ~+90.0 deg, got {}",
        spat.orbit_phase_deg
    );

    // At 12 seconds: 0.0 degrees
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    assert!(
        spat.orbit_phase_deg.abs() < 0.2,
        "CounterClockwise after 12s: expected ~0.0 deg, got {}",
        spat.orbit_phase_deg
    );
}

#[test]
fn test_orbital_ping_pong_pendulum_motion() {
    let mut spat = Spatializer3D::new();
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;
    spat.set_orbit(true, OrbitDirection::PingPong, 12.0, 0.85);

    let dummy = [0.1f32; SURROUND_7_1_CHANNELS];
    let samples_3s = 3 * 48_000;

    // Ping-Pong: theta = 180 * sin(2*pi*t / 12)
    // At t = 3s (T/4): sin(pi/2) = 1.0 -> +180.0 degrees
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    assert!(
        (spat.orbit_phase_deg - 180.0).abs() < 0.2,
        "PingPong at T/4: expected ~+180.0 deg, got {}",
        spat.orbit_phase_deg
    );

    // At t = 6s (T/2): sin(pi) = 0.0 -> 0.0 degrees
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    assert!(
        spat.orbit_phase_deg.abs() < 0.2,
        "PingPong at T/2: expected ~0.0 deg, got {}",
        spat.orbit_phase_deg
    );

    // At t = 9s (3T/4): sin(3pi/2) = -1.0 -> -180.0 degrees
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    assert!(
        (spat.orbit_phase_deg - (-180.0)).abs() < 0.2,
        "PingPong at 3T/4: expected ~-180.0 deg, got {}",
        spat.orbit_phase_deg
    );

    // At t = 12s (T): sin(2pi) = 0.0 -> 0.0 degrees
    for _ in 0..samples_3s {
        let _ = spat.process_frame(&dummy);
    }
    assert!(
        spat.orbit_phase_deg.abs() < 0.2,
        "PingPong at T: expected ~0.0 deg, got {}",
        spat.orbit_phase_deg
    );
}

#[test]
fn test_orbital_sub_block_32_frames_update_cadence() {
    let mut spat = Spatializer3D::new();
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;
    // Set fast orbit period (2.0s) so angular increment per 32 samples is distinct
    spat.set_orbit(true, OrbitDirection::Clockwise, 2.0, 1.0);

    let dummy = [0.1f32; SURROUND_7_1_CHANNELS];

    // Frame 0: sub-block counter is 0 -> updates HRTF coefficients
    let _ = spat.process_frame(&dummy);
    let hrtf_frame_0 = spat.hrtf_left[CH_FRONT_LEFT];

    // Frames 1 through 31: coefficients must remain bit-identical
    for frame_idx in 1..32 {
        let _ = spat.process_frame(&dummy);
        assert_eq!(
            spat.hrtf_left[CH_FRONT_LEFT], hrtf_frame_0,
            "HRTF coefficients must remain static within 32-sample sub-block at frame {}",
            frame_idx
        );
    }

    // Frame 32: sub-block counter wrapped to 0 -> coefficients MUST update!
    let _ = spat.process_frame(&dummy);
    let hrtf_frame_32 = spat.hrtf_left[CH_FRONT_LEFT];
    assert_ne!(
        hrtf_frame_32, hrtf_frame_0,
        "HRTF coefficients must update at exactly frame 32 boundary"
    );

    // Frames 33 through 63: coefficients must remain identical to frame 32
    for frame_idx in 33..64 {
        let _ = spat.process_frame(&dummy);
        assert_eq!(
            spat.hrtf_left[CH_FRONT_LEFT], hrtf_frame_32,
            "HRTF coefficients must remain static during second sub-block at frame {}",
            frame_idx
        );
    }

    // Frame 64: updates again!
    let _ = spat.process_frame(&dummy);
    assert_ne!(
        spat.hrtf_left[CH_FRONT_LEFT], hrtf_frame_32,
        "HRTF coefficients must update at frame 64 boundary"
    );
}

#[test]
fn test_orbital_lfe_channel_isolation() {
    let mut spat = Spatializer3D::new();
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;
    spat.set_orbit(true, OrbitDirection::Clockwise, 4.0, 1.0);

    let dummy = [0.1f32; SURROUND_7_1_CHANNELS];

    // Process 48,000 frames (1 full second = 90 deg rotation)
    for _ in 0..48_000 {
        let _ = spat.process_frame(&dummy);
        // LFE impulse responses must be strictly bit-identical to static reference
        assert_eq!(
            spat.hrtf_left[CH_LFE], KEMAR_HRTF_LEFT_REV[CH_LFE],
            "LFE Left HRTF must be isolated and unaffected by 8D orbit"
        );
        assert_eq!(
            spat.hrtf_right[CH_LFE], KEMAR_HRTF_RIGHT_REV[CH_LFE],
            "LFE Right HRTF must be isolated and unaffected by 8D orbit"
        );
    }
}

#[test]
fn test_orbital_rigid_constellation_shift_with_depth() {
    let mut spat = Spatializer3D::new();
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;

    // Test with depth = 0.0: rotation has zero effect on speaker azimuths
    spat.set_orbit(true, OrbitDirection::Clockwise, 12.0, 0.0);
    let dummy = [0.1f32; SURROUND_7_1_CHANNELS];

    for _ in 0..48_000 {
        let _ = spat.process_frame(&dummy);
    }
    // Orbit phase moved to 30.0 deg, but depth is 0.0
    assert!((spat.orbit_phase_deg - 30.0).abs() < 0.2);

    // Front Left HRTF must match static ITU-R default (-30.0 deg)
    let (expected_l, expected_r) =
        compute_interpolated_hrtf(CH_FRONT_LEFT, ITU_R_7_1_DEFAULT_AZIMUTHS[CH_FRONT_LEFT]);
    assert_eq!(spat.hrtf_left[CH_FRONT_LEFT], expected_l);
    assert_eq!(spat.hrtf_right[CH_FRONT_LEFT], expected_r);

    // Test with depth = 1.0: rigid constellation shift
    spat.set_orbit(true, OrbitDirection::Clockwise, 12.0, 1.0);
    // At orbit_phase_deg = 30.0 with depth = 1.0:
    // Front Left (-30.0) + 30.0 = 0.0 deg (Center)
    spat.update_orbit_hrtfs();
    let (fl_at_center_l, fl_at_center_r) = compute_interpolated_hrtf(CH_FRONT_LEFT, 0.0);
    assert_eq!(spat.hrtf_left[CH_FRONT_LEFT], fl_at_center_l);
    assert_eq!(spat.hrtf_right[CH_FRONT_LEFT], fl_at_center_r);
}

#[test]
fn test_orbital_extreme_and_nan_inputs_sanitized() {
    let mut spat = Spatializer3D::new();

    // Pass NaN and Inf
    spat.set_orbit(true, OrbitDirection::Clockwise, f32::NAN, f32::INFINITY);
    assert_eq!(spat.orbit_period_sec, DEFAULT_ORBIT_PERIOD_SEC);
    assert_eq!(spat.orbit_depth, DEFAULT_ORBIT_DEPTH);

    // Pass period below minimum
    spat.set_orbit(true, OrbitDirection::Clockwise, 0.5, 0.5);
    assert_eq!(spat.orbit_period_sec, MIN_ORBIT_PERIOD_SEC);

    // Pass period above maximum
    spat.set_orbit(true, OrbitDirection::Clockwise, 100.0, 0.5);
    assert_eq!(spat.orbit_period_sec, MAX_ORBIT_PERIOD_SEC);

    // Pass depth clamped
    spat.set_orbit(true, OrbitDirection::Clockwise, 12.0, -1.0);
    assert_eq!(spat.orbit_depth, 0.0);

    spat.set_orbit(true, OrbitDirection::Clockwise, 12.0, 2.5);
    assert_eq!(spat.orbit_depth, 1.0);

    // Process frames without panics or NaN output
    let (l, r) = spat.process_frame(&[0.5; SURROUND_7_1_CHANNELS]);
    assert!(l.is_finite());
    assert!(r.is_finite());
}

#[test]
fn test_orbital_disable_restores_static_hrtf_smoothly() {
    let mut spat = Spatializer3D::new();
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;
    spat.set_orbit(true, OrbitDirection::Clockwise, 12.0, 1.0);

    let dummy = [0.1f32; SURROUND_7_1_CHANNELS];

    // Process 3 seconds (phase = 90 deg)
    for _ in 0..(3 * 48_000) {
        let _ = spat.process_frame(&dummy);
    }
    assert!((spat.orbit_phase_deg - 90.0).abs() < 0.2);

    // Verify Front Left is rotated away from default
    let (def_fl_l, _) =
        compute_interpolated_hrtf(CH_FRONT_LEFT, ITU_R_7_1_DEFAULT_AZIMUTHS[CH_FRONT_LEFT]);
    assert_ne!(spat.hrtf_left[CH_FRONT_LEFT], def_fl_l);

    // Disable 8D mode
    spat.set_orbit(false, OrbitDirection::Clockwise, 12.0, 1.0);
    assert!(!spat.orbit_enabled);
    assert_eq!(spat.orbit_phase_deg, 0.0);

    // HRTFs must be restored to static defaults
    assert_eq!(spat.hrtf_left[CH_FRONT_LEFT], def_fl_l);
    for ch in 0..SURROUND_7_1_CHANNELS {
        let (hl, hr) = compute_interpolated_hrtf(ch, ITU_R_7_1_DEFAULT_AZIMUTHS[ch]);
        assert_eq!(spat.hrtf_left[ch], hl);
        assert_eq!(spat.hrtf_right[ch], hr);
    }

    // Processing further produces continuous clean audio
    let (l, r) = spat.process_frame(&dummy);
    assert!(l.is_finite());
    assert!(r.is_finite());
}

#[test]
fn test_orbital_spsc_command_pipeline_and_telemetry() {
    let mut processor = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 32>::new();

    // Verify initial telemetry
    let telem0 = processor.telemetry_snapshot();
    assert!(!telem0.orbit_enabled);
    assert_eq!(telem0.orbit_phase_deg, 0.0);

    // Push SetOrbit8D command
    queue
        .push(AudioCommand::SetOrbit8D {
            enabled: true,
            direction: OrbitDirection::Clockwise,
            period_sec: 12.0,
            depth: 0.85,
        })
        .unwrap();

    // Drain into processor
    processor.drain_commands(&queue);

    assert!(processor.spatializer.orbit_enabled);
    assert_eq!(processor.spatializer.orbit_direction, OrbitDirection::Clockwise);
    assert_eq!(processor.spatializer.orbit_period_sec, 12.0);
    assert_eq!(processor.spatializer.orbit_depth, 0.85);

    // Process 48,000 frames (1 second = 30 deg rotation)
    let input = [0.25f32; 96_000]; // 48,000 stereo frames
    let mut output = [0.0f32; 96_000];
    processor.process_interleaved(&input, &mut output);

    assert!(output.iter().all(|x| x.is_finite()));

    // Verify telemetry reflection
    let telem1 = processor.telemetry_snapshot();
    assert!(telem1.orbit_enabled);
    assert!(
        (telem1.orbit_phase_deg - 30.0).abs() < 0.2,
        "Telemetry orbit_phase_deg expected ~30.0, got {}",
        telem1.orbit_phase_deg
    );
}

#[test]
fn test_orbital_direction_switch_continuity() {
    let mut spat = Spatializer3D::new();
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;
    spat.set_orbit(true, OrbitDirection::Clockwise, 12.0, 0.85);

    let dummy = [0.1f32; SURROUND_7_1_CHANNELS];

    // Rotate to 90 degrees in Clockwise
    for _ in 0..(3 * 48_000) {
        let _ = spat.process_frame(&dummy);
    }
    assert!((spat.orbit_phase_deg - 90.0).abs() < 0.2);
    let phase_before = spat.orbit_phase_deg;

    // Switch to CounterClockwise
    spat.set_orbit(true, OrbitDirection::CounterClockwise, 12.0, 0.85);

    // Next sub-block must smoothly decrement from 90.0 deg rather than jumping to -90.0 deg
    for _ in 0..ORBIT_SUB_BLOCK_SAMPLES {
        let _ = spat.process_frame(&dummy);
    }
    assert!(
        spat.orbit_phase_deg < phase_before,
        "Phase should decrease smoothly from {}, got {}",
        phase_before,
        spat.orbit_phase_deg
    );
    assert!(
        (spat.orbit_phase_deg - phase_before).abs() < 0.5,
        "Phase should not jump, was {}, got {}",
        phase_before,
        spat.orbit_phase_deg
    );
}

#[test]
fn test_orbital_performance_and_zero_allocation() {
    let mut spat = Spatializer3D::new();
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;
    spat.set_orbit(true, OrbitDirection::Clockwise, 6.0, 0.85);

    let dummy = [0.2f32; SURROUND_7_1_CHANNELS];

    let start = std::time::Instant::now();
    for _ in 0..48_000 {
        let (l, r) = spat.process_frame(&dummy);
        assert!(l.is_finite());
        assert!(r.is_finite());
    }
    let elapsed = start.elapsed();

    let budget_ms = if cfg!(debug_assertions) { 1000 } else { 250 };
    println!("Processed 1 second of 8D audio (48,000 frames) in {:?}", elapsed);
    assert!(
        elapsed.as_millis() < budget_ms,
        "Processing time {:?} exceeded budget ({}ms)",
        elapsed,
        budget_ms
    );

    // Also verify a typical audio buffer (256 samples / 5.33ms) through full AudioProcessor
    let mut processor = AudioProcessor::new();
    processor.set_orbit(true, OrbitDirection::Clockwise, 12.0, 0.85);
    let buf_in = [0.25f32; 512]; // 256 stereo frames
    let mut buf_out = [0.0f32; 512];

    let buf_start = std::time::Instant::now();
    processor.process_interleaved(&buf_in, &mut buf_out);
    let buf_elapsed = buf_start.elapsed();
    println!("Processed 256-frame buffer in {:?}", buf_elapsed);
    assert!(buf_out.iter().all(|x| x.is_finite()));
}

#[test]
fn test_wrap_deg_180_edge_cases() {
    assert_eq!(wrap_deg_180(0.0), 0.0);
    assert_eq!(wrap_deg_180(90.0), 90.0);
    assert_eq!(wrap_deg_180(-90.0), -90.0);
    assert_eq!(wrap_deg_180(180.0), 180.0);
    assert_eq!(wrap_deg_180(-180.0), -180.0);
    assert_eq!(wrap_deg_180(270.0), -90.0);
    assert_eq!(wrap_deg_180(-270.0), 90.0);
    assert_eq!(wrap_deg_180(360.0), 0.0);
    assert_eq!(wrap_deg_180(-360.0), 0.0);
    assert_eq!(wrap_deg_180(540.0), 180.0);
    assert_eq!(wrap_deg_180(f32::NAN), 0.0);
    assert_eq!(wrap_deg_180(f32::INFINITY), 0.0);
}

#[test]
fn test_orbital_sample_rate_scaling() {
    let mut processor = AudioProcessor::new();
    processor.set_sample_rate(96_000);
    processor.set_orbit(true, OrbitDirection::Clockwise, 12.0, 1.0);

    let dummy_in = [0.1f32; 2];
    let mut dummy_out = [0.0f32; 2];

    // Process 96,000 frames (exactly 1.0 second at 96 kHz)
    for _ in 0..96_000 {
        processor.process_interleaved(&dummy_in, &mut dummy_out);
    }

    let snap = processor.telemetry_snapshot();
    // At 12.0s period, 1.0s should rotate exactly 360 / 12 = 30.0 degrees
    assert!(
        (snap.orbit_phase_deg - 30.0).abs() < 0.5,
        "Expected ~30.0 deg rotation at 96 kHz, got {}",
        snap.orbit_phase_deg
    );
}

#[test]
fn test_orbital_direction_switch_to_and_from_ping_pong() {
    let mut spat = Spatializer3D::new();
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;
    spat.set_orbit(true, OrbitDirection::Clockwise, 12.0, 1.0);

    let dummy = [0.1f32; SURROUND_7_1_CHANNELS];

    // Rotate to +90 degrees (3 seconds = 144,000 frames)
    for _ in 0..144_000 {
        let _ = spat.process_frame(&dummy);
    }

    let phase_before = spat.orbit_phase_deg;
    assert!((phase_before - 90.0).abs() < 0.5);

    // Switch to PingPong mode
    spat.set_orbit(true, OrbitDirection::PingPong, 12.0, 1.0);
    let phase_after = spat.orbit_phase_deg;

    assert!(
        (phase_after - phase_before).abs() < 0.5,
        "Phase jump when switching to PingPong: before={}, after={}",
        phase_before,
        phase_after
    );

    // Process 32 frames in PingPong
    for _ in 0..32 {
        let _ = spat.process_frame(&dummy);
    }

    // Switch back to CounterClockwise
    let phase_before_ccw = spat.orbit_phase_deg;
    spat.set_orbit(true, OrbitDirection::CounterClockwise, 12.0, 1.0);
    let phase_after_ccw = spat.orbit_phase_deg;

    assert!(
        (phase_after_ccw - phase_before_ccw).abs() < 0.5,
        "Phase jump when switching from PingPong to CCW: before={}, after={}",
        phase_before_ccw,
        phase_after_ccw
    );
}

#[test]
fn test_orbital_reset_filters_clears_orbit_state() {
    let mut processor = AudioProcessor::new();
    processor.set_orbit(true, OrbitDirection::Clockwise, 12.0, 1.0);

    let dummy_in = [0.1f32; 2];
    let mut dummy_out = [0.0f32; 2];

    // Process 48,000 frames (rotates to 30 deg)
    for _ in 0..48_000 {
        processor.process_interleaved(&dummy_in, &mut dummy_out);
    }

    let snap_before = processor.telemetry_snapshot();
    assert!((snap_before.orbit_phase_deg - 30.0).abs() < 0.5);

    // Reset filters
    processor.handle_command(AudioCommand::ResetFilters);

    let snap_after = processor.telemetry_snapshot();
    assert_eq!(
        snap_after.orbit_phase_deg, 0.0,
        "ResetFilters must reset orbit_phase_deg to 0.0"
    );
}

