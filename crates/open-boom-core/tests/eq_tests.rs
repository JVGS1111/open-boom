//! Comprehensive integration and unit test suite for Story 3.1:
//! 31-Band Biquad Filter Engine & Direct Form II Transposed Topology.
//!
//! Covers:
//! - 31-band pole modulus stability (|z| < 1.0) across all ISO frequencies and gain extremes
//! - Flat response unity pass and THD+N < 0.005% benchmark
//! - Peak boost (+12 dB) and peak cut (-12 dB) at 1 kHz with Q=4.31 tapering
//! - Rapid parameter drag with 32-sample sub-buffer smoothing and zero clicks
//! - Extreme, NaN, and infinite inputs sanitization and clamping
//! - Filter delay flush (ResetFilters) state zeroing and transient suppression
//! - AudioProcessor pipeline integration and SPSC command dispatch

use open_boom_core::dsp::biquad::BiquadCoefficients;
use open_boom_core::dsp::graphic_eq_stage::GraphicEqStage;
use open_boom_core::dsp::pipeline::AudioProcessor;
use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{
    AudioCommand, EQ_Q, GRAPHIC_EQ_BANDS, INTERNAL_SAMPLE_RATE_F32, ISO_31_BAND_FREQUENCIES,
    MAX_EQ_GAIN_DB, MIN_EQ_GAIN_DB,
};
use std::f32::consts::PI;

/// Helper: generate a single sine wave sample.
fn sine_sample(freq: f32, sample_rate: f32, sample_idx: usize) -> f32 {
    let t = sample_idx as f32 / sample_rate;
    (2.0 * PI * freq * t).sin()
}

#[test]
fn test_31_band_pole_stability_across_all_frequencies_and_gains() {
    let test_gains = [-12.0, -9.0, -6.0, -3.0, -0.1, 0.0, 0.1, 3.0, 6.0, 9.0, 12.0];
    let sample_rates = [44_100.0, 48_000.0, 96_000.0];

    for &sr in &sample_rates {
        for &freq in &ISO_31_BAND_FREQUENCIES {
            for &gain in &test_gains {
                let coeffs = BiquadCoefficients::peaking_eq(sr, freq, EQ_Q, gain);

                // Jury / Schur-Cohn stability criterion: |a2| < 1.0 and |a1| < 1.0 + a2
                assert!(
                    coeffs.is_stable(),
                    "Filter unstable at sr={sr}, freq={freq}, gain={gain}: a1={}, a2={}",
                    coeffs.a1,
                    coeffs.a2
                );

                // Strictly within unit circle: |z| < 1.0
                let max_modulus = coeffs.max_pole_modulus();
                assert!(
                    max_modulus < 1.0,
                    "Pole modulus >= 1.0 at sr={sr}, freq={freq}, gain={gain}: |z| = {max_modulus}"
                );
            }
        }
    }
}

#[test]
fn test_flat_response_clean_unity_and_thd_n() {
    let mut eq = GraphicEqStage::new();
    let sample_rate = INTERNAL_SAMPLE_RATE_F32;
    let frames = 4800; // 100 ms of audio

    // Test with a 1 kHz test tone at nominal -6 dBFS (amplitude 0.5)
    let freq = 1000.0;
    let mut input_signal = Vec::with_capacity(frames);
    let mut output_signal = Vec::with_capacity(frames);

    for n in 0..frames {
        let x = 0.5 * sine_sample(freq, sample_rate, n);
        input_signal.push(x);
        let (out_l, out_r) = eq.process_stereo(x, x);
        assert_eq!(out_l, out_r, "Left and right channels must be symmetrical");
        output_signal.push(out_l);
    }

    // Skip initial settling samples (first 128 samples / ~2.6ms) to measure steady state
    let eval_range = 128..frames;
    let mut noise_power = 0.0f64;
    let mut signal_power = 0.0f64;

    for i in eval_range {
        let in_val = input_signal[i] as f64;
        let out_val = output_signal[i] as f64;
        let diff = out_val - in_val;
        noise_power += diff * diff;
        signal_power += in_val * in_val;
    }

    let thd_n = (noise_power / signal_power).sqrt();
    // NFR6 acceptance requirement: THD+N < 0.005% (0.00005)
    assert!(
        thd_n < 0.00005,
        "Flat response THD+N exceeded 0.005%: measured {thd_n} ({:.6}%)",
        thd_n * 100.0
    );

    // Also verify maximum absolute sample difference is near machine epsilon
    let max_diff = input_signal
        .iter()
        .zip(output_signal.iter())
        .map(|(&a, &b)| (a - b).abs())
        .fold(0.0f32, f32::max);

    assert!(
        max_diff < 1e-5,
        "Flat response output deviated from input: max diff = {max_diff}"
    );
}

