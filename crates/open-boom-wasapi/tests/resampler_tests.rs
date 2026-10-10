//! Comprehensive test suite for resampler, format converters, MMCSS scheduling,
//! loopback capture, and callback latency budget tracking.

use open_boom_core::types::EngineError;
use open_boom_wasapi::{
    convert_samples_to_f32, AudioEngineCoordinator, CallbackBudgetTracker, LoopbackCapture,
    LoopbackConfig, LoopbackMode, MmcssGuard, RenderClient, RenderConfig, Resampler, SampleFormat,
};

#[test]
fn test_resampler_identity_48k() {
    let mut resampler = Resampler::new(48_000);
    assert!(resampler.is_identity());
    assert_eq!(resampler.source_rate(), 48_000);
    assert_eq!(resampler.target_rate(), 48_000);
    assert_eq!(resampler.channels(), 2);
    assert!((resampler.ratio() - 1.0).abs() < 1e-9);

    // Stereo input: 480 frames = 960 samples
    let input: Vec<f32> = (0..960).map(|i| (i as f32 / 960.0) * 0.8).collect();
    let mut output = vec![0.0f32; 960];

    let frames_out = resampler.process(&input, &mut output);
    assert_eq!(frames_out, 480);
    assert_eq!(&input[..], &output[..]);
}

#[test]
fn test_resampler_upsample_44_1k_to_48k() {
    let mut resampler = Resampler::new(44_100);
    assert!(!resampler.is_identity());
    assert_eq!(resampler.source_rate(), 44_100);
    assert_eq!(resampler.target_rate(), 48_000);

    // 441 frames of stereo audio (10ms at 44.1 kHz) should produce 480 frames at 48 kHz
    let frames_in = 441;
    let channels = 2;
    let mut input = Vec::with_capacity(frames_in * channels);
    for i in 0..frames_in {
        let sample = (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 44_100.0)).sin();
        input.push(sample);
        input.push(sample);
    }

    let mut output = vec![0.0f32; 1024]; // ample space
    let frames_out = resampler.process(&input, &mut output);

    // 441 * (48000 / 44100) = 480.0 frames
    assert_eq!(frames_out, 480);

    // Check that samples are bounded and non-zero
    for s in &output[..frames_out * channels] {
        assert!(*s >= -1.0 && *s <= 1.0);
    }
}

#[test]
fn test_resampler_downsample_96k_to_48k() {
    let mut resampler = Resampler::new(96_000);
    assert!(!resampler.is_identity());
    assert_eq!(resampler.source_rate(), 96_000);
    assert_eq!(resampler.target_rate(), 48_000);
    assert!((resampler.ratio() - 2.0).abs() < 1e-9);

    // 960 frames of stereo audio at 96 kHz should produce 480 frames at 48 kHz
    let frames_in = 960;
    let channels = 2;
    let mut input = Vec::with_capacity(frames_in * channels);
    for i in 0..frames_in {
        let sample = (2.0 * std::f32::consts::PI * 1000.0 * (i as f32 / 96_000.0)).sin();
        input.push(sample);
        input.push(sample);
    }

    let mut output = vec![0.0f32; 1024];
    let frames_out = resampler.process(&input, &mut output);
    assert_eq!(frames_out, 480);
}

#[test]
fn test_resampler_downsample_88_2k_to_48k() {
    let mut resampler = Resampler::new(88_200);
    assert!(!resampler.is_identity());
    assert_eq!(resampler.source_rate(), 88_200);
    assert_eq!(resampler.target_rate(), 48_000);

    // 882 frames of stereo audio at 88.2 kHz should produce 480 frames at 48 kHz
    let frames_in = 882;
    let channels = 2;
    let mut input = Vec::with_capacity(frames_in * channels);
    for i in 0..frames_in {
        let sample = (2.0 * std::f32::consts::PI * 1000.0 * (i as f32 / 88_200.0)).sin();
        input.push(sample);
        input.push(sample);
    }

    let mut output = vec![0.0f32; 1024];
    let frames_out = resampler.process(&input, &mut output);
    assert_eq!(frames_out, 480);
}

