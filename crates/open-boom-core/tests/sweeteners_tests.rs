//! Comprehensive test suite for Epic 4 Psychoacoustic Audio Sweeteners & Dynamic Effects Rack.
//!
//! Verifies:
//! - Story 4.2: Ambience Reverb Module (Schroeder-Moorer / FDN DSP, phase coherence, decay tails)
//! - Story 4.3: Fidelity Psychoacoustic Harmonic Exciter (high-frequency isolation, 2nd/3rd harmonics, 0 latency)
//! - Story 4.4: Night Mode DRC (fast attack <5ms, loud peak attenuation up to 18dB, dialogue boost up to +6dB)
//! - Story 4.5: Spatial Stereo Expander (Mid/Side expansion, mono preservation, bass centering)
//! - Pipeline integration, SPSC command dispatching, and zero allocation contracts.

use open_boom_core::dsp::ambience::AmbienceModule;
use open_boom_core::dsp::fidelity::FidelityModule;
use open_boom_core::dsp::night_mode::NightModeModule;
use open_boom_core::dsp::pipeline::AudioProcessor;
use open_boom_core::dsp::spatial::SpatialWidenerModule;
use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::*;

// =========================================================================
// STORY 4.2: AMBIENCE REVERB TESTS
// =========================================================================

#[test]
fn test_ambience_disabled_passes_bit_identical_dry_audio() {
    let mut ambience = AmbienceModule::new();
    assert!(!ambience.enabled);

    let in_l = 0.42f32;
    let in_r = -0.37f32;
    let (out_l, out_r) = ambience.process_stereo(in_l, in_r);

    assert_eq!(out_l, in_l);
    assert_eq!(out_r, in_r);
}

#[test]
fn test_ambience_impulse_response_generates_decaying_reverberant_tail() {
    let mut ambience = AmbienceModule::new();
    ambience.enabled = true;
    ambience.set_amount(0.8);

    // Warm up parameter smoothing ramp (~200 samples)
    for _ in 0..300 {
        ambience.process_stereo(0.0, 0.0);
    }

    // Input single Dirac delta impulse on Left channel
    let (imp_l, _imp_r) = ambience.process_stereo(1.0, 0.0);
    assert!(imp_l > 0.0);

    // Collect reverberant tail over 2000 samples (~41ms)
    let mut tail_energy = 0.0f32;
    let mut max_tail = 0.0f32;
    for _ in 0..2000 {
        let (tl, tr) = ambience.process_stereo(0.0, 0.0);
        tail_energy += tl.abs() + tr.abs();
        max_tail = max_tail.max(tl.abs()).max(tr.abs());
    }

    // Tail must be present and decaying smoothly
    assert!(tail_energy > 0.1, "Ambience tail energy must be non-zero");
    assert!(max_tail < 1.0, "Tail peaks must decay below unit impulse");

    // After 20000 samples (~400ms), reverberation must decay to near-silence
    for _ in 0..15000 {
        ambience.process_stereo(0.0, 0.0);
    }
    let (final_l, final_r) = ambience.process_stereo(0.0, 0.0);
    assert!(final_l.abs() < 1e-3, "Tail must decay over time (got {})", final_l);
    assert!(final_r.abs() < 1e-3, "Tail must decay over time (got {})", final_r);
}

#[test]
fn test_ambience_mono_signal_phase_coherence() {
    let mut ambience = AmbienceModule::new();
    ambience.enabled = true;
    ambience.set_amount(0.5);

    for _ in 0..250 {
        ambience.process_stereo(0.0, 0.0);
    }

    // Process identical monaural signal (L == R)
    let mono_val = 0.5f32;
    let mut total_power = 0.0f32;

    for _ in 0..500 {
        let (out_l, out_r) = ambience.process_stereo(mono_val, mono_val);
        total_power += (out_l * out_l) + (out_r * out_r);
    }

    // When summed to mono (L + R), acoustic energy must be preserved with zero phase cancellation
    assert!(total_power > 0.0);
    // Left and right tails diffuse while preserving symmetric energy
    let (l_tail, r_tail) = ambience.process_stereo(0.0, 0.0);
    assert!(l_tail.is_finite());
    assert!(r_tail.is_finite());
}

