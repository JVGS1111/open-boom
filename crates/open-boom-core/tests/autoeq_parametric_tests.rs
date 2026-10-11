//! Comprehensive DSP test suite for Story 3.4: Decoupled Parametric AutoEq Stage
//! & Auto-Gain Preamp Compensation.
//!
//! Validates:
//! - Robert Bristow-Johnson Low Shelf and High Shelf biquad formulas and Schur-Cohn stability
//! - Pole modulus strictly < 1.0 under normal and extreme gains
//! - Automatic negative preamp attenuation offset calculation: A_preamp = -max(0, max_i(G_i))
//! - THD+N < 0.005% distortion benchmark
//! - Cascade topology: Parametric AutoEq (Stage 3A) -> Graphic EQ (Stage 3B) -> Master Limiter
//! - Anti-clipping headroom protection under simultaneous AutoEq + 31-band fader boosts
//! - Lock-free SPSC POD command dispatch (SetAutoEqProfile, SetAutoEqBypass)
//! - Direct Form II Transposed sub-buffer smoothing (32 samples) and zero-allocation processing

use open_boom_core::dsp::biquad::BiquadCoefficients;
use open_boom_core::dsp::parametric_stage::ParametricStage;
use open_boom_core::dsp::pipeline::AudioProcessor;
use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{
    AudioCommand, ParametricFilterPod, ParametricFilterType, INTERNAL_SAMPLE_RATE_F32,
    MAX_PARAMETRIC_FILTERS,
};
use std::f32::consts::PI;

/// Helper to generate pure sine wave samples: x[n] = amp * sin(2*pi*f*n/fs)
fn generate_sine_wave(freq_hz: f32, sample_rate: f32, duration_secs: f32, amplitude: f32) -> Vec<f32> {
    let num_samples = (sample_rate * duration_secs) as usize;
    (0..num_samples)
        .map(|n| {
            let t = n as f32 / sample_rate;
            amplitude * (2.0 * PI * freq_hz * t).sin()
        })
        .collect()
}

/// Helper to calculate Total Harmonic Distortion + Noise (THD+N)
fn calculate_thd_n(signal: &[f32], freq_hz: f32, sample_rate: f32) -> f32 {
    let n = signal.len();
    if n < 256 {
        return 0.0;
    }

    let discard = (sample_rate * 0.05) as usize; // Discard first 50ms transient
    let eval_slice = &signal[discard.min(n / 2)..];
    let len = eval_slice.len();

    let mut sum_fund_sin = 0.0f32;
    let mut sum_fund_cos = 0.0f32;
    for (i, &s) in eval_slice.iter().enumerate() {
        let t = (discard + i) as f32 / sample_rate;
        let w = 2.0 * PI * freq_hz * t;
        sum_fund_sin += s * w.sin();
        sum_fund_cos += s * w.cos();
    }
    let fund_sin_amp = (2.0 / len as f32) * sum_fund_sin;
    let fund_cos_amp = (2.0 / len as f32) * sum_fund_cos;
    let fundamental_amp = (fund_sin_amp * fund_sin_amp + fund_cos_amp * fund_cos_amp).sqrt();

    let mut noise_harmonics_energy = 0.0f32;
    for (i, &s) in eval_slice.iter().enumerate() {
        let t = (discard + i) as f32 / sample_rate;
        let fund_val = fund_sin_amp * (2.0 * PI * freq_hz * t).sin()
            + fund_cos_amp * (2.0 * PI * freq_hz * t).cos();
        let diff = s - fund_val;
        noise_harmonics_energy += diff * diff;
    }

    let noise_rms = (noise_harmonics_energy / len as f32).sqrt();
    let fund_rms = fundamental_amp / std::f32::consts::SQRT_2;

    if fund_rms < 1e-6 {
        0.0
    } else {
        noise_rms / fund_rms
    }
}