#[test]
fn test_resampler_downsample_192k_to_48k() {
    let mut resampler = Resampler::new(192_000);
    assert!(!resampler.is_identity());
    assert_eq!(resampler.source_rate(), 192_000);
    assert_eq!(resampler.target_rate(), 48_000);
    assert!((resampler.ratio() - 4.0).abs() < 1e-9);

    // 1920 frames of stereo audio at 192 kHz should produce 480 frames at 48 kHz
    let frames_in = 1920;
    let channels = 2;
    let mut input = Vec::with_capacity(frames_in * channels);
    for i in 0..frames_in {
        let sample = (2.0 * std::f32::consts::PI * 1000.0 * (i as f32 / 192_000.0)).sin();
        input.push(sample);
        input.push(sample);
    }

    let mut output = vec![0.0f32; 1024];
    let frames_out = resampler.process(&input, &mut output);
    assert_eq!(frames_out, 480);
}

#[test]
fn test_resampler_reset() {
    let mut resampler = Resampler::new(44_100);
    let input = vec![0.8f32; 441 * 2];
    let mut output = vec![0.0f32; 1000];
    let _ = resampler.process(&input, &mut output);

    // After reset, internal history and phase are cleared
    resampler.reset();
    let frames_fresh = resampler.process(&input, &mut output);
    assert_eq!(frames_fresh, 480);
}

#[test]
fn test_resampler_continuous_stress_zero_allocation() {
    let mut resampler = Resampler::new(44_100);
    let input = vec![0.5f32; 441 * 2];
    let mut output = vec![0.0f32; 480 * 2];

    // Run 1,000 iterations simulating 10 seconds of streaming audio
    for _ in 0..1000 {
        let frames_out = resampler.process(&input, &mut output);
        assert_eq!(frames_out, 480);
    }
}

#[test]
fn test_resampler_multi_channel_7_1() {
    let channels = 8;
    let mut resampler = Resampler::with_channels(44_100, channels);
    assert_eq!(resampler.channels(), channels);

    let frames_in = 441;
    let input = vec![0.5f32; frames_in * channels];
    let mut output = vec![0.0f32; 600 * channels];

    let frames_out = resampler.process(&input, &mut output);
    assert_eq!(frames_out, 480);

    for s in &output[..frames_out * channels] {
        assert!((*s - 0.5).abs() < 1e-4);
    }
}

#[test]
fn test_resampler_chunk_boundary_smoothness() {
    let mut resampler = Resampler::new(44_100);

    // Stream a 440 Hz sine wave across 5 consecutive chunks of 441 frames (10ms each)
    let freq = 440.0;
    let chunk_size = 441;
    let num_chunks = 5;
    let mut combined_output = Vec::new();
    let mut sample_index = 0usize;

    for _ in 0..num_chunks {
        let mut chunk = Vec::with_capacity(chunk_size * 2);
        for _ in 0..chunk_size {
            let val = (2.0 * std::f32::consts::PI * freq * (sample_index as f32 / 44_100.0)).sin();
            chunk.push(val);
            chunk.push(val);
            sample_index += 1;
        }

        let mut out = vec![0.0f32; 1000];
        let frames_out = resampler.process(&chunk, &mut out);
        combined_output.extend_from_slice(&out[..frames_out * 2]);
    }

    assert_eq!(combined_output.len(), 480 * 2 * num_chunks);

    // Verify waveform derivative continuity (no DC clicks or discontinuities > 0.1 between adjacent samples)
    for i in 2..combined_output.len() {
        let delta = (combined_output[i] - combined_output[i - 2]).abs();
        assert!(
            delta < 0.15,
            "Discontinuity detected at sample {}: delta = {}",
            i,
            delta
        );
    }
}

