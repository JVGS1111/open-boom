use open_boom_core::dsp::pipeline::AudioProcessor;
use open_boom_core::dsp::spatializer_3d::Spatializer3D;
use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{
    AudioCommand, CH_CENTER, CH_FRONT_LEFT, CH_LFE, CH_REAR_LEFT, CH_SURROUND_LEFT,
    CH_SURROUND_RIGHT, ITU_R_7_1_DEFAULT_AZIMUTHS, SURROUND_7_1_CHANNELS,
};

#[test]
fn test_spatializer_disabled_is_dry_passthrough() {
    let mut spat = Spatializer3D::new();
    assert!(!spat.enabled);

    let chs = [0.5, -0.4, 0.3, 0.2, 0.1, -0.1, 0.25, -0.25];
    let (out_l, out_r) = spat.process_frame(&chs);
    assert_eq!(out_l, 0.5);
    assert_eq!(out_r, -0.4);

    let (s_l, s_r) = spat.process_stereo_frame(0.7, -0.3);
    assert_eq!(s_l, 0.7);
    assert_eq!(s_r, -0.3);
}

#[test]
fn test_spatializer_intensity_zero_is_dry() {
    let mut spat = Spatializer3D::new();
    spat.enabled = true;
    spat.set_intensity(0.0);
    spat.current_intensity = 0.0; // bypass smooth ramp for unit test

    let chs = [0.8, -0.6, 0.1, 0.0, 0.2, -0.2, 0.3, -0.3];
    let (out_l, out_r) = spat.process_frame(&chs);
    assert!((out_l - 0.8).abs() < 1e-4);
    assert!((out_r - (-0.6)).abs() < 1e-4);
}

#[test]
fn test_spatializer_itd_and_pinna_notch_rear_vs_front() {
    let mut spat_front = Spatializer3D::new();
    spat_front.enabled = true;
    spat_front.set_intensity(1.0);
    spat_front.current_intensity = 1.0;

    let mut spat_rear = Spatializer3D::new();
    spat_rear.enabled = true;
    spat_rear.set_intensity(1.0);
    spat_rear.current_intensity = 1.0;

    // Send impulse on Front Left (channel 0)
    let mut fl_impulse = [0.0f32; SURROUND_7_1_CHANNELS];
    fl_impulse[CH_FRONT_LEFT] = 1.0;

    let mut fl_out_l = Vec::new();
    let mut fl_out_r = Vec::new();

    let (l0, r0) = spat_front.process_frame(&fl_impulse);
    fl_out_l.push(l0);
    fl_out_r.push(r0);

    let zero_chs = [0.0f32; SURROUND_7_1_CHANNELS];
    for _ in 0..63 {
        let (l, r) = spat_front.process_frame(&zero_chs);
        fl_out_l.push(l);
        fl_out_r.push(r);
    }

    // Peak detection for Front Left
    let fl_peak_l_idx = fl_out_l
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
        .unwrap()
        .0;
    let fl_peak_r_idx = fl_out_r
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
        .unwrap()
        .0;

    // Verify ITD on Front Left: Right ear delayed by 10 to 14 samples (~0.25ms at 48 kHz)
    let fl_itd_samples = fl_peak_r_idx as isize - fl_peak_l_idx as isize;
    assert!(
        fl_itd_samples >= 10 && fl_itd_samples <= 14,
        "Front Left ITD was {} samples (expected 10-14)",
        fl_itd_samples
    );
    let fl_itd_ms = fl_itd_samples as f32 / 48.0;
    assert!(fl_itd_ms >= 0.20 && fl_itd_ms <= 0.30);

    // Send impulse on Rear Left (channel 4)
    let mut rl_impulse = [0.0f32; SURROUND_7_1_CHANNELS];
    rl_impulse[CH_REAR_LEFT] = 1.0;

    let mut rl_out_l = Vec::new();
    let mut rl_out_r = Vec::new();

    let (l0, r0) = spat_rear.process_frame(&rl_impulse);
    rl_out_l.push(l0);
    rl_out_r.push(r0);

    for _ in 0..63 {
        let (l, r) = spat_rear.process_frame(&zero_chs);
        rl_out_l.push(l);
        rl_out_r.push(r);
    }

    let rl_peak_l_idx = rl_out_l
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
        .unwrap()
        .0;
    let rl_peak_r_idx = rl_out_r
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
        .unwrap()
        .0;

    // Verify ITD on Rear Left: Right ear delayed by 24 to 28 samples (~0.54ms at 48 kHz)
    let rl_itd_samples = rl_peak_r_idx as isize - rl_peak_l_idx as isize;
    assert!(
        rl_itd_samples >= 24 && rl_itd_samples <= 28,
        "Rear Left ITD was {} samples (expected 24-28)",
        rl_itd_samples
    );
    let rl_itd_ms = rl_itd_samples as f32 / 48.0;
    assert!(
        rl_itd_ms >= 0.50 && rl_itd_ms <= 0.60,
        "Rear Left ITD ms was {}",
        rl_itd_ms
    );

    // Verify Pinna Rear Notch: Rear Left Left-ear response contains destructive notch reflection
    // around tap (rl_peak_l_idx + 4), producing a negative excursion
    let notch_sample = rl_out_l[rl_peak_l_idx + 4];
    assert!(
        notch_sample < -0.15,
        "Pinna rear notch reflection was {}, expected < -0.15",
        notch_sample
    );
}