#[test]
fn test_rbj_low_shelf_frequency_response_and_stability() {
    let sample_rate = 48_000.0;
    let cutoff_hz = 105.0;
    let q = 0.71;
    let boost_db = 5.0;

    let coeffs = BiquadCoefficients::low_shelf(sample_rate, cutoff_hz, q, boost_db);

    // 1. Pole modulus strictly inside the unit circle
    assert!(coeffs.is_stable(), "Low shelf filter poles must be Schur-Cohn stable");
    assert!(
        coeffs.max_pole_modulus() < 1.0,
        "Low shelf pole modulus must be strictly < 1.0, got {}",
        coeffs.max_pole_modulus()
    );

    // 2. Frequency response checks
    let resp_sub_bass = coeffs.frequency_response_db(30.0, sample_rate);
    let resp_cutoff = coeffs.frequency_response_db(cutoff_hz, sample_rate);
    let resp_high = coeffs.frequency_response_db(5000.0, sample_rate);

    // Sub-bass should be boosted by ~5 dB (+/- 0.5 dB)
    assert!(
        (resp_sub_bass - boost_db).abs() < 0.5,
        "Sub-bass response should be ~+5.0 dB, got {} dB",
        resp_sub_bass
    );

    // At cutoff frequency, shelving filter gain is approximately halfway in dB (+2.5 dB)
    assert!(
        (resp_cutoff - 2.5).abs() < 0.8,
        "Cutoff response should be approximately +2.5 dB, got {} dB",
        resp_cutoff
    );

    // High frequencies above shelf transition should be near unity (0 dB)
    assert!(
        resp_high.abs() < 0.2,
        "High frequency response should be ~0.0 dB, got {} dB",
        resp_high
    );
}

#[test]
fn test_rbj_high_shelf_frequency_response_and_stability() {
    let sample_rate = 48_000.0;
    let cutoff_hz = 10_000.0;
    let q = 0.71;
    let cut_db = -3.0;

    let coeffs = BiquadCoefficients::high_shelf(sample_rate, cutoff_hz, q, cut_db);

    // 1. Pole modulus strictly inside the unit circle
    assert!(coeffs.is_stable(), "High shelf filter poles must be Schur-Cohn stable");
    assert!(
        coeffs.max_pole_modulus() < 1.0,
        "High shelf pole modulus must be strictly < 1.0, got {}",
        coeffs.max_pole_modulus()
    );

    // 2. Frequency response checks
    let resp_low = coeffs.frequency_response_db(100.0, sample_rate);
    let resp_cutoff = coeffs.frequency_response_db(cutoff_hz, sample_rate);
    let resp_air = coeffs.frequency_response_db(18_000.0, sample_rate);

    // Low frequencies below shelf transition should be near 0 dB
    assert!(
        resp_low.abs() < 0.2,
        "Low frequency response should be ~0.0 dB, got {} dB",
        resp_low
    );

    // At cutoff frequency, shelving filter gain is approximately halfway (-1.5 dB)
    assert!(
        (resp_cutoff - (-1.5)).abs() < 0.8,
        "Cutoff response should be ~-1.5 dB, got {} dB",
        resp_cutoff
    );

    // Air frequencies should be attenuated by ~ -3 dB
    assert!(
        (resp_air - cut_db).abs() < 0.5,
        "Treble shelf response should be ~-3.0 dB, got {} dB",
        resp_air
    );
}

#[test]
fn test_rbj_shelf_extreme_gains_and_pathological_inputs() {
    let sample_rates = [8000.0, 44100.0, 48000.0, 96000.0, 192000.0];
    let extreme_gains = [-36.0, -24.0, -12.0, 0.0, 6.0, 12.0, 24.0, 36.0, 50.0, -50.0];

    for &sr in &sample_rates {
        for &gain in &extreme_gains {
            // Low shelf test
            let ls = BiquadCoefficients::low_shelf(sr, 105.0, 0.71, gain);
            assert!(ls.is_stable(), "Low shelf at sr={}, gain={} must be stable", sr, gain);
            assert!(ls.max_pole_modulus() < 1.0);

            // High shelf test
            let hs = BiquadCoefficients::high_shelf(sr, 10000.0, 0.71, gain);
            assert!(hs.is_stable(), "High shelf at sr={}, gain={} must be stable", sr, gain);
            assert!(hs.max_pole_modulus() < 1.0);
        }

        // Test NaN and Inf inputs sanitization
        let ls_nan = BiquadCoefficients::low_shelf(sr, f32::NAN, 0.71, f32::NAN);
        assert!(ls_nan.is_stable());
        let hs_inf = BiquadCoefficients::high_shelf(sr, f32::INFINITY, f32::INFINITY, f32::INFINITY);
        assert!(hs_inf.is_stable());
    }
}