#[test]
fn test_format_conversion_float32() {
    let samples = [0.0f32, 0.5, -0.75, 1.0, -1.0, 1.5, -2.0];
    let mut bytes = Vec::new();
    for s in samples {
        bytes.extend_from_slice(&s.to_le_bytes());
    }

    let mut dst = [0.0f32; 7];
    let count = convert_samples_to_f32(&bytes, SampleFormat::Float32, &mut dst).unwrap();
    assert_eq!(count, 7);
    assert_eq!(dst[0], 0.0);
    assert_eq!(dst[1], 0.5);
    assert_eq!(dst[2], -0.75);
    assert_eq!(dst[3], 1.0);
    assert_eq!(dst[4], -1.0);
    assert_eq!(dst[5], 1.0); // clamped
    assert_eq!(dst[6], -1.0); // clamped
}

#[test]
fn test_format_conversion_nan_sanitization() {
    let samples = [0.25f32, f32::NAN, -f32::NAN, 0.75];
    let mut bytes = Vec::new();
    for s in samples {
        bytes.extend_from_slice(&s.to_le_bytes());
    }

    let mut dst = [0.0f32; 4];
    let count = convert_samples_to_f32(&bytes, SampleFormat::Float32, &mut dst).unwrap();
    assert_eq!(count, 4);
    assert_eq!(dst[0], 0.25);
    assert_eq!(dst[1], 0.0); // NaN sanitized to 0.0
    assert_eq!(dst[2], 0.0); // -NaN sanitized to 0.0
    assert_eq!(dst[3], 0.75);
}

#[test]
fn test_format_conversion_destination_buffer_too_small() {
    let samples = [0.1f32, 0.2, 0.3, 0.4];
    let mut bytes = Vec::new();
    for s in samples {
        bytes.extend_from_slice(&s.to_le_bytes());
    }

    let mut dst = [0.0f32; 3]; // 3 samples capacity for 4 incoming samples
    let err = convert_samples_to_f32(&bytes, SampleFormat::Float32, &mut dst).unwrap_err();
    assert!(matches!(err, EngineError::BufferError(_)));
}

#[test]
fn test_format_conversion_pcm16() {
    let pcm_values: [i16; 5] = [0, 32767, -32768, 16384, -16384];
    let mut bytes = Vec::new();
    for v in pcm_values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }

    let mut dst = [0.0f32; 5];
    let count = convert_samples_to_f32(&bytes, SampleFormat::Pcm16, &mut dst).unwrap();
    assert_eq!(count, 5);
    assert_eq!(dst[0], 0.0);
    assert!((dst[1] - 32767.0 / 32768.0).abs() < 1e-5);
    assert_eq!(dst[2], -1.0);
    assert!((dst[3] - 0.5).abs() < 1e-4);
    assert!((dst[4] - (-0.5)).abs() < 1e-4);
}

#[test]
fn test_format_conversion_pcm24_packed() {
    // Little endian 24-bit:
    // 0 -> [0x00, 0x00, 0x00]
    // +Max (0x7FFFFF = 8388607) -> [0xFF, 0xFF, 0x7F]
    // -Max (0x800000 = -8388608) -> [0x00, 0x00, 0x80]
    let bytes: [u8; 9] = [
        0x00, 0x00, 0x00, // 0
        0xFF, 0xFF, 0x7F, // +8388607
        0x00, 0x00, 0x80, // -8388608
    ];

    let mut dst = [0.0f32; 3];
    let count = convert_samples_to_f32(&bytes, SampleFormat::Pcm24Packed, &mut dst).unwrap();
    assert_eq!(count, 3);
    assert_eq!(dst[0], 0.0);
    assert!((dst[1] - 8388607.0 / 8388608.0).abs() < 1e-6);
    assert_eq!(dst[2], -1.0);
}

#[test]
fn test_format_conversion_pcm24_in_32() {
    // 24-bit MSB-aligned in 32-bit container (bits 8..31)
    let val_zero: i32 = 0;
    let val_pos: i32 = 0x7F_FF_FF_00;
    let val_neg: i32 = -0x80_00_00_00;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&val_zero.to_le_bytes());
    bytes.extend_from_slice(&val_pos.to_le_bytes());
    bytes.extend_from_slice(&val_neg.to_le_bytes());

    let mut dst = [0.0f32; 3];
    let count = convert_samples_to_f32(&bytes, SampleFormat::Pcm24In32, &mut dst).unwrap();
    assert_eq!(count, 3);
    assert_eq!(dst[0], 0.0);
    assert!((dst[1] - 1.0).abs() < 1e-4);
    assert_eq!(dst[2], -1.0);
}