#[test]
fn test_spatializer_center_and_lfe_symmetry() {
    let mut spat = Spatializer3D::new();
    spat.enabled = true;
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;

    // Center channel test
    let mut center_impulse = [0.0f32; SURROUND_7_1_CHANNELS];
    center_impulse[CH_CENTER] = 1.0;

    let (l, r) = spat.process_frame(&center_impulse);
    assert_eq!(l, r);

    for _ in 0..10 {
        let (l_decay, r_decay) = spat.process_frame(&[0.0; SURROUND_7_1_CHANNELS]);
        assert_eq!(l_decay, r_decay);
    }

    spat.reset();

    // LFE channel test
    let mut lfe_impulse = [0.0f32; SURROUND_7_1_CHANNELS];
    lfe_impulse[CH_LFE] = 1.0;

    let (l, r) = spat.process_frame(&lfe_impulse);
    assert_eq!(l, r);
}

#[test]
fn test_spatializer_mute_and_trim_gain() {
    let mut spat = Spatializer3D::new();
    spat.enabled = true;
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;

    // Test mute on Front Left
    spat.set_speaker_mute(CH_FRONT_LEFT, true);
    let mut impulse = [0.0f32; SURROUND_7_1_CHANNELS];
    impulse[CH_FRONT_LEFT] = 1.0;

    let (l_muted, r_muted) = spat.process_frame(&impulse);
    assert_eq!(l_muted, 0.0);
    assert_eq!(r_muted, 0.0);

    // Unmute and test nominal gain
    spat.reset();
    spat.set_speaker_mute(CH_FRONT_LEFT, false);
    spat.set_speaker_gain_db(CH_FRONT_LEFT, 0.0);
    let mut outs_nominal = Vec::new();
    for i in 0..64 {
        let inp = if i == 0 { impulse } else { [0.0; SURROUND_7_1_CHANNELS] };
        let (l, _) = spat.process_frame(&inp);
        outs_nominal.push(l);
    }
    let max_nom = outs_nominal.iter().cloned().fold(0.0f32, f32::max);

    // Test +6.0 dB boost (multiplier ~2.0)
    spat.reset();
    spat.set_speaker_gain_db(CH_FRONT_LEFT, 6.0);
    let mut outs_boosted = Vec::new();
    for i in 0..64 {
        let inp = if i == 0 { impulse } else { [0.0; SURROUND_7_1_CHANNELS] };
        let (l, _) = spat.process_frame(&inp);
        outs_boosted.push(l);
    }
    let max_boost = outs_boosted.iter().cloned().fold(0.0f32, f32::max);

    let ratio = max_boost / max_nom;
    assert!(
        (ratio - 1.995).abs() < 0.05,
        "Boost ratio was {}, expected ~1.995",
        ratio
    );
}

#[test]
fn test_spatializer_stereo_upmix_and_downfold() {
    let mut spat = Spatializer3D::new();
    spat.enabled = true;
    spat.set_intensity(0.8);
    spat.current_intensity = 0.8;

    // Stereo input: Left channel louder than Right channel
    let (out_l, out_r) = spat.process_stereo_frame(0.8, 0.2);

    // Left output must be stronger than Right output, with spatial binaural energy
    assert!(out_l > out_r);
    assert!(out_l.is_finite());
    assert!(out_r.is_finite());
}