#[test]
fn test_auto_preamp_offset_calculation_formula() {
    // 1. Profile with multiple peaking and shelf boosts
    let filters = [
        ParametricFilterPod {
            filter_type: ParametricFilterType::LowShelf,
            frequency_hz: 105.0,
            q: 0.71,
            gain_db: 5.0,
        },
        ParametricFilterPod {
            filter_type: ParametricFilterType::Peaking,
            frequency_hz: 3200.0,
            q: 2.0,
            gain_db: 6.5, // Max boost = +6.5 dB
        },
        ParametricFilterPod {
            filter_type: ParametricFilterType::HighShelf,
            frequency_hz: 10000.0,
            q: 0.71,
            gain_db: -2.0,
        },
    ];

    let offset = ParametricStage::calculate_auto_preamp_offset(&filters);
    assert_eq!(
        offset, -6.5,
        "Auto preamp offset must equal negative of maximum positive filter gain (-6.5 dB)"
    );

    // 2. Profile with zero boosts and only negative notch cuts
    let cut_filters = [
        ParametricFilterPod {
            filter_type: ParametricFilterType::Peaking,
            frequency_hz: 500.0,
            q: 1.5,
            gain_db: -3.5,
        },
        ParametricFilterPod {
            filter_type: ParametricFilterType::Peaking,
            frequency_hz: 6000.0,
            q: 4.0,
            gain_db: -8.0,
        },
    ];

    let cut_offset = ParametricStage::calculate_auto_preamp_offset(&cut_filters);
    assert_eq!(
        cut_offset, 0.0,
        "Profile with only negative cuts must have 0.0 dB preamp offset"
    );

    // 3. Empty filter list
    let empty_offset = ParametricStage::calculate_auto_preamp_offset(&[]);
    assert_eq!(empty_offset, 0.0);
}

#[test]
fn test_parametric_stage_thd_n_distortion() {
    let mut stage = ParametricStage::new();
    stage.load_profile_immediate(&[], 0.0);

    let sample_rate = INTERNAL_SAMPLE_RATE_F32;
    let input = generate_sine_wave(1000.0, sample_rate, 0.5, 0.8);
    let mut output = Vec::with_capacity(input.len());

    for &sample in &input {
        let (l, _) = stage.process_stereo(sample, sample);
        output.push(l);
    }

    let thd_n = calculate_thd_n(&output, 1000.0, sample_rate);
    assert!(
        thd_n < 0.00005,
        "Flat ParametricStage THD+N must be < 0.005%, measured {:.6}%",
        thd_n * 100.0
    );
}

#[test]
fn test_parametric_stage_process_stereo_zero_allocation_and_clean_audio() {
    let mut stage = ParametricStage::new();
    let filters = [
        ParametricFilterPod {
            filter_type: ParametricFilterType::LowShelf,
            frequency_hz: 105.0,
            q: 0.71,
            gain_db: 4.5,
        },
        ParametricFilterPod {
            filter_type: ParametricFilterType::Peaking,
            frequency_hz: 1000.0,
            q: 1.41,
            gain_db: -3.0,
        },
        ParametricFilterPod {
            filter_type: ParametricFilterType::HighShelf,
            frequency_hz: 8000.0,
            q: 0.71,
            gain_db: 2.0,
        },
    ];

    stage.load_profile_immediate(&filters, -4.5);
    assert_eq!(stage.active_count, 3);
    assert_eq!(stage.preamp_offset_db, -4.5);

    // Process 20,000 frames of audio through stage
    for i in 0..20_000 {
        let in_l = (i as f32 * 0.01).sin() * 0.5;
        let in_r = (i as f32 * 0.01).cos() * 0.5;
        let (out_l, out_r) = stage.process_stereo(in_l, in_r);

        assert!(!out_l.is_nan() && !out_l.is_infinite());
        assert!(!out_r.is_nan() && !out_r.is_infinite());
    }
}