#[test]
fn test_peak_boost_1khz_band_17() {
    let mut eq = GraphicEqStage::new();
    // Band 17 is 1000.0 Hz
    let band_17_freq = ISO_31_BAND_FREQUENCIES[17];
    assert_eq!(band_17_freq, 1000.0);

    eq.set_band_gain_immediate(17, 12.0);

    // 1. Theoretical cumulative response check
    let resp_1khz = eq.cumulative_response_db(1000.0);
    assert!(
        (resp_1khz - 12.0).abs() < 0.05,
        "Expected +12.0 dB boost at 1 kHz, got {resp_1khz:.2} dB"
    );

    // 2. Q=4.31 tapering envelope check: adjacent bands (800 Hz and 1250 Hz) must taper down
    let resp_800hz = eq.cumulative_response_db(800.0);
    let resp_1250hz = eq.cumulative_response_db(1250.0);
    assert!(
        resp_800hz < 12.0 && resp_800hz > 0.0,
        "Adjacent band 800 Hz did not taper: {resp_800hz:.2} dB"
    );
    assert!(
        resp_1250hz < 12.0 && resp_1250hz > 0.0,
        "Adjacent band 1250 Hz did not taper: {resp_1250hz:.2} dB"
    );

    // Distant frequencies should be near 0 dB
    let resp_100hz = eq.cumulative_response_db(100.0);
    let resp_10khz = eq.cumulative_response_db(10000.0);
    assert!(
        resp_100hz.abs() < 0.25,
        "Distant frequency 100 Hz affected: {resp_100hz:.2} dB"
    );
    assert!(
        resp_10khz.abs() < 0.25,
        "Distant frequency 10 kHz affected: {resp_10khz:.2} dB"
    );

    // 3. Steady-state sinusoidal execution check: feed 1 kHz sine tone
    let sample_rate = INTERNAL_SAMPLE_RATE_F32;
    let frames = 4800;
    let mut max_in = 0.0f32;
    let mut max_out = 0.0f32;

    for n in 0..frames {
        let x = 0.1 * sine_sample(1000.0, sample_rate, n);
        let (out_l, _) = eq.process_stereo(x, x);
        if n >= 2400 {
            // After settling
            max_in = max_in.max(x.abs());
            max_out = max_out.max(out_l.abs());
        }
    }

    let measured_gain_db = 20.0 * (max_out / max_in).log10();
    assert!(
        (measured_gain_db - 12.0).abs() < 0.1,
        "Measured steady-state gain at 1 kHz was {measured_gain_db:.2} dB, expected +12.0 dB"
    );
}

#[test]
fn test_peak_cut_1khz_band_17() {
    let mut eq = GraphicEqStage::new();
    eq.set_band_gain_immediate(17, -12.0);

    // 1. Theoretical response check
    let resp_1khz = eq.cumulative_response_db(1000.0);
    assert!(
        (resp_1khz - (-12.0)).abs() < 0.05,
        "Expected -12.0 dB cut at 1 kHz, got {resp_1khz:.2} dB"
    );

    // 2. Steady-state sinusoidal execution check
    let sample_rate = INTERNAL_SAMPLE_RATE_F32;
    let frames = 4800;
    let mut max_in = 0.0f32;
    let mut max_out = 0.0f32;

    for n in 0..frames {
        let x = 0.5 * sine_sample(1000.0, sample_rate, n);
        let (out_l, _) = eq.process_stereo(x, x);
        if n >= 2400 {
            max_in = max_in.max(x.abs());
            max_out = max_out.max(out_l.abs());
        }
    }

    let measured_gain_db = 20.0 * (max_out / max_in).log10();
    assert!(
        (measured_gain_db - (-12.0)).abs() < 0.1,
        "Measured steady-state cut at 1 kHz was {measured_gain_db:.2} dB, expected -12.0 dB"
    );
}