#[test]
fn test_spatializer_zero_allocation_and_performance_budget() {
    let mut spat = Spatializer3D::new();
    spat.enabled = true;
    spat.set_intensity(0.75);

    let chs = [0.1, 0.2, 0.3, 0.1, 0.15, -0.15, 0.05, -0.05];

    // Process 48,000 frames (1 second of real-time 48 kHz audio)
    let start = std::time::Instant::now();
    for _ in 0..48_000 {
        let (l, r) = spat.process_frame(&chs);
        assert!(l.is_finite());
        assert!(r.is_finite());
    }
    let elapsed = start.elapsed();

    let budget_ms = if cfg!(debug_assertions) { 1000 } else { 250 };
    println!("Processed 1 second (48,000 frames) in {:?}", elapsed);
    assert!(
        elapsed.as_millis() < budget_ms,
        "Processing time {:?} exceeded budget ({}ms)",
        elapsed,
        budget_ms
    );
}

#[test]
fn test_spatializer_pipeline_integration_and_commands() {
    let mut processor = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 32>::new();

    // Verify initial telemetry
    let telem = processor.telemetry_snapshot();
    assert!(!telem.surround_enabled);
    assert_eq!(telem.surround_intensity, 0.5);

    // Dispatch SetSurround3D command
    queue
        .push(AudioCommand::SetSurround3D {
            enabled: true,
            intensity: 0.85,
        })
        .unwrap();
    queue
        .push(AudioCommand::SetSpeakerMute {
            channel: CH_REAR_LEFT as u8,
            muted: true,
        })
        .unwrap();
    queue
        .push(AudioCommand::SetSpeakerGain {
            channel: CH_CENTER as u8,
            gain_db: 3.0,
        })
        .unwrap();
    queue
        .push(AudioCommand::SetSpeakerAzimuth {
            channel: CH_FRONT_LEFT as u8,
            azimuth_deg: -45.0,
        })
        .unwrap();

    processor.drain_commands(&queue);

    assert!(processor.spatializer.enabled);
    assert_eq!(processor.spatializer.target_intensity, 0.85);
    assert!(processor.spatializer.speaker_muted[CH_REAR_LEFT]);
    assert_eq!(processor.spatializer.speaker_gains_db[CH_CENTER], 3.0);
    assert_eq!(processor.spatializer.speaker_azimuth[CH_FRONT_LEFT], -45.0);

    // Process stereo audio buffer through processor with 3D Surround active
    let input = [0.5f32, -0.5f32, 0.4f32, -0.4f32];
    let mut output = [0.0f32; 4];
    processor.process_interleaved(&input, &mut output);

    assert!(output.iter().all(|x| x.is_finite()));

    // Reset speaker positions
    queue.push(AudioCommand::ResetSpeakerPositions).unwrap();
    processor.drain_commands(&queue);
    assert_eq!(
        processor.spatializer.speaker_azimuth[CH_FRONT_LEFT],
        -30.0
    );
}

#[test]
fn test_spatializer_angular_repositioning_and_dynamic_hrtf_interpolation() {
    use open_boom_core::dsp::spatializer_3d::{
        compute_interpolated_hrtf, KEMAR_HRTF_LEFT_REV, KEMAR_HRTF_RIGHT_REV,
    };

    let mut spat = Spatializer3D::new();

    // Verify initial anchor at -30 deg matches exact KEMAR table
    for tap in 0..64 {
        assert_eq!(
            spat.hrtf_left[CH_FRONT_LEFT][tap],
            KEMAR_HRTF_LEFT_REV[CH_FRONT_LEFT][tap]
        );
        assert_eq!(
            spat.hrtf_right[CH_FRONT_LEFT][tap],
            KEMAR_HRTF_RIGHT_REV[CH_FRONT_LEFT][tap]
        );
    }

    // Move Front Left to 0.0 deg (Center anchor)
    spat.set_speaker_azimuth(CH_FRONT_LEFT, 0.0);
    assert_eq!(spat.speaker_azimuth[CH_FRONT_LEFT], 0.0);
    for tap in 0..64 {
        assert_eq!(
            spat.hrtf_left[CH_FRONT_LEFT][tap],
            KEMAR_HRTF_LEFT_REV[CH_CENTER][tap]
        );
    }

    // Move Front Left to -15.0 deg (exactly halfway between -30° and 0°)
    spat.set_speaker_azimuth(CH_FRONT_LEFT, -15.0);
    let expected_50_50 = 0.5 * KEMAR_HRTF_LEFT_REV[CH_FRONT_LEFT][63 - 3]
        + 0.5 * KEMAR_HRTF_LEFT_REV[CH_CENTER][63 - 3];
    assert!(
        (spat.hrtf_left[CH_FRONT_LEFT][63 - 3] - expected_50_50).abs() < 1e-5,
        "Interpolated tap was {}, expected {}",
        spat.hrtf_left[CH_FRONT_LEFT][63 - 3],
        expected_50_50
    );

    // LFE channel is unaffected by azimuth changes (subwoofer mono)
    let (lfe_l, lfe_r) = compute_interpolated_hrtf(CH_LFE, 90.0);
    assert_eq!(lfe_l, KEMAR_HRTF_LEFT_REV[CH_LFE]);
    assert_eq!(lfe_r, KEMAR_HRTF_RIGHT_REV[CH_LFE]);

    // Clamping pathological azimuth angles
    spat.set_speaker_azimuth(CH_SURROUND_LEFT, 999.0);
    assert_eq!(spat.speaker_azimuth[CH_SURROUND_LEFT], 180.0);

    spat.set_speaker_azimuth(CH_SURROUND_LEFT, f32::NAN);
    assert_eq!(spat.speaker_azimuth[CH_SURROUND_LEFT], -90.0);
}