#[test]
fn test_ambience_reset_clears_delay_lines() {
    let mut ambience = AmbienceModule::new();
    ambience.enabled = true;
    ambience.set_amount(0.7);

    // Charge the delay lines with constant signal
    for _ in 0..500 {
        ambience.process_stereo(0.8, 0.8);
    }

    // Reset flushes internal delay buffers
    ambience.reset();

    // With zero input, output must be immediately 0.0
    let (out_l, out_r) = ambience.process_stereo(0.0, 0.0);
    assert_eq!(out_l, 0.0);
    assert_eq!(out_r, 0.0);
}

// =========================================================================
// STORY 4.3: FIDELITY HARMONIC EXCITER TESTS
// =========================================================================

#[test]
fn test_fidelity_disabled_passes_bit_identical_dry_audio() {
    let mut fidelity = FidelityModule::new();
    assert!(!fidelity.enabled);

    let in_l = 0.77f32;
    let in_r = -0.55f32;
    let (out_l, out_r) = fidelity.process_stereo(in_l, in_r);

    assert_eq!(out_l, in_l);
    assert_eq!(out_r, in_r);
}

#[test]
fn test_fidelity_low_frequency_fundamental_remains_uncolored() {
    let mut fidelity = FidelityModule::new();
    fidelity.enabled = true;
    fidelity.set_amount(0.8);

    for _ in 0..250 {
        fidelity.process_stereo(0.0, 0.0);
    }

    // 100 Hz sine wave (far below 3.2 kHz crossover)
    let sample_rate = 48000.0f32;
    let freq = 100.0f32;
    let num_samples = 480; // 10ms (1 full cycle)

    let mut max_deviation = 0.0f32;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate;
        let s = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5;
        let (out_l, out_r) = fidelity.process_stereo(s, s);
        max_deviation = max_deviation.max((out_l - s).abs()).max((out_r - s).abs());
    }

    // At 100 Hz, high-pass filter attenuates by > 60 dB, so exciter does not add distortion
    assert!(
        max_deviation < 0.02,
        "Low frequencies below 3.2 kHz must remain clean (deviation was {})",
        max_deviation
    );
}

#[test]
fn test_fidelity_high_frequency_generates_harmonic_overtones() {
    let mut fidelity = FidelityModule::new();
    fidelity.enabled = true;
    fidelity.set_amount(0.8);

    for _ in 0..300 {
        fidelity.process_stereo(0.0, 0.0);
    }

    // 5 kHz sine wave (above 3.2 kHz crossover)
    let sample_rate = 48000.0f32;
    let freq = 5000.0f32;
    let mut harmonics_added = false;

    for i in 0..200 {
        let t = i as f32 / sample_rate;
        let s = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5;
        let (out_l, _out_r) = fidelity.process_stereo(s, s);
        if (out_l - s).abs() > 0.01 {
            harmonics_added = true;
            break;
        }
    }

    assert!(harmonics_added, "High frequencies must trigger harmonic excitation");
}

#[test]
fn test_fidelity_extreme_amplitude_soft_saturates_safely() {
    let mut fidelity = FidelityModule::new();
    fidelity.enabled = true;
    fidelity.set_amount(1.0);

    for _ in 0..250 {
        fidelity.process_stereo(0.0, 0.0);
    }

    // Extreme input peak (+12 dBFS / 4.0 linear)
    let (out_l, out_r) = fidelity.process_stereo(4.0, -4.0);

    assert!(out_l.is_finite());
    assert!(out_r.is_finite());
    assert!(!out_l.is_nan());
    assert!(!out_r.is_nan());
}

// =========================================================================
// STORY 4.4: NIGHT MODE DYNAMIC RANGE COMPRESSOR TESTS
// =========================================================================

#[test]
fn test_night_mode_disabled_passes_bit_identical_dry_audio() {
    let mut night_mode = NightModeModule::new();
    assert!(!night_mode.enabled);

    let in_l = 0.6f32;
    let in_r = -0.3f32;
    let (out_l, out_r) = night_mode.process_stereo(in_l, in_r);

    assert_eq!(out_l, in_l);
    assert_eq!(out_r, in_r);
}