#[test]
fn test_rapid_parameter_drag_sub_buffer_smoothing_zero_clicks() {
    let mut eq = GraphicEqStage::new();
    eq.set_band_gain_immediate(17, -12.0);

    let sample_rate = INTERNAL_SAMPLE_RATE_F32;
    let frames = 100;
    let mut outputs = Vec::with_capacity(frames);

    // Sweep slider from -12 dB to +12 dB across 100 frames
    for n in 0..frames {
        let target_gain = -12.0 + (24.0 * (n as f32 / frames as f32));
        eq.set_band_gain(17, target_gain);

        let x = 0.5 * sine_sample(1000.0, sample_rate, n);
        let (out_l, out_r) = eq.process_stereo(x, x);

        assert!(out_l.is_finite(), "Output was non-finite at frame {n}");
        assert_eq!(out_l, out_r);
        outputs.push(out_l);
    }

    // Verify smooth transition without discontinuity spikes (zero clicks or pops)
    for i in 1..outputs.len() {
        let delta = (outputs[i] - outputs[i - 1]).abs();
        // Under a 1 kHz tone at 48 kHz, max theoretical sine derivative step is ~0.5 * (2*pi*1000/48000) * 4 ≈ 0.26
        // Discontinuities or zipper noise would produce instantaneous jumps >> 0.5
        assert!(
            delta < 0.45,
            "Click/pop detected at frame {i}: step delta = {delta}"
        );
    }
}

#[test]
fn test_extreme_and_nan_inputs_sanitized_and_clamped() {
    let mut eq = GraphicEqStage::new();

    // 1. Extreme positive boost: +50 dB clamped to +12 dB
    eq.set_band_gain(17, 50.0);
    assert_eq!(eq.target_gains()[17], MAX_EQ_GAIN_DB);

    // 2. Extreme negative cut: -50 dB clamped to -12 dB
    eq.set_band_gain(17, -50.0);
    assert_eq!(eq.target_gains()[17], MIN_EQ_GAIN_DB);

    // 3. Positive infinity clamped to +12 dB
    eq.set_band_gain(17, f32::INFINITY);
    assert_eq!(eq.target_gains()[17], MAX_EQ_GAIN_DB);

    // 4. Negative infinity clamped to -12 dB
    eq.set_band_gain(17, f32::NEG_INFINITY);
    assert_eq!(eq.target_gains()[17], MIN_EQ_GAIN_DB);

    // 5. NaN sanitized to 0.0 dB
    eq.set_band_gain(17, f32::NAN);
    assert_eq!(eq.target_gains()[17], 0.0);

    // Verify filter stability remains 100% intact after sanitization
    eq.flush_smoothing();
    let coeffs = eq.coefficients(17).expect("Band 17 must exist");
    assert!(coeffs.is_stable());
    assert!(coeffs.max_pole_modulus() < 1.0);
}

#[test]
fn test_filter_delay_flush_reset_filters() {
    let mut eq = GraphicEqStage::new();
    eq.set_band_gain_immediate(17, 12.0);

    // Drive filter with high amplitude impulse to charge delay lines
    for _ in 0..64 {
        let _ = eq.process_stereo(1.0, 1.0);
    }

    // Verify internal state variables s1, s2 are non-zero
    let (s1_l, s2_l) = eq.filters_l[17].state();
    let (s1_r, s2_r) = eq.filters_r[17].state();
    assert!(
        s1_l.abs() > 1e-4 || s2_l.abs() > 1e-4,
        "Filter state was expected to be charged"
    );
    assert_eq!(s1_l, s1_r);
    assert_eq!(s2_l, s2_r);

    // Execute ResetFilters
    eq.reset();

    // Verify all delay lines are zeroed across all 31 bands on both channels
    for i in 0..GRAPHIC_EQ_BANDS {
        assert_eq!(
            eq.filters_l[i].state(),
            (0.0, 0.0),
            "Left filter {i} delay line not zeroed"
        );
        assert_eq!(
            eq.filters_r[i].state(),
            (0.0, 0.0),
            "Right filter {i} delay line not zeroed"
        );
    }

    // Feed silence: must produce exactly zero output with zero transient
    let (out_l, out_r) = eq.process_stereo(0.0, 0.0);
    assert_eq!(out_l, 0.0, "Expected zero transient on flush");
    assert_eq!(out_r, 0.0, "Expected zero transient on flush");
}