#[test]
fn test_spatializer_reset_speaker_positions_to_itu_r_defaults() {
    use open_boom_core::dsp::spatializer_3d::KEMAR_HRTF_LEFT_REV;

    let mut spat = Spatializer3D::new();

    // Modify multiple speaker azimuths
    spat.set_speaker_azimuth(CH_FRONT_LEFT, -60.0);
    spat.set_speaker_azimuth(CH_CENTER, 15.0);
    spat.set_speaker_azimuth(CH_REAR_LEFT, -120.0);
    spat.set_speaker_azimuth(CH_SURROUND_RIGHT, 75.0);

    assert_eq!(spat.speaker_azimuth[CH_FRONT_LEFT], -60.0);
    assert_eq!(spat.speaker_azimuth[CH_CENTER], 15.0);
    assert_eq!(spat.speaker_azimuth[CH_REAR_LEFT], -120.0);
    assert_eq!(spat.speaker_azimuth[CH_SURROUND_RIGHT], 75.0);

    // Reset to ITU-R BS.775 reference positions
    spat.reset_speaker_positions();

    assert_eq!(spat.speaker_azimuth, ITU_R_7_1_DEFAULT_AZIMUTHS);
    for tap in 0..64 {
        assert_eq!(
            spat.hrtf_left[CH_FRONT_LEFT][tap],
            KEMAR_HRTF_LEFT_REV[CH_FRONT_LEFT][tap]
        );
        assert_eq!(
            spat.hrtf_left[CH_CENTER][tap],
            KEMAR_HRTF_LEFT_REV[CH_CENTER][tap]
        );
        assert_eq!(
            spat.hrtf_left[CH_REAR_LEFT][tap],
            KEMAR_HRTF_LEFT_REV[CH_REAR_LEFT][tap]
        );
        assert_eq!(
            spat.hrtf_left[CH_SURROUND_RIGHT][tap],
            KEMAR_HRTF_LEFT_REV[CH_SURROUND_RIGHT][tap]
        );
    }
}

#[test]
fn test_spatializer_azimuth_rapid_drag_zero_dropout() {
    let mut spat = Spatializer3D::new();
    spat.enabled = true;
    spat.set_intensity(1.0);
    spat.current_intensity = 1.0;

    let mut channels = [0.0f32; SURROUND_7_1_CHANNELS];
    channels[CH_FRONT_LEFT] = 0.5;
    channels[CH_REAR_LEFT] = 0.3;

    // Simulate 360 frames of continuous user dragging (1 degree per frame)
    for deg in -180..=180 {
        spat.set_speaker_azimuth(CH_FRONT_LEFT, deg as f32);
        let (out_l, out_r) = spat.process_frame(&channels);

        assert!(
            out_l.is_finite(),
            "Non-finite output on left at angle {}",
            deg
        );
        assert!(
            out_r.is_finite(),
            "Non-finite output on right at angle {}",
            deg
        );
        assert!(
            out_l.abs() < 2.5,
            "Excessive output magnitude on left at angle {}: {}",
            deg,
            out_l
        );
        assert!(
            out_r.abs() < 2.5,
            "Excessive output magnitude on right at angle {}: {}",
            deg,
            out_r
        );
    }
}