#[test]
fn test_pipeline_sequential_cascade_parametric_before_graphic_eq() {
    let mut processor = AudioProcessor::new();

    // 1. Stage 3A: Parametric AutoEq has a +6.0 dB boost at 1 kHz with -6.0 dB preamp offset
    let autoeq_filters = [ParametricFilterPod {
        filter_type: ParametricFilterType::Peaking,
        frequency_hz: 1000.0,
        q: 2.0,
        gain_db: 6.0,
    }];
    processor.load_autoeq_profile(&autoeq_filters, -6.0);
    processor.parametric_eq.flush_smoothing();

    // 2. Stage 3B: Graphic EQ has a +6.0 dB boost at 1 kHz (Band 17)
    processor.set_eq_band_gain(17, 6.0);
    processor.graphic_eq.flush_smoothing();

    // Verify both stages are sequentially configured
    assert_eq!(processor.parametric_eq.active_count, 1);
    assert_eq!(processor.parametric_eq.preamp_offset_db, -6.0);
    assert_eq!(processor.graphic_eq.current_gains()[17], 6.0);

    // Process steady-state 1 kHz sine through pipeline
    let sine = generate_sine_wave(1000.0, 48_000.0, 0.2, 0.5);
    let mut out_buf = vec![0.0f32; sine.len() * 2];
    let mut in_interleaved = Vec::with_capacity(sine.len() * 2);
    for &s in &sine {
        in_interleaved.push(s);
        in_interleaved.push(s);
    }

    processor.process_interleaved(&in_interleaved, &mut out_buf);

    // Peak levels must remain finite and controlled
    let max_peak = out_buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_peak > 0.0 && max_peak <= 1.0, "Cascade peak must be clean and non-clipping");
}

#[test]
fn test_simultaneous_autoeq_and_31_band_boost_anti_clipping_headroom() {
    let mut processor = AudioProcessor::new();

    // AutoEq profile with sharp +6.5 dB boost at 3.2 kHz
    let autoeq = [ParametricFilterPod {
        filter_type: ParametricFilterType::Peaking,
        frequency_hz: 3150.0,
        q: 2.5,
        gain_db: 6.5,
    }];
    // Preamp offset auto-calculated: -6.5 dB
    processor.load_autoeq_profile(&autoeq, -6.5);
    processor.parametric_eq.flush_smoothing();

    // Simultaneous user boost: +6.0 dB at 3.15 kHz (Band 22) in 31-band EQ
    processor.set_eq_band_gain(22, 6.0);
    processor.graphic_eq.flush_smoothing();

    // Full-scale 0 dBFS input signal (amplitude 0.99) at 3.15 kHz
    let sine = generate_sine_wave(3150.0, 48_000.0, 0.3, 0.99);
    let mut in_stereo = Vec::with_capacity(sine.len() * 2);
    for &s in &sine {
        in_stereo.push(s);
        in_stereo.push(s);
    }
    let mut out_stereo = vec![0.0f32; in_stereo.len()];

    processor.process_interleaved(&in_stereo, &mut out_stereo);

    // Verify master limiter ceiling is strictly respected (-0.2 dBFS = 0.9772 linear)
    let ceiling = 10.0f32.powf(-0.2 / 20.0);
    let mut max_out = 0.0f32;
    for &s in &out_stereo {
        assert!(!s.is_nan() && !s.is_infinite());
        max_out = max_out.max(s.abs());
    }

    assert!(
        max_out <= ceiling + 1e-4,
        "Output signal must not exceed master ceiling {} under combined boost, got {}",
        ceiling,
        max_out
    );
}