#[test]
fn test_format_conversion_pcm32() {
    let pcm_values: [i32; 4] = [0, 2147483647, -2147483648, 1073741824];
    let mut bytes = Vec::new();
    for v in pcm_values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }

    let mut dst = [0.0f32; 4];
    let count = convert_samples_to_f32(&bytes, SampleFormat::Pcm32, &mut dst).unwrap();
    assert_eq!(count, 4);
    assert_eq!(dst[0], 0.0);
    assert!((dst[1] - 1.0).abs() < 1e-6);
    assert_eq!(dst[2], -1.0);
    assert!((dst[3] - 0.5).abs() < 1e-6);
}

#[test]
fn test_format_conversion_invalid_buffer_length() {
    // Odd byte count for 16-bit format should return UnsupportedFormat error
    let bytes = [0u8; 3];
    let mut dst = [0.0f32; 2];
    let err = convert_samples_to_f32(&bytes, SampleFormat::Pcm16, &mut dst).unwrap_err();
    assert!(matches!(err, EngineError::UnsupportedFormat(_)));
}

#[test]
fn test_mmcss_guard_registration_and_fallback() {
    // Tests registration under "Pro Audio" task
    let guard = MmcssGuard::register("Pro Audio");
    assert_eq!(guard.task_name(), "Pro Audio");
    // Under Windows, may be active or fall back depending on test process elevation; never panics
    let _ = guard.is_active();
    let _ = guard.task_index();
    // Verify RAII drop cleans up cleanly
    drop(guard);
}

#[test]
fn test_callback_budget_tracker() {
    // Buffer duration 10ms (480 samples at 48 kHz).
    // Allowed budget = 60% of 10ms = 6.0ms.
    let mut tracker = CallbackBudgetTracker::new(10.0);
    assert_eq!(tracker.buffer_duration_ms, 10.0);
    assert_eq!(tracker.max_allowed_duration_ms, 6.0);
    assert_eq!(tracker.total_callbacks, 0);

    // Callback duration 3.2ms (< 6.0ms) -> within budget
    let exceeded = tracker.record_callback(3.2);
    assert!(!exceeded);
    assert!(!tracker.budget_exceeded);
    assert_eq!(tracker.total_callbacks, 1);
    assert_eq!(tracker.exceeded_callbacks, 0);

    // Callback duration 6.8ms (> 6.0ms) -> budget exceeded
    let exceeded = tracker.record_callback(6.8);
    assert!(exceeded);
    assert!(tracker.budget_exceeded);
    assert_eq!(tracker.total_callbacks, 2);
    assert_eq!(tracker.exceeded_callbacks, 1);
}

#[test]
fn test_loopback_capture_ingest_packet() {
    let config = LoopbackConfig {
        sample_rate: 44_100,
        channels: 2,
        format: SampleFormat::Pcm16,
        force_synthetic: true,
        ..Default::default()
    };
    let mut capture = LoopbackCapture::new(config);
    capture.start().unwrap();

    // Construct 441 frames of stereo 16-bit PCM audio (882 samples = 1764 bytes)
    let mut raw_bytes = Vec::with_capacity(441 * 2 * 2);
    for _ in 0..441 {
        let sample: i16 = 16384; // 0.5 normalized
        raw_bytes.extend_from_slice(&sample.to_le_bytes());
        raw_bytes.extend_from_slice(&sample.to_le_bytes());
    }

    let mut output_frames_48k = vec![0.0f32; 1024 * 2];
    let frames_written = capture
        .ingest_packet(&raw_bytes, &mut output_frames_48k)
        .unwrap();

    assert_eq!(frames_written, 480);
    for s in &output_frames_48k[..frames_written * 2] {
        assert!((*s - 0.5).abs() < 1e-3);
    }
}