#[test]
fn test_pipeline_integration_and_spsc_command_dispatch() {
    let mut processor = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 32>::new();

    // Default state: Graphic EQ is enabled with flat 0.0 dB
    assert!(processor.graphic_eq.is_enabled());
    assert!(!processor.graphic_eq.is_bypassed());
    assert_eq!(processor.graphic_eq.target_gains()[17], 0.0);

    // 1. Dispatch SetEqBandGain
    queue
        .push(AudioCommand::SetEqBandGain {
            band_idx: 17,
            gain_db: 6.0,
        })
        .expect("Push must succeed");

    processor.drain_commands(&queue);
    assert_eq!(processor.graphic_eq.target_gains()[17], 6.0);

    // 2. Dispatch SetEqBypass
    queue
        .push(AudioCommand::SetEqBypass { bypassed: true })
        .expect("Push must succeed");

    processor.drain_commands(&queue);
    assert!(processor.graphic_eq.is_bypassed());
    assert!(!processor.graphic_eq.is_enabled());

    // 3. Dispatch LoadGraphicEqSnapshot
    let mut gains = [0.0; GRAPHIC_EQ_BANDS];
    gains[0] = 3.0; // 20 Hz
    gains[17] = -4.0; // 1 kHz
    gains[30] = 5.0; // 20 kHz

    queue
        .push(AudioCommand::LoadGraphicEqSnapshot {
            gains_db: gains,
            preamp_offset_db: -2.5,
        })
        .expect("Push must succeed");

    processor.drain_commands(&queue);
    assert_eq!(processor.graphic_eq.target_gains()[0], 3.0);
    assert_eq!(processor.graphic_eq.target_gains()[17], -4.0);
    assert_eq!(processor.graphic_eq.target_gains()[30], 5.0);
    assert_eq!(processor.graphic_eq.preamp_offset_db, -2.5);

    // 4. Dispatch ResetFilters
    queue.push(AudioCommand::ResetFilters).expect("Push must succeed");
    processor.drain_commands(&queue);

    for i in 0..GRAPHIC_EQ_BANDS {
        assert_eq!(processor.graphic_eq.filters_l[i].state(), (0.0, 0.0));
        assert_eq!(processor.graphic_eq.filters_r[i].state(), (0.0, 0.0));
    }
}

#[test]
fn test_coefficient_lerp_stability_intermediate_steps() {
    for &freq in &ISO_31_BAND_FREQUENCIES {
        let c_cut = BiquadCoefficients::peaking_eq(48_000.0, freq, EQ_Q, -12.0);
        let c_boost = BiquadCoefficients::peaking_eq(48_000.0, freq, EQ_Q, 12.0);

        for step in 0..=20 {
            let t = step as f32 / 20.0;
            let interpolated = c_cut.lerp(&c_boost, t);

            assert!(
                interpolated.is_stable(),
                "Interpolated filter unstable at freq={freq}, t={t}"
            );
            assert!(
                interpolated.max_pole_modulus() < 1.0,
                "Interpolated pole modulus >= 1.0 at freq={freq}, t={t}: |z| = {}",
                interpolated.max_pole_modulus()
            );
        }
    }
}

#[test]
fn test_pipeline_active_eq_boost_audio_processing() {
    use open_boom_core::dsp::pipeline::AudioProcessor;
    use open_boom_core::ring_buffer::SpscQueue;
    use std::f32::consts::PI;

    let mut processor = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 32>::new();

    // Configure +12 dB boost at 1 kHz (Band 17) via SPSC command
    queue
        .push(AudioCommand::SetEqBandGain {
            band_idx: 17,
            gain_db: 12.0,
        })
        .expect("Push must succeed");
    processor.drain_commands(&queue);
    processor.graphic_eq.flush_smoothing();

    // Generate 1 kHz sine wave at 48 kHz
    let sample_rate = 48_000.0f32;
    let frames = 4800; // 100 ms
    let mut in_buffer = vec![0.0f32; frames * 2];
    let mut out_buffer = vec![0.0f32; frames * 2];

    for i in 0..frames {
        let t = i as f32 / sample_rate;
        let s = (2.0 * PI * 1000.0 * t).sin() * 0.1; // -20 dBFS input to avoid limiter
        in_buffer[i * 2] = s;
        in_buffer[i * 2 + 1] = s;
    }

    processor.process_interleaved(&in_buffer, &mut out_buffer);

    // Evaluate steady-state output (last 1000 frames)
    let steady_start = frames - 1000;
    let mut in_energy = 0.0f32;
    let mut out_energy = 0.0f32;
    for i in steady_start..frames {
        in_energy += in_buffer[i * 2] * in_buffer[i * 2];
        out_energy += out_buffer[i * 2] * out_buffer[i * 2];
    }

    let in_rms = (in_energy / 1000.0).sqrt();
    let out_rms = (out_energy / 1000.0).sqrt();
    let measured_gain_db = 20.0 * (out_rms / in_rms).log10();

    // Measured gain should match +12 dB (within 0.3 dB tolerance)
    assert!(
        (measured_gain_db - 12.0).abs() < 0.3,
        "AudioProcessor output did not reflect +12 dB EQ boost! Measured: {measured_gain_db:.2} dB"
    );
}