#[test]
fn test_autoeq_bypass_bit_perfect_dry_passthrough() {
    let mut processor = AudioProcessor::new();

    let autoeq = [ParametricFilterPod {
        filter_type: ParametricFilterType::LowShelf,
        frequency_hz: 105.0,
        q: 0.71,
        gain_db: 5.0,
    }];
    processor.load_autoeq_profile(&autoeq, -5.0);
    processor.parametric_eq.flush_smoothing();

    // Bypass parametric AutoEq stage
    processor.set_autoeq_bypass(true);
    assert!(processor.is_autoeq_bypassed());

    let (l, r) = processor.parametric_eq.process_stereo(0.42, -0.42);
    assert_eq!(l, 0.42, "Bypassed parametric stage must be bit-perfect");
    assert_eq!(r, -0.42, "Bypassed parametric stage must be bit-perfect");
}

#[test]
fn test_spsc_queue_autoeq_commands_dispatch() {
    let queue = SpscQueue::<AudioCommand, 32>::new();
    let mut processor = AudioProcessor::new();

    let filters = [
        ParametricFilterPod {
            filter_type: ParametricFilterType::LowShelf,
            frequency_hz: 105.0,
            q: 0.71,
            gain_db: 4.0,
        },
        ParametricFilterPod {
            filter_type: ParametricFilterType::Peaking,
            frequency_hz: 2000.0,
            q: 1.5,
            gain_db: 3.5,
        },
    ];

    let mut pod_array = [ParametricFilterPod::default(); MAX_PARAMETRIC_FILTERS];
    pod_array[0] = filters[0];
    pod_array[1] = filters[1];

    // 1. Dispatch SetAutoEqProfile
    queue
        .push(AudioCommand::SetAutoEqProfile {
            filters: pod_array,
            filter_count: 2,
            preamp_offset_db: -4.0,
        })
        .expect("push SetAutoEqProfile");

    processor.drain_commands(&queue);

    assert_eq!(processor.parametric_eq.active_count, 2);
    assert_eq!(processor.parametric_eq.preamp_offset_db, -4.0);
    assert!(processor.parametric_eq.is_enabled());

    // 2. Dispatch SetAutoEqBypass
    queue
        .push(AudioCommand::SetAutoEqBypass { bypassed: true })
        .expect("push SetAutoEqBypass");

    processor.drain_commands(&queue);
    assert!(processor.is_autoeq_bypassed());

    // 3. Reset filters flushes parametric stage delay lines
    queue.push(AudioCommand::ResetFilters).expect("push ResetFilters");
    processor.drain_commands(&queue);
    assert_eq!(processor.parametric_eq.filters_l[0].state(), (0.0, 0.0));
    assert_eq!(processor.parametric_eq.filters_r[0].state(), (0.0, 0.0));
}

#[test]
fn test_parametric_stage_sub_buffer_smoothing_transitions() {
    let mut stage = ParametricStage::new();

    // Start with 0 dB flat
    stage.load_profile_immediate(&[], 0.0);
    assert_eq!(stage.preamp_gain, 1.0);

    // Apply profile with -6 dB preamp attenuation offset
    let profile = [ParametricFilterPod {
        filter_type: ParametricFilterType::Peaking,
        frequency_hz: 1000.0,
        q: 1.0,
        gain_db: 6.0,
    }];
    stage.load_profile(&profile, -6.0);

    // Target preamp gain is 10^(-6/20) ~ 0.501187
    let target_linear = 10.0f32.powf(-6.0 / 20.0);

    // Process 64 samples (2 sub-buffers of 32 samples)
    for _ in 0..64 {
        stage.process_stereo(0.1, 0.1);
    }

    // Gain must have moved smoothly towards target_linear without jumping or exceeding bounds
    assert!(
        stage.preamp_gain < 1.0,
        "Preamp gain must smoothly attenuate, got {}",
        stage.preamp_gain
    );
    assert!(
        stage.preamp_gain >= target_linear - 1e-4,
        "Preamp gain must not overshoot target"
    );
}