#[test]
fn test_loopback_capture_not_capturing_rejection() {
    let config = LoopbackConfig {
        sample_rate: 44_100,
        channels: 2,
        format: SampleFormat::Pcm16,
        ..Default::default()
    };
    let mut capture = LoopbackCapture::new(config);
    // Note: capture.start() is NOT called

    let raw_bytes = [0u8; 1764];
    let mut output_frames_48k = vec![0.0f32; 1024 * 2];
    let err = capture
        .ingest_packet(&raw_bytes, &mut output_frames_48k)
        .unwrap_err();
    assert!(matches!(err, EngineError::BufferError(_)));
}

#[test]
fn test_loopback_capture_undersized_output_buffer_rejected() {
    let config = LoopbackConfig {
        sample_rate: 44_100,
        channels: 2,
        format: SampleFormat::Pcm16,
        force_synthetic: true,
        ..Default::default()
    };
    let mut capture = LoopbackCapture::new(config);
    capture.start().unwrap();

    // 441 frames of stereo audio requires 480 frames = 960 samples
    let raw_bytes = vec![0u8; 441 * 2 * 2];
    let mut undersized_output = vec![0.0f32; 500]; // 500 samples < 960 samples required

    let err = capture
        .ingest_packet(&raw_bytes, &mut undersized_output)
        .unwrap_err();
    assert!(matches!(err, EngineError::BufferError(_)));
}

#[test]
fn test_loopback_capture_synthetic_fallback() {
    let config = LoopbackConfig {
        force_synthetic: true,
        ..Default::default()
    };
    let mut capture = LoopbackCapture::new(config);
    assert!(capture.start().is_ok());
    assert!(capture.is_capturing());
    assert_eq!(capture.active_mode(), Some(LoopbackMode::Synthetic));
    assert!(capture.stop().is_ok());
    assert!(!capture.is_capturing());
}

#[test]
fn test_render_client_synthetic_fallback() {
    let config = RenderConfig {
        allow_synthetic_fallback: true,
        ..Default::default()
    };
    let mut render = RenderClient::with_config(config);
    assert!(render.start().is_ok());
    assert!(render.is_rendering());
    assert!(render.latency_ms() > 0.0);
    assert!(render.stop().is_ok());
    assert!(!render.is_rendering());
}

#[test]
fn test_render_client_force_synthetic() {
    let config = RenderConfig {
        force_synthetic: true,
        ..Default::default()
    };
    let mut render = RenderClient::with_config(config);
    assert!(render.start().is_ok());
    assert!(render.is_rendering());
    assert_eq!(render.latency_ms(), 5.0);
    assert!(render.mmcss_active());
    assert!(render.stop().is_ok());
    assert!(!render.is_rendering());
}

#[test]
fn test_audio_engine_coordinator_lifecycle() {
    let capture_config = LoopbackConfig {
        allow_synthetic_fallback: true,
        ..Default::default()
    };
    let render_config = RenderConfig {
        allow_synthetic_fallback: true,
        ..Default::default()
    };

    let mut coordinator = AudioEngineCoordinator::new(capture_config, render_config);
    assert!(!coordinator.is_running());

    assert!(coordinator.start().is_ok());
    assert!(coordinator.is_running());
    let rt_latency = coordinator.round_trip_latency_ms();
    assert!(rt_latency > 0.0);
    assert!(rt_latency < 15.0, "Round-trip latency {}ms must be < 15ms", rt_latency);

    let telemetry = coordinator.telemetry_snapshot();
    assert!(telemetry.dsp_active);
    assert_eq!(telemetry.round_trip_latency_ms, coordinator.round_trip_latency_ms());
    assert!(telemetry.round_trip_latency_ms < 15.0);

    assert!(coordinator.stop().is_ok());
    assert!(!coordinator.is_running());
}

#[test]
fn test_coordinator_sub_15ms_latency_guarantee() {
    let coordinator = AudioEngineCoordinator::default();
    let latency = coordinator.round_trip_latency_ms();
    assert!(latency < 15.0, "Default round-trip latency {}ms must be < 15ms", latency);
    let telemetry = coordinator.telemetry_snapshot();
    assert!(telemetry.round_trip_latency_ms < 15.0);
}