#[test]
fn test_pipeline_reset_filters_clears_charged_state() {
    use open_boom_core::dsp::pipeline::AudioProcessor;
    use open_boom_core::ring_buffer::SpscQueue;

    let mut processor = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 32>::new();

    // 1. Charge filter delay lines with active EQ and sinusoidal signal
    processor.graphic_eq.set_band_gain_immediate(17, 6.0);

    let frames = 256;
    let in_buf: Vec<f32> = (0..frames * 2)
        .map(|i| ((i as f32) * 0.1).sin() * 0.5)
        .collect();
    let mut out_buf = vec![0.0f32; frames * 2];
    processor.process_interleaved(&in_buf, &mut out_buf);

    // Verify filter states are non-zero
    let mut has_non_zero_state = false;
    for i in 0..GRAPHIC_EQ_BANDS {
        let (s1, s2) = processor.graphic_eq.filters_l[i].state();
        if s1.abs() > 1e-4 || s2.abs() > 1e-4 {
            has_non_zero_state = true;
            break;
        }
    }
    assert!(has_non_zero_state, "Filters should be charged after processing audio");

    // 2. Dispatch ResetFilters
    queue.push(AudioCommand::ResetFilters).expect("Push must succeed");
    processor.drain_commands(&queue);

    // 3. Verify all states cleared back to (0.0, 0.0)
    for i in 0..GRAPHIC_EQ_BANDS {
        assert_eq!(
            processor.graphic_eq.filters_l[i].state(),
            (0.0, 0.0),
            "Filter L band {i} was not cleared"
        );
        assert_eq!(
            processor.graphic_eq.filters_r[i].state(),
            (0.0, 0.0),
            "Filter R band {i} was not cleared"
        );
    }
}

#[test]
fn test_pipeline_set_eq_gains_full_bank_dispatch() {
    use open_boom_core::dsp::pipeline::AudioProcessor;
    use open_boom_core::ring_buffer::SpscQueue;

    let mut processor = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 32>::new();

    let mut new_gains = [0.0f32; GRAPHIC_EQ_BANDS];
    for i in 0..GRAPHIC_EQ_BANDS {
        new_gains[i] = ((i as f32 % 5.0) - 2.0) * 3.0; // [-6, -3, 0, 3, 6] dB
    }

    queue.push(AudioCommand::SetEqGains { gains_db: new_gains }).expect("Push must succeed");
    processor.drain_commands(&queue);

    assert_eq!(processor.graphic_eq.target_gains(), &new_gains);
}

#[test]
fn test_graphic_eq_bypass_bit_perfect_dry_passthrough() {
    use open_boom_core::dsp::graphic_eq_stage::GraphicEqStage;

    let mut eq = GraphicEqStage::new();
    eq.set_band_gain_immediate(17, 12.0); // +12 dB at 1 kHz
    eq.set_bypass(true);

    let test_samples = [0.1f32, -0.4f32, 0.8f32, -0.2f32];
    for &s in &test_samples {
        let (out_l, out_r) = eq.process_stereo(s, s);
        assert_eq!(out_l, s, "Bypassed output must match input exactly");
        assert_eq!(out_r, s, "Bypassed output must match input exactly");
    }
}

#[test]
fn test_preamp_offset_attenuation_applied_to_audio() {
    use open_boom_core::dsp::graphic_eq_stage::GraphicEqStage;

    let mut eq = GraphicEqStage::new();
    eq.set_preamp_offset_db_immediate(-6.0); // -6 dB ~= 0.5011872

    let input = 1.0f32;
    let (out_l, out_r) = eq.process_stereo(input, input);

    let expected = 10.0f32.powf(-6.0 / 20.0);
    assert!((out_l - expected).abs() < 1e-4);
    assert!((out_r - expected).abs() < 1e-4);
}

#[test]
fn test_edge_frequencies_audio_processing_20hz_and_20khz() {
    use open_boom_core::dsp::graphic_eq_stage::GraphicEqStage;

    let mut eq = GraphicEqStage::new();
    eq.set_band_gain_immediate(0, 12.0); // 20 Hz
    eq.set_band_gain_immediate(30, 12.0); // 20 kHz

    for i in 0..1000 {
        let sample = ((i as f32) * 0.01).sin() * 0.1;
        let (out_l, out_r) = eq.process_stereo(sample, sample);
        assert!(out_l.is_finite(), "20Hz/20kHz band output must be finite");
        assert!(out_r.is_finite(), "20Hz/20kHz band output must be finite");
    }
}