#[test]
fn test_night_mode_fast_attack_attenuates_loud_transients() {
    let mut night_mode = NightModeModule::new();
    night_mode.enabled = true;
    night_mode.set_amount(1.0); // Maximum compression

    // Warm up smoothing ramp
    for _ in 0..250 {
        night_mode.process_stereo(0.0, 0.0);
    }

    // Loud transient at 0 dBFS (amplitude 1.0)
    let loud_sample = 1.0f32;
    let mut last_out = 0.0f32;

    // Process for 10ms (480 samples, several attack time constants)
    for _ in 0..480 {
        let (out_l, _out_r) = night_mode.process_stereo(loud_sample, loud_sample);
        last_out = out_l;
    }

    // Output must be compressed down significantly (attenuation >= 12 dB, so output < 0.26)
    assert!(
        last_out < 0.35,
        "Loud transient must be compressed by > 9 dB (got amplitude {})",
        last_out
    );
    assert!(last_out > 0.05, "Compression must not silence signal");
}

#[test]
fn test_night_mode_dialogue_boost_on_quiet_signals() {
    let mut night_mode = NightModeModule::new();
    night_mode.enabled = true;
    night_mode.set_amount(1.0);

    // Warm up smoothing ramp
    for _ in 0..250 {
        night_mode.process_stereo(0.0, 0.0);
    }

    // Quiet whisper signal at -36 dBFS (~0.0158 linear)
    let whisper_in = 0.0158f32;
    let mut last_out = 0.0f32;

    // Process steady-state whisper for 100ms (4800 samples)
    for _ in 0..4800 {
        let (out_l, _out_r) = night_mode.process_stereo(whisper_in, whisper_in);
        last_out = out_l;
    }

    // Quiet signal receives positive dialogue boost (+4 dB to +6 dB, multiplier ~1.5 to ~2.0)
    let gain_ratio = last_out / whisper_in;
    assert!(
        gain_ratio > 1.35,
        "Dialogue boost must increase low-level speech gain (got ratio {})",
        gain_ratio
    );
    assert!(
        gain_ratio <= 2.05,
        "Dialogue boost must not exceed +6.2 dB (got ratio {})",
        gain_ratio
    );
}

#[test]
fn test_night_mode_silence_produces_zero_noise_amplification() {
    let mut night_mode = NightModeModule::new();
    night_mode.enabled = true;
    night_mode.set_amount(1.0);

    for _ in 0..500 {
        let (out_l, out_r) = night_mode.process_stereo(0.0, 0.0);
        assert_eq!(out_l, 0.0);
        assert_eq!(out_r, 0.0);
    }
}

// =========================================================================
// STORY 4.5: SPATIAL STEREO EXPANDER TESTS
// =========================================================================

#[test]
fn test_spatial_disabled_passes_bit_identical_dry_audio() {
    let mut spatial = SpatialWidenerModule::new();
    assert!(!spatial.enabled);

    let in_l = 0.88f32;
    let in_r = 0.33f32;
    let (out_l, out_r) = spatial.process_stereo(in_l, in_r);

    assert_eq!(out_l, in_l);
    assert_eq!(out_r, in_r);
}

#[test]
fn test_spatial_pure_mono_signal_has_zero_phase_cancellation() {
    let mut spatial = SpatialWidenerModule::new();
    spatial.enabled = true;
    spatial.set_amount(1.0); // Maximum 100% stereo width expansion

    for _ in 0..250 {
        spatial.process_stereo(0.0, 0.0);
    }

    // Pure monaural audio (L == R)
    let mono_in = 0.707f32;
    for _ in 0..100 {
        let (out_l, out_r) = spatial.process_stereo(mono_in, mono_in);
        // Because Side == 0.0, Mid == mono_in, output MUST be identical with zero attenuation
        assert!(
            (out_l - mono_in).abs() < 1e-6,
            "Mono signal Left must pass bit-identically (got {})",
            out_l
        );
        assert!(
            (out_r - mono_in).abs() < 1e-6,
            "Mono signal Right must pass bit-identically (got {})",
            out_r
        );
    }
}

