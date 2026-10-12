//! Unit and integration tests for real-time WASAPI AudioStreamPump lifecycle.

use open_boom_wasapi::coordinator::AudioEngineCoordinator;
use open_boom_wasapi::loopback_capture::LoopbackConfig;
use open_boom_wasapi::render_client::RenderConfig;
use open_boom_wasapi::stream_pump::{AudioStreamPump, StreamPumpConfig};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_audio_stream_pump_lifecycle() {
    let coordinator = Arc::new(Mutex::new(AudioEngineCoordinator::new(
        LoopbackConfig::default(),
        RenderConfig {
            allow_synthetic_fallback: true,
            ..Default::default()
        },
    )));

    let config = StreamPumpConfig {
        allow_synthetic: true,
        ..Default::default()
    };

    let mut pump = AudioStreamPump::start(coordinator, config).expect("start pump");
    assert!(pump.is_running(), "Audio pump should be running after start");

    // Allow pump thread to run briefly
    std::thread::sleep(Duration::from_millis(100));

    pump.stop();
    assert!(!pump.is_running(), "Audio pump should not be running after stop");
}

#[cfg(windows)]
mod windows_loopback_mode_b_tests {
    use super::*;
    use open_boom_core::types::{AudioCommand, GRAPHIC_EQ_BANDS};
    use open_boom_wasapi::loopback_capture::activate_process_loopback_client;
    use open_boom_wasapi::virtual_bridge::BridgeMode;
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IAudioCaptureClient, IAudioClient, IAudioRenderClient, IMMDevice,
        IMMDeviceEnumerator, MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT,
        AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM,
        AUDCLNT_STREAMFLAGS_LOOPBACK, AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY, WAVEFORMATEX,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED,
    };

    /// Generates a stereo 32-bit IEEE float test audio buffer (two-note A4 -> E5 chime)
    /// and also writes a valid 16-bit PCM `.wav` file to disk for diagnostic verification.
    fn generate_test_audio_wav_and_samples(
        sample_rate: u32,
        duration_ms: u32,
        amplitude: f32,
    ) -> (std::path::PathBuf, Vec<f32>) {
        let total_frames = ((sample_rate as u64 * duration_ms as u64) / 1000) as usize;
        let mut stereo_samples = Vec::with_capacity(total_frames * 2);

        for i in 0..total_frames {
            let t = i as f32 / sample_rate as f32;
            let freq = if i < total_frames / 2 { 440.0 } else { 659.25 };
            // Smooth Hann fade-in / fade-out envelope so there are no clicks
            let progress = i as f32 / total_frames.max(1) as f32;
            let env = (std::f32::consts::PI * progress).sin();
            let s = (2.0 * std::f32::consts::PI * freq * t).sin() * amplitude * env;
            stereo_samples.push(s);
            stereo_samples.push(s);
        }

        // Write standard 44-byte RIFF WAVE header + 16-bit PCM payload to temp dir
        let wav_path = std::env::temp_dir().join("open_boom_mode_b_test_tone.wav");
        let num_channels = 2u16;
        let bits_per_sample = 16u16;
        let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample / 8);
        let block_align = num_channels * (bits_per_sample / 8);
        let data_bytes = (stereo_samples.len() * 2) as u32;
        let mut wav_bytes = Vec::with_capacity(44 + data_bytes as usize);

        wav_bytes.extend_from_slice(b"RIFF");
        wav_bytes.extend_from_slice(&(36 + data_bytes).to_le_bytes());
        wav_bytes.extend_from_slice(b"WAVEfmt ");
        wav_bytes.extend_from_slice(&16u32.to_le_bytes());
        wav_bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav_bytes.extend_from_slice(&num_channels.to_le_bytes());
        wav_bytes.extend_from_slice(&sample_rate.to_le_bytes());
        wav_bytes.extend_from_slice(&byte_rate.to_le_bytes());
        wav_bytes.extend_from_slice(&block_align.to_le_bytes());
        wav_bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        wav_bytes.extend_from_slice(b"data");
        wav_bytes.extend_from_slice(&data_bytes.to_le_bytes());

        for &sample in &stereo_samples {
            let pcm16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            wav_bytes.extend_from_slice(&pcm16.to_le_bytes());
        }
        let _ = std::fs::write(&wav_path, &wav_bytes);

        (wav_path, stereo_samples)
    }

    /// Computes narrowband quadrature (DFT) amplitude at `freq_hz` on the Left channel
    /// of an interleaved stereo buffer, immune to broadband background music.
    fn measure_pilot_frequency_amplitude(
        stereo_samples: &[f32],
        sample_rate: u32,
        freq_hz: f32,
    ) -> f32 {
        let frames = stereo_samples.len() / 2;
        if frames == 0 {
            return 0.0;
        }
        let mut re = 0.0f64;
        let mut im = 0.0f64;
        let omega = 2.0 * std::f64::consts::PI * (freq_hz as f64) / (sample_rate as f64);
        for i in 0..frames {
            let x = stereo_samples[i * 2] as f64;
            let phase = omega * (i as f64);
            re += x * phase.cos();
            im += x * phase.sin();
        }
        ((2.0 / frames as f64) * (re * re + im * im).sqrt()) as f32
    }

    #[test]
    fn test_mode_b_self_exclusion_prevents_bell_feedback_oscillation() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator =
                match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                    Ok(e) => e,
                    Err(_) => return, // Skip if no COM audio subsystem in CI container
                };
            let default_render: IMMDevice =
                match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
                    Ok(d) => d,
                    Err(_) => return, // Skip if no active audio output endpoint
                };

            // 1. Verify AudioStreamPump with force_loopback=true switches coordinator bridge to Mode B
            let mut coord_inner = AudioEngineCoordinator::new(
                LoopbackConfig::default(),
                RenderConfig {
                    allow_synthetic_fallback: true,
                    ..Default::default()
                },
            );
            coord_inner.processor_mut().set_master_power(true);
            coord_inner
                .processor_mut()
                .handle_command(AudioCommand::SetMasterVolume {
                    volume: 1.0,
                    boost: 1.4,
                });
            coord_inner
                .processor_mut()
                .handle_command(AudioCommand::SetEqBypass { bypassed: false });
            coord_inner
                .processor_mut()
                .handle_command(AudioCommand::SetEqGains {
                    gains_db: [9.0; GRAPHIC_EQ_BANDS],
                });
            coord_inner
                .processor_mut()
                .handle_command(AudioCommand::SetSurround3D {
                    enabled: true,
                    intensity: 0.85,
                });

            let coordinator = Arc::new(Mutex::new(coord_inner));
            let mut pump = AudioStreamPump::start(
                coordinator.clone(),
                StreamPumpConfig {
                    target_device_id: None,
                    capture_device_id: None,
                    allow_synthetic: false,
                    buffer_guard_enabled: true,
                    force_loopback: true, // Force Mode B
                },
            )
            .expect("start Mode B pump");

            std::thread::sleep(Duration::from_millis(80));
            let bridge_mode = coordinator.lock().unwrap().bridge().active_mode();
            pump.stop();

            assert_eq!(
                bridge_mode,
                BridgeMode::LoopbackFallbackModeB,
                "Coordinator bridge must report LoopbackFallbackModeB when force_loopback is enabled"
            );

            // 2. Verify Process Loopback self-exclusion (`GetCurrentProcessId()`) rejects our own
            // rendered pilot tone (>30 dB rejection vs unexcluded capture), preventing feedback bell loops.
            let render_client: IAudioClient = default_render.Activate(CLSCTX_ALL, None).unwrap();
            let fmt_ptr = render_client.GetMixFormat().unwrap();
            let sample_rate = (*fmt_ptr).nSamplesPerSec;
            let channels = (*fmt_ptr).nChannels.max(1) as usize;

            let current_pid = windows::Win32::System::Threading::GetCurrentProcessId();
            let self_excluded_client = activate_process_loopback_client(current_pid)
                .expect("activate self-excluded process loopback client");
            let unexcluded_client = activate_process_loopback_client(4)
                .expect("activate unexcluded (PID 4) process loopback client");

            let loopback_fmt = WAVEFORMATEX {
                wFormatTag: 3,
                nChannels: 2,
                nSamplesPerSec: sample_rate,
                nAvgBytesPerSec: sample_rate * 8,
                nBlockAlign: 8,
                wBitsPerSample: 32,
                cbSize: 0,
            };
            let flags = AUDCLNT_STREAMFLAGS_LOOPBACK
                | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
                | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY;

            self_excluded_client
                .Initialize(AUDCLNT_SHAREMODE_SHARED, flags, 200_000, 0, &loopback_fmt, None)
                .unwrap();
            unexcluded_client
                .Initialize(AUDCLNT_SHAREMODE_SHARED, flags, 200_000, 0, &loopback_fmt, None)
                .unwrap();

            let self_excluded_svc: IAudioCaptureClient = self_excluded_client.GetService().unwrap();
            let unexcluded_svc: IAudioCaptureClient = unexcluded_client.GetService().unwrap();

            render_client
                .Initialize(AUDCLNT_SHAREMODE_SHARED, 0, 500_000, 0, fmt_ptr, None)
                .unwrap();
            let render_svc: IAudioRenderClient = render_client.GetService().unwrap();
            let buf_size = render_client.GetBufferSize().unwrap_or(480);

            // Generate a 220ms narrowband pilot tone at 14,321 Hz (amplitude 0.04)
            let pilot_freq = 14_321.0f32;
            let total_frames = ((sample_rate as u64 * 220) / 1000) as usize;
            let mut pilot_samples = Vec::with_capacity(total_frames * 2);
            for i in 0..total_frames {
                let t = i as f32 / sample_rate as f32;
                let s = (2.0 * std::f32::consts::PI * pilot_freq * t).sin() * 0.04;
                pilot_samples.push(s);
                pilot_samples.push(s);
            }

            self_excluded_client.Start().unwrap();
            unexcluded_client.Start().unwrap();
            render_client.Start().unwrap();

            let mut frame_cursor = 0usize;
            let mut excluded_captured = Vec::new();
            let mut unexcluded_captured = Vec::new();

            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_millis(240) {
                if frame_cursor < total_frames {
                    let padding = render_client.GetCurrentPadding().unwrap_or(0);
                    let avail = buf_size.saturating_sub(padding) as usize;
                    let to_write = avail.min(total_frames - frame_cursor);
                    if to_write > 0 {
                        if let Ok(ptr) = render_svc.GetBuffer(to_write as u32) {
                            let out = std::slice::from_raw_parts_mut(
                                ptr as *mut f32,
                                to_write * channels,
                            );
                            for f in 0..to_write {
                                let l = pilot_samples[(frame_cursor + f) * 2];
                                let r = pilot_samples[(frame_cursor + f) * 2 + 1];
                                out[f * channels] = l;
                                if channels > 1 {
                                    out[f * channels + 1] = r;
                                }
                                for c in 2..channels {
                                    out[f * channels + c] = 0.0;
                                }
                            }
                            let _ = render_svc.ReleaseBuffer(to_write as u32, 0);
                            frame_cursor += to_write;
                        }
                    }
                }

                for (svc, dest) in [
                    (&self_excluded_svc, &mut excluded_captured),
                    (&unexcluded_svc, &mut unexcluded_captured),
                ] {
                    while svc.GetNextPacketSize().unwrap_or(0) > 0 {
                        let mut p_data: *mut u8 = std::ptr::null_mut();
                        let mut frames_read = 0u32;
                        let mut pkt_flags = 0u32;
                        if svc
                            .GetBuffer(&mut p_data, &mut frames_read, &mut pkt_flags, None, None)
                            .is_ok()
                        {
                            let n = frames_read as usize * 2;
                            if (pkt_flags & (AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)) != 0
                                || p_data.is_null()
                            {
                                dest.extend(std::iter::repeat(0.0f32).take(n));
                            } else {
                                let s = std::slice::from_raw_parts(p_data as *const f32, n);
                                dest.extend_from_slice(s);
                            }
                            let _ = svc.ReleaseBuffer(frames_read);
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(5));
            }

            let _ = render_client.Stop();
            let _ = render_client.Reset();
            let _ = self_excluded_client.Stop();
            let _ = unexcluded_client.Stop();
            CoTaskMemFree(Some(fmt_ptr as _));

            let unexcluded_pilot =
                measure_pilot_frequency_amplitude(&unexcluded_captured, sample_rate, pilot_freq);
            let excluded_pilot =
                measure_pilot_frequency_amplitude(&excluded_captured, sample_rate, pilot_freq);

            assert!(
                unexcluded_pilot > 0.005,
                "Unexcluded loopback should capture the rendered pilot tone (got {:.6})",
                unexcluded_pilot
            );
            assert!(
                excluded_pilot < unexcluded_pilot * 0.02,
                "Self-excluded Mode B loopback must reject our own process's rendered audio (excluded={:.6}, unexcluded={:.6})",
                excluded_pilot,
                unexcluded_pilot
            );
        }
    }

    #[test]
    fn test_mode_b_captures_generated_test_audio_and_decays_to_silence() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator = match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                Ok(e) => e,
                Err(_) => return,
            };
            let default_render: IMMDevice = match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
                Ok(d) => d,
                Err(_) => return,
            };

            let render_client: IAudioClient = default_render.Activate(CLSCTX_ALL, None).unwrap();
            let fmt_ptr = render_client.GetMixFormat().unwrap();
            let sample_rate = (*fmt_ptr).nSamplesPerSec;
            let channels = (*fmt_ptr).nChannels.max(1) as usize;

            // Activate Process Loopback excluding PID 4 (System), so audio from our test tone
            // generator IS captured as external audio, proving the capture + DSP + decay pipeline works end-to-end.
            let capture_client = activate_process_loopback_client(4)
                .expect("activate_process_loopback_client(4) should succeed");
            let loopback_fmt = WAVEFORMATEX {
                wFormatTag: 3,
                nChannels: 2,
                nSamplesPerSec: sample_rate,
                nAvgBytesPerSec: sample_rate * 2 * 4,
                nBlockAlign: 8,
                wBitsPerSample: 32,
                cbSize: 0,
            };
            capture_client
                .Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    AUDCLNT_STREAMFLAGS_LOOPBACK
                        | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
                        | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                    200_000,
                    0,
                    &loopback_fmt,
                    None,
                )
                .expect("Initialize process loopback capture client");
            let capture_svc: IAudioCaptureClient = capture_client.GetService().unwrap();
            capture_client.Start().unwrap();

            render_client
                .Initialize(AUDCLNT_SHAREMODE_SHARED, 0, 500_000, 0, fmt_ptr, None)
                .expect("Initialize test tone render client");
            let render_svc: IAudioRenderClient = render_client.GetService().unwrap();
            let buf_size = render_client.GetBufferSize().unwrap_or(480);

            let (wav_path, test_samples) =
                generate_test_audio_wav_and_samples(sample_rate, 200, 0.05);
            assert!(wav_path.exists(), "Generated test WAV file must exist");

            let mut coord = AudioEngineCoordinator::new(
                LoopbackConfig::default(),
                RenderConfig {
                    allow_synthetic_fallback: true,
                    ..Default::default()
                },
            );
            coord.processor_mut().set_sample_rate(sample_rate);
            coord.processor_mut().set_master_power(true);
            coord
                .processor_mut()
                .handle_command(AudioCommand::SetEqBypass { bypassed: false });
            coord
                .processor_mut()
                .handle_command(AudioCommand::SetEqGains {
                    gains_db: [6.0; GRAPHIC_EQ_BANDS],
                });

            let _ = render_client.Start();
            let total_frames = test_samples.len() / 2;
            let mut frame_cursor = 0usize;
            let mut max_active_peak_dbfs = -100.0f32;
            let mut input_scratch = vec![0.0f32; 8192];
            let mut output_scratch = vec![0.0f32; 8192];

            // 1. Play generated 200ms test tone and capture + process it via Process Loopback
            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_millis(260) {
                if frame_cursor < total_frames {
                    let padding = render_client.GetCurrentPadding().unwrap_or(0);
                    let avail = buf_size.saturating_sub(padding) as usize;
                    let to_write = avail.min(total_frames - frame_cursor);
                    if to_write > 0 {
                        if let Ok(ptr) = render_svc.GetBuffer(to_write as u32) {
                            let out = std::slice::from_raw_parts_mut(
                                ptr as *mut f32,
                                to_write * channels,
                            );
                            for f in 0..to_write {
                                let l = test_samples[(frame_cursor + f) * 2];
                                let r = test_samples[(frame_cursor + f) * 2 + 1];
                                out[f * channels] = l;
                                if channels > 1 {
                                    out[f * channels + 1] = r;
                                }
                                for c in 2..channels {
                                    out[f * channels + c] = 0.0;
                                }
                            }
                            let _ = render_svc.ReleaseBuffer(to_write as u32, 0);
                            frame_cursor += to_write;
                        }
                    }
                }

                while capture_svc.GetNextPacketSize().unwrap_or(0) > 0 {
                    let mut p_data: *mut u8 = std::ptr::null_mut();
                    let mut frames_read = 0u32;
                    let mut flags = 0u32;
                    if capture_svc
                        .GetBuffer(&mut p_data, &mut frames_read, &mut flags, None, None)
                        .is_ok()
                    {
                        let n = (frames_read as usize * 2).min(input_scratch.len());
                        if n > 0 && !p_data.is_null() {
                            let is_silent = (flags & (AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)) != 0;
                            if is_silent {
                                input_scratch[..n].fill(0.0);
                            } else {
                                let slice =
                                    std::slice::from_raw_parts(p_data as *const f32, n);
                                input_scratch[..n].copy_from_slice(slice);
                            }
                            coord.process_audio_chunk(
                                &input_scratch[..n],
                                &mut output_scratch[..n],
                            );
                            let peak = coord.telemetry_snapshot().peak_output_l_dbfs;
                            if peak > max_active_peak_dbfs {
                                max_active_peak_dbfs = peak;
                            }
                        }
                        let _ = capture_svc.ReleaseBuffer(frames_read);
                    }
                }
                std::thread::sleep(Duration::from_millis(5));
            }

            // 2. Stop the test audio stream and simulate the stream_pump idle flush
            let _ = render_client.Stop();
            let _ = render_client.Reset();
            let _ = capture_client.Stop();
            CoTaskMemFree(Some(fmt_ptr as _));
            let _ = std::fs::remove_file(&wav_path);

            input_scratch[..960].fill(0.0);
            for tick in 1..=8 {
                if tick >= 6 {
                    coord
                        .processor_mut()
                        .handle_command(AudioCommand::ResetFilters);
                }
                coord.process_audio_chunk(&input_scratch[..960], &mut output_scratch[..960]);
            }
            let final_peak_dbfs = coord.telemetry_snapshot().peak_output_l_dbfs;

            assert!(
                max_active_peak_dbfs > -45.0,
                "Mode B process loopback must capture generated external test audio (max_active_peak_dbfs = {:.1} dBFS)",
                max_active_peak_dbfs
            );
            assert!(
                final_peak_dbfs <= -90.0,
                "After external test audio stops, DSP and telemetry must cleanly decay to silence <= -90 dBFS (got {:.1} dBFS)",
                final_peak_dbfs
            );
        }
    }
}