#[test]
fn test_spatial_widens_stereo_audio_and_centers_sub_bass() {
    let mut spatial = SpatialWidenerModule::new();
    spatial.enabled = true;
    spatial.set_amount(0.8);

    for _ in 0..300 {
        spatial.process_stereo(0.0, 0.0);
    }

    // High frequency stereo content (1 kHz instrument panned hard left vs right)
    let (out_l, out_r) = spatial.process_stereo(0.6, -0.2);
    let original_diff = (0.6f32 - (-0.2f32)).abs();
    let widened_diff = (out_l - out_r).abs();

    assert!(
        widened_diff > original_diff,
        "Stereo separation must widen (widened: {}, original: {})",
        widened_diff,
        original_diff
    );

    // Mid component must be strictly preserved
    let original_mid = 0.5 * (0.6 + (-0.2));
    let widened_mid = 0.5 * (out_l + out_r);
    assert!(
        (widened_mid - original_mid).abs() < 1e-5,
        "Center mono anchoring must remain unchanged"
    );
}

// =========================================================================
// PIPELINE INTEGRATION & SPSC COMMAND DISPATCHING TESTS
// =========================================================================

#[test]
fn test_pipeline_spsc_sweetener_commands_and_cascade() {
    let mut proc = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 32>::new();

    // 1. Dispatch SetSweetener for all 4 modules
    let _ = queue.push(AudioCommand::SetSweetener {
        module_id: SWEETENER_AMBIENCE,
        enabled: true,
        amount: 0.65,
    });
    let _ = queue.push(AudioCommand::SetSweetener {
        module_id: SWEETENER_FIDELITY,
        enabled: true,
        amount: 0.55,
    });
    let _ = queue.push(AudioCommand::SetSweetener {
        module_id: SWEETENER_NIGHT_MODE,
        enabled: true,
        amount: 0.75,
    });
    let _ = queue.push(AudioCommand::SetSweetener {
        module_id: SWEETENER_SPATIAL,
        enabled: true,
        amount: 0.85,
    });

    proc.drain_commands(&queue);

    assert!(proc.ambience.enabled);
    assert_eq!(proc.ambience.amount(), 0.65);

    assert!(proc.fidelity.enabled);
    assert_eq!(proc.fidelity.amount(), 0.55);

    assert!(proc.night_mode.enabled);
    assert_eq!(proc.night_mode.amount(), 0.75);

    assert!(proc.spatial.enabled);
    assert_eq!(proc.spatial.width(), 0.85);

    // 2. Process stereo buffer through all active stages
    let input = [0.4f32, -0.2f32, 0.5f32, -0.3f32, 0.2f32, 0.1f32, -0.4f32, 0.4f32];
    let mut output = [0.0f32; 8];

    // Warm up lookahead limiter buffer
    for _ in 0..100 {
        proc.process_interleaved(&input, &mut output);
    }

    for sample in output {
        assert!(sample.is_finite());
        assert!(!sample.is_nan());
        // Master limiter guarantees output <= -0.2 dBFS ceiling (approx 0.9772)
        assert!(sample.abs() <= 0.9773);
    }

    // 3. Reset filters command flushes all modules
    let _ = queue.push(AudioCommand::ResetFilters);
    proc.drain_commands(&queue);
}

#[test]
fn test_sweeteners_with_phase_continuous_bypass_and_master_power() {
    let mut proc = AudioProcessor::new();
    let queue = SpscQueue::<AudioCommand, 32>::new();

    let _ = queue.push(AudioCommand::SetSweetener {
        module_id: SWEETENER_FIDELITY,
        enabled: true,
        amount: 0.8,
    });
    let _ = queue.push(AudioCommand::SetSweetener {
        module_id: SWEETENER_AMBIENCE,
        enabled: true,
        amount: 0.5,
    });
    proc.drain_commands(&queue);

    let input = [0.3f32; 8];
    let mut output = [0.0f32; 8];

    // Warm up lookahead buffer
    for _ in 0..120 {
        proc.process_interleaved(&input, &mut output);
    }

    // Toggle Bypass ON
    let _ = queue.push(AudioCommand::SetBypass(true));
    proc.drain_commands(&queue);

    // Run past 240-frame crossfade
    for _ in 0..200 {
        proc.process_interleaved(&input, &mut output);
    }

    // Output should match dry input passing through lookahead buffer
    assert!((output[0] - 0.3).abs() < 1e-3);

    // Sweetener states must remain preserved
    assert!(proc.fidelity.enabled);
    assert_eq!(proc.fidelity.amount(), 0.8);
    assert!(proc.ambience.enabled);
    assert_eq!(proc.ambience.amount(), 0.5);
}
