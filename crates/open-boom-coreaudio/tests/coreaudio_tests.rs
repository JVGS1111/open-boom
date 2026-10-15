//! Integration tests for the `open-boom-coreaudio` macOS adapter.
//!
//! Everything here runs headless through the deterministic [`MockHal`] boundary:
//! the capture/render IOProcs are driven frame-by-frame, so the tests exercise the
//! real HAL data plane rather than a synthesized shortcut. A macOS-gated test also
//! drives the production CoreAudio IOProcs over synthetic buffer lists.

use std::sync::Arc;
use std::time::{Duration, Instant};

use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::traits::AudioHostBackend;
use open_boom_core::types::{AudioCommand, AudioDeviceInfo, EngineError, EngineResult};
use open_boom_coreaudio::hal::{AudioHal, MOCK_DEFAULT_ID, MOCK_SPEAKERS_ID, MOCK_VIRTUAL_ID};
use open_boom_coreaudio::{
    set_realtime_priority, CaptureConfig, CoreAudioCapture, CoreAudioCoordinator,
    CoreAudioDeviceManager, CoreAudioRender, HalStream, HalStreamConfig, MockHal, RenderConfig,
    RealtimeGuard, SampleRing, StreamPumpConfig, DEFAULT_AUDIO_PRIORITY, DEFAULT_COMMAND_QUEUE_CAPACITY,
    DEFAULT_STEREO_CHANNELS,
};

fn coordinator_with_mock(hal: Arc<MockHal>) -> CoreAudioCoordinator {
    CoreAudioCoordinator::with_hal(hal, CaptureConfig::default(), RenderConfig::default())
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_coordinator_lifecycle_start_stop() {
    let hal = Arc::new(MockHal::new());
    let mut coordinator = coordinator_with_mock(hal.clone());

    assert!(!coordinator.is_running());
    coordinator.start().expect("start");
    assert!(coordinator.is_running());
    assert!(coordinator.capture().is_capturing());
    assert!(coordinator.render().is_rendering());
    assert!(coordinator.capture().ring().available() == 0);

    coordinator.stop().expect("stop");
    assert!(!coordinator.is_running());
    assert!(!coordinator.capture().is_capturing());
    assert!(!coordinator.render().is_rendering());
}

#[test]
fn test_audibackend_trait_object_lifecycle() {
    let hal = Arc::new(MockHal::new());
    let mut backend: Box<dyn AudioHostBackend> = Box::new(coordinator_with_mock(hal));

    backend.start().expect("trait start");
    assert!(backend.is_running());
    assert!(!backend.enumerate_devices().expect("enumerate").is_empty());
    backend.stop().expect("trait stop");
    assert!(!backend.is_running());
}

// ---------------------------------------------------------------------------
// DSP boundary
// ---------------------------------------------------------------------------

#[test]
fn test_bit_perfect_passthrough_when_master_power_off() {
    let hal = Arc::new(MockHal::new());
    let mut coordinator = coordinator_with_mock(hal);

    coordinator.processor_mut().set_master_power(false);

    let input: Vec<f32> = (0..512).map(|i| ((i as f32) * 0.01).sin() * 0.8).collect();
    let mut output = vec![0.0f32; input.len()];
    coordinator.process_chunk(&input, &mut output);

    assert_eq!(
        output, input,
        "master power OFF must produce a bit-perfect slice copy with zero DSP"
    );
}

#[test]
fn test_limiter_brickwall_ceiling_under_extreme_boost() {
    let hal = Arc::new(MockHal::new());
    let mut coordinator = coordinator_with_mock(hal);

    coordinator.processor_mut().handle_command(AudioCommand::SetMasterVolume {
        volume: 1.0,
        boost: 1.5,
    });

    let input: Vec<f32> = (0..1024).map(|i| ((i as f32) * 0.05).sin()).collect();
    let mut output = vec![0.0f32; input.len()];
    coordinator.process_chunk(&input, &mut output);

    let ceiling = 10.0f32.powf(-0.2 / 20.0); // -0.2 dBFS
    for (index, sample) in output.iter().enumerate() {
        assert!(
            sample.abs() <= ceiling + 1e-5,
            "sample {index} ({sample}) exceeds the -0.2 dBFS limiter ceiling"
        );
    }
    assert!(coordinator.processor().telemetry_snapshot().limiter_active);
}

// ---------------------------------------------------------------------------
// Latency
// ---------------------------------------------------------------------------

#[test]
fn test_round_trip_latency_default_config_under_15ms() {
    let hal = Arc::new(MockHal::new());
    let coordinator = coordinator_with_mock(hal);

    let latency = coordinator.round_trip_latency_ms();
    assert!(
        latency < 15.0,
        "documented default configuration must stay below 15 ms (got {latency} ms)"
    );
    // 240 frames capture + 240 frames render + 96-sample limiter lookahead = 12 ms.
    assert!((latency - 12.0).abs() < 0.05, "expected ~12 ms, got {latency} ms");
}

#[test]
fn test_round_trip_latency_reflects_configured_buffers() {
    let hal = Arc::new(MockHal::new());
    let capture = CaptureConfig {
        buffer_frames: 1024,
        ..CaptureConfig::default()
    };
    let render = RenderConfig {
        buffer_frames: 1024,
        ..RenderConfig::default()
    };
    let coordinator = CoreAudioCoordinator::with_hal(hal, capture, render);

    let latency = coordinator.round_trip_latency_ms();
    assert!(
        latency > 15.0,
        "larger configured buffers must raise the reported latency, got {latency} ms"
    );
    let expected = (1024.0 / 48_000.0) * 1000.0 * 2.0 + (96.0 / 48_000.0) * 1000.0;
    assert!((latency - expected).abs() < 0.05);
}

#[test]
fn test_capture_render_sample_rate_mismatch_is_rejected() {
    let hal = Arc::new(MockHal::new());
    let capture = CaptureConfig {
        sample_rate: 48_000,
        ..CaptureConfig::default()
    };
    let render = RenderConfig {
        sample_rate: 44_100,
        ..RenderConfig::default()
    };
    let mut coordinator = CoreAudioCoordinator::with_hal(hal, capture, render);

    assert!(
        coordinator.start().is_err(),
        "capture/render sample-rate disagreement must be rejected"
    );
    assert!(!coordinator.is_running());
}

#[test]
fn test_synthetic_capture_config_selects_synthetic_hal() {
    // A caller requesting synthetic capture must not silently get the real HAL/dsp path.
    let coordinator = CoreAudioCoordinator::with_configs(
        CaptureConfig {
            force_synthetic: true,
            ..CaptureConfig::default()
        },
        RenderConfig::default(),
    );
    assert!(coordinator.device_manager().is_forced_synthetic());
    assert!(coordinator
        .enumerate_devices()
        .expect("enumerate")
        .iter()
        .all(|d| d.id.starts_with("mock_")));
}

// ---------------------------------------------------------------------------
// Device enumeration / switching / loss
// ---------------------------------------------------------------------------

#[test]
fn test_device_switching_updates_os_default_endpoint() {
    let hal = Arc::new(MockHal::new());
    let mut coordinator = coordinator_with_mock(hal.clone());
    coordinator.start().expect("start");

    let devices = coordinator.enumerate_devices().expect("enumerate");
    assert!(devices.iter().any(|d| d.id == MOCK_SPEAKERS_ID));

    coordinator
        .switch_device(Some(MOCK_SPEAKERS_ID.to_string()))
        .expect("switch device");

    assert_eq!(coordinator.current_device_id().as_deref(), Some(MOCK_SPEAKERS_ID));
    assert_eq!(
        hal.default_output_device_id().unwrap().as_deref(),
        Some(MOCK_SPEAKERS_ID),
        "switch_device must update the OS default output endpoint"
    );
    assert!(hal
        .enumerate_devices()
        .unwrap()
        .iter()
        .find(|d| d.id == MOCK_SPEAKERS_ID)
        .unwrap()
        .is_default);

    // Switching to an unknown endpoint must fail loudly instead of silently using device 0.
    assert!(coordinator.switch_device(Some("does-not-exist".into())).is_err());
    // A rejected switch must not leave the coordinator reporting a dead stream.
    assert!(coordinator.is_running());
    assert_eq!(coordinator.current_device_id().as_deref(), Some(MOCK_SPEAKERS_ID));
}

#[test]
fn test_device_loss_restores_default_within_500ms() {
    let hal = Arc::new(MockHal::new());
    let mut coordinator = coordinator_with_mock(hal.clone());
    coordinator.start().expect("start");
    coordinator
        .switch_device(Some(MOCK_SPEAKERS_ID.to_string()))
        .expect("switch device");

    let started = Instant::now();
    coordinator.handle_device_lost().expect("handle device lost");
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_millis(500),
        "device-loss recovery took {elapsed:?}, must be < 500 ms"
    );
    assert_eq!(
        coordinator.current_device_id(),
        None,
        "device loss must detach from the lost endpoint"
    );
    assert!(coordinator.render().is_rendering());
}

#[test]
fn test_poll_device_loss_consumes_the_hal_edge() {
    let hal = Arc::new(MockHal::new());
    let mut coordinator = coordinator_with_mock(hal.clone());
    coordinator.start().expect("start");
    coordinator
        .switch_device(Some(MOCK_SPEAKERS_ID.to_string()))
        .expect("switch device");

    hal.mark_default_output_changed();
    assert!(
        coordinator.poll_device_loss().expect("poll device loss"),
        "the listener edge must trigger a recovery"
    );
    assert_eq!(coordinator.current_device_id(), None);
    assert!(coordinator.render().is_rendering());

    // The edge is consumed, so a second poll is a no-op.
    assert!(!coordinator.poll_device_loss().expect("second poll"));
}

#[test]
fn test_virtual_device_name_heuristic_is_narrow() {
    assert!(CoreAudioDeviceManager::is_virtual_device_name("BlackHole 16ch"));
    assert!(CoreAudioDeviceManager::is_virtual_device_name(
        "CABLE Input (VB-Audio Virtual Cable)"
    ));
    assert!(CoreAudioDeviceManager::is_virtual_device_name(
        "OpenBoom Spatial Audio (powered by BlackHole)"
    ));
    assert!(CoreAudioDeviceManager::is_virtual_device_name(
        "VoiceMeeter Input (VB-Audio VoiceMeeter VAIO)"
    ));
    assert!(CoreAudioDeviceManager::is_virtual_device_name("Boom Audio"));
    assert!(CoreAudioDeviceManager::is_virtual_device_name("Soundflower (2ch)"));

    // Real physical endpoints must not be hidden by overly broad substrings.
    assert!(!CoreAudioDeviceManager::is_virtual_device_name("MacBook Pro Speakers"));
    assert!(!CoreAudioDeviceManager::is_virtual_device_name("External Headphones"));
    assert!(!CoreAudioDeviceManager::is_virtual_device_name("BoomBox USB Audio"));
    assert!(!CoreAudioDeviceManager::is_virtual_device_name("Cable TV Tuner"));
    assert!(!CoreAudioDeviceManager::is_virtual_device_name("UltraStudio 4K Mini"));
}

#[test]
fn test_physical_enumeration_filters_virtual_and_defaults() {
    let manager = CoreAudioDeviceManager::with_forced_synthetic_and_virtual(true, true);
    let all = manager.enumerate_devices().expect("enumerate");
    assert!(all.iter().any(|d| d.id == MOCK_VIRTUAL_ID));

    let physical = manager.enumerate_physical_devices().expect("physical");
    assert!(physical.iter().all(|d| d.id != MOCK_VIRTUAL_ID));
    assert!(physical.iter().any(|d| d.is_default));

    let physical_default = manager
        .get_physical_default_device()
        .expect("physical default")
        .expect("one default");
    assert!(physical.iter().any(|d| d.id == physical_default.id));

    let by_id = manager
        .get_device_by_id(MOCK_SPEAKERS_ID)
        .expect("by id")
        .expect("speakers");
    assert_eq!(by_id.name, "Mock Headphones (Synthetic Audio)");
    assert!(manager.get_device_by_id("missing").expect("by id").is_none());
}

#[test]
fn test_production_enumeration_never_injects_synthetic_fixtures() {
    // A HAL with no hardware must yield an empty list, not synthetic fixtures.
    let manager = CoreAudioDeviceManager::with_hal(Arc::new(EmptyHal));
    assert!(manager
        .enumerate_devices()
        .expect("enumerate")
        .is_empty());

    // Only explicit forced-synthetic managers surface the fixture list.
    let forced = CoreAudioDeviceManager::with_forced_synthetic_and_virtual(true, true);
    assert!(forced
        .enumerate_devices()
        .expect("enumerate")
        .iter()
        .any(|d| d.id == MOCK_VIRTUAL_ID));
}

#[test]
fn test_synthetic_fallback_is_opt_in() {
    let manager = CoreAudioDeviceManager::with_hal_options(Arc::new(EmptyHal), true);
    assert_eq!(
        manager.enumerate_devices().expect("enumerate").len(),
        2,
        "opt-in fallback yields the deterministic mock device list"
    );
}

// ---------------------------------------------------------------------------
// HAL data plane (headless IOProc verification)
// ---------------------------------------------------------------------------

#[test]
fn test_capture_io_proc_frames_reach_read_samples() {
    let hal = Arc::new(MockHal::new());
    let mut coordinator = coordinator_with_mock(hal.clone());
    coordinator.start().expect("start");

    let scripted = [0.1f32, -0.1, 0.2, -0.2, 0.3, -0.3];
    hal.queue_capture_frames(&scripted);

    let frames = coordinator
        .capture_mut()
        .drive_io_proc(3)
        .expect("drive capture io proc");
    assert_eq!(frames, 3);

    let mut out = [0.0f32; 6];
    let read = coordinator.capture_mut().read_samples(&mut out);
    assert_eq!(read, 6);
    assert_eq!(
        out, scripted,
        "read_samples must return exactly the frames delivered by the capture IOProc"
    );
}

#[test]
fn test_capture_without_input_is_silent_not_synthesized() {
    let hal = Arc::new(MockHal::new());
    let mut capture = CoreAudioCapture::new(hal.clone(), CaptureConfig::default());
    capture.start().expect("start");

    capture.drive_io_proc(4).expect("drive capture io proc");
    let mut out = [0.0f32; 8];
    let read = capture.read_samples(&mut out);
    assert_eq!(read, 8);
    assert!(
        out.iter().all(|s| *s == 0.0),
        "a mock capture stream with no queued input must be silent, not a synthesized tone"
    );
}

#[test]
fn test_forced_synthetic_capture_generates_440hz_tone() {
    let hal = Arc::new(MockHal::with_tone(true));
    let mut capture = CoreAudioCapture::new(hal, CaptureConfig::default());
    capture.start().expect("start");

    capture.drive_io_proc(4).expect("drive capture io proc");
    let mut out = [0.0f32; 8];
    let read = capture.read_samples(&mut out);
    assert_eq!(read, 8);

    // Frame 0 is sin(0) = 0; frame 1 is sin(2*pi*440/48000) ~= 0.0576.
    assert!(out[2].abs() > 1e-3, "synthetic path must emit the 440 Hz tone");
    let expected = (2.0 * std::f64::consts::PI * 440.0 / 48_000.0).sin() as f32;
    assert!((out[2] - expected).abs() < 1e-4);
}

#[test]
fn test_render_io_proc_consumes_exactly_the_written_frames() {
    let hal = Arc::new(MockHal::new());
    let mut coordinator = coordinator_with_mock(hal.clone());
    coordinator.start().expect("start");

    let written_samples = [0.5f32, -0.5, 0.25, -0.25];
    let written = coordinator.render_mut().write_samples(&written_samples);
    assert_eq!(written, written_samples.len());

    let consumed = coordinator
        .render_mut()
        .drive_io_proc(2)
        .expect("drive render io proc");
    assert_eq!(consumed, 2);
    assert_eq!(
        hal.rendered_samples(),
        written_samples.to_vec(),
        "the render IOProc must consume exactly the frames written"
    );
}

#[test]
fn test_render_without_writes_consumes_silence() {
    let hal = Arc::new(MockHal::new());
    let mut render = CoreAudioRender::new(hal.clone(), RenderConfig::default());
    render.start().expect("start");

    let consumed = render.drive_io_proc(2).expect("drive render io proc");
    assert_eq!(consumed, 0);
    assert!(hal.rendered_samples().is_empty());
}

#[test]
fn test_restart_clears_stale_ring_samples() {
    let hal = Arc::new(MockHal::new());
    let mut render = CoreAudioRender::new(hal.clone(), RenderConfig::default());
    render.start().expect("start");
    render.write_samples(&[0.9f32, -0.9]);

    // Restarting must drop the queued samples instead of replaying them.
    render.stop().expect("stop");
    render.start().expect("restart");
    assert_eq!(render.ring().available(), 0);

    render.drive_io_proc(1).expect("drive render io proc");
    assert!(hal.rendered_samples().is_empty());
}

#[test]
fn test_full_ring_drops_are_counted() {
    let hal = Arc::new(MockHal::new());
    let capture = CoreAudioCapture::new(hal, CaptureConfig::default());
    let capacity = capture.ring().capacity();
    let oversized = vec![1.0f32; capacity * 2];
    capture.ring().push_frames(&oversized, 2);
    assert!(capture.dropped_samples() > 0, "full-ring drops must be observable");
}

// ---------------------------------------------------------------------------
// Real-time scheduling
// ---------------------------------------------------------------------------

#[test]
fn test_realtime_priority_graceful_fallback() {
    // Unprivileged macOS runs generally deny SCHED_RR; the contract is graceful fallback
    // that reports the errno, never a panic and never a fabricated `active = true`.
    let guard = RealtimeGuard::apply(DEFAULT_AUDIO_PRIORITY);
    assert_eq!(guard.priority(), DEFAULT_AUDIO_PRIORITY);

    match set_realtime_priority(DEFAULT_AUDIO_PRIORITY) {
        Ok(()) => assert!(guard.is_active(), "active must track a successful kernel grant"),
        Err(err) => {
            assert!(!guard.is_active(), "a denied request must not report active");
            let message = format!("{err}");
            assert!(
                message.contains("SCHED_RR"),
                "the failure must identify the RT policy: {message}"
            );
        }
    }
    drop(guard);
}

// ---------------------------------------------------------------------------
// Stream pump
// ---------------------------------------------------------------------------

#[test]
fn test_stream_pump_moves_samples_capture_to_render() {
    let hal = Arc::new(MockHal::new());
    let mut coordinator = coordinator_with_mock(hal.clone());

    // Route a master-power-off command to the pump's processor for bit-perfect passthrough.
    let queue = SpscQueue::<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>::new();
    queue
        .push(AudioCommand::SetMasterPower(false))
        .expect("queue command");
    coordinator.set_command_queue(queue);

    coordinator.start().expect("start");

    let input = [0.1f32, -0.1, 0.2, -0.2, 0.3, -0.3, 0.4, -0.4];
    hal.queue_capture_frames(&input);

    coordinator
        .create_stream_pump(StreamPumpConfig { block_frames: 4 })
        .expect("create stream pump");
    assert!(coordinator.pump_is_running());

    // Publish frames through the capture IOProc, then let the pump move them.
    let produced = coordinator
        .capture_mut()
        .drive_io_proc(4)
        .expect("drive capture io proc");
    assert_eq!(produced, 4);

    let render_ring = coordinator.render().ring();
    let deadline = Instant::now() + Duration::from_secs(2);
    while render_ring.available() < input.len() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        render_ring.available() >= input.len(),
        "the pump must move capture frames onto the render ring"
    );

    let consumed = coordinator
        .render_mut()
        .drive_io_proc(4)
        .expect("drive render io proc");
    assert_eq!(consumed, 4);
    assert_eq!(
        hal.rendered_samples(),
        input.to_vec(),
        "a sample must traverse capture -> DSP -> render unchanged under passthrough"
    );

    coordinator.stop().expect("stop");
    assert!(!coordinator.pump_is_running());
}

#[test]
fn test_only_one_stream_pump_can_be_configured() {
    let hal = Arc::new(MockHal::new());
    let mut coordinator = coordinator_with_mock(hal);

    coordinator
        .create_stream_pump(StreamPumpConfig::default())
        .expect("first pump");
    assert!(coordinator.is_pump_configured());
    assert!(
        coordinator.create_stream_pump(StreamPumpConfig::default()).is_err(),
        "a second pump must be rejected to keep a single SPSC consumer"
    );
}

// ---------------------------------------------------------------------------
// Test doubles
// ---------------------------------------------------------------------------

/// HAL that reports no devices and fails every stream open, simulating a Mac with no
/// usable audio hardware.
struct EmptyHal;

impl AudioHal for EmptyHal {
    fn enumerate_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        Ok(Vec::new())
    }

    fn default_output_device_id(&self) -> EngineResult<Option<String>> {
        Ok(None)
    }

    fn device_name(&self, _device_id: &str) -> EngineResult<Option<String>> {
        Ok(None)
    }

    fn set_default_output_device(&self, _device_id: &str) -> EngineResult<()> {
        Err(EngineError::DeviceInitialization("no audio hardware".to_string()))
    }

    fn open_capture(
        &self,
        _config: HalStreamConfig,
        _ring: Arc<SampleRing>,
    ) -> EngineResult<Box<dyn HalStream>> {
        Err(EngineError::DeviceInitialization("no audio hardware".to_string()))
    }

    fn open_render(
        &self,
        _config: HalStreamConfig,
        _ring: Arc<SampleRing>,
    ) -> EngineResult<Box<dyn HalStream>> {
        Err(EngineError::DeviceInitialization("no audio hardware".to_string()))
    }
}

/// Stream that starts cleanly but always fails to stop, to exercise `stop()` error paths.
struct FailingStopStream;

impl HalStream for FailingStopStream {
    fn start(&mut self) -> EngineResult<()> {
        Ok(())
    }

    fn stop(&mut self) -> EngineResult<()> {
        Err(EngineError::DeviceInitialization("stop failed".to_string()))
    }

    fn is_running(&self) -> bool {
        true
    }

    fn sample_rate(&self) -> u32 {
        48_000
    }

    fn channels(&self) -> u16 {
        2
    }

    fn buffer_frames(&self) -> u32 {
        240
    }

    fn drive_io_proc(&mut self, _frames: usize) -> EngineResult<usize> {
        Ok(0)
    }
}

/// HAL whose streams fail on stop.
struct FailingStopHal;

impl AudioHal for FailingStopHal {
    fn enumerate_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        Ok(Vec::new())
    }

    fn default_output_device_id(&self) -> EngineResult<Option<String>> {
        Ok(None)
    }

    fn device_name(&self, _device_id: &str) -> EngineResult<Option<String>> {
        Ok(None)
    }

    fn set_default_output_device(&self, _device_id: &str) -> EngineResult<()> {
        Ok(())
    }

    fn open_capture(
        &self,
        _config: HalStreamConfig,
        _ring: Arc<SampleRing>,
    ) -> EngineResult<Box<dyn HalStream>> {
        Ok(Box::new(FailingStopStream))
    }

    fn open_render(
        &self,
        _config: HalStreamConfig,
        _ring: Arc<SampleRing>,
    ) -> EngineResult<Box<dyn HalStream>> {
        Ok(Box::new(FailingStopStream))
    }
}

#[test]
fn test_stop_reports_capture_and_render_errors() {
    let hal = Arc::new(FailingStopHal);
    let mut coordinator =
        CoreAudioCoordinator::with_hal(hal, CaptureConfig::default(), RenderConfig::default());
    coordinator.start().expect("start");

    let err = coordinator.stop().expect_err("stop must surface both failures");
    let message = format!("{err}");
    assert!(message.contains("capture stop failed"), "{message}");
    assert!(message.contains("render stop failed"), "{message}");
    assert!(!coordinator.is_running());
}

#[test]
fn test_default_stereo_boundary_is_two_channels() {
    assert_eq!(DEFAULT_STEREO_CHANNELS, 2);
    assert_ne!(MOCK_DEFAULT_ID, MOCK_SPEAKERS_ID);
}

#[cfg(target_os = "macos")]
#[test]
fn test_real_coreaudio_hal_enumerates_without_synthetic_fixtures() {
    // Exercises the real CoreAudio FFI path: listener registration, device enumeration,
    // CFString name read (+CFRelease) and the default-output query. Must never surface
    // synthetic fixtures and must never panic.
    let manager = CoreAudioDeviceManager::new();
    let devices = manager
        .enumerate_devices()
        .expect("real CoreAudio enumeration");
    assert!(
        devices.iter().all(|d| !d.id.starts_with("mock_")),
        "production enumeration must not inject synthetic fixtures"
    );
    for device in &devices {
        assert!(
            !device.name.trim().is_empty(),
            "real device names must never be empty or garbage"
        );
    }
    let _ = manager.get_default_device().expect("default device query");
}

#[cfg(target_os = "macos")]
#[test]
fn test_real_io_procs_move_frames_over_synthetic_buffer_lists() {
    use open_boom_coreaudio::hal::{test_drive_capture_io_proc, test_drive_render_io_proc};

    // Capture (stereo): the real IOProc publishes interleaved Float32 frames into the ring.
    let capture_ring = SampleRing::new(64);
    let captured = [0.1f32, -0.1, 0.2, -0.2, 0.3, -0.3];
    assert_eq!(test_drive_capture_io_proc(2, &captured, capture_ring.clone()), 0);
    let mut out = [0.0f32; 6];
    assert_eq!(capture_ring.pop_samples(&mut out), 6);
    assert_eq!(out, captured, "real capture IOProc must publish the input frames");

    // Capture (mono): each sample is duplicated to a stereo frame.
    let mono_ring = SampleRing::new(64);
    assert_eq!(test_drive_capture_io_proc(1, &[0.4, 0.5], mono_ring.clone()), 0);
    let mut mono_out = [0.0f32; 4];
    assert_eq!(mono_ring.pop_samples(&mut mono_out), 4);
    assert_eq!(mono_out, [0.4, 0.4, 0.5, 0.5]);

    // Capture (>2 channels): only the first two channels are kept.
    let multi_ring = SampleRing::new(64);
    let surround = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
    assert_eq!(test_drive_capture_io_proc(6, &surround, multi_ring.clone()), 0);
    let mut multi_out = [0.0f32; 4];
    assert_eq!(multi_ring.pop_samples(&mut multi_out), 4);
    assert_eq!(multi_out, [1.0, 2.0, 7.0, 8.0]);

    // Render (stereo): the real IOProc fills the output buffer from the ring.
    let render_ring = SampleRing::new(64);
    render_ring.push_samples(&[0.5, -0.5, 0.25, -0.25]);
    let mut rendered = [9.0f32; 4];
    assert_eq!(test_drive_render_io_proc(2, &mut rendered, render_ring.clone()), 0);
    assert_eq!(rendered, [0.5, -0.5, 0.25, -0.25]);

    // Render (mono): L/R are averaged.
    let mono_render_ring = SampleRing::new(64);
    mono_render_ring.push_samples(&[1.0, 0.0, 2.0, 0.0]);
    let mut mono_rendered = [0.0f32; 2];
    assert_eq!(
        test_drive_render_io_proc(1, &mut mono_rendered, mono_render_ring.clone()),
        0
    );
    assert_eq!(mono_rendered, [0.5, 1.0]);

    // Render (>2 channels): stereo goes to the first two channels, the rest are muted.
    let multi_render_ring = SampleRing::new(64);
    multi_render_ring.push_samples(&[0.7, -0.7]);
    let mut multi_rendered = [9.0f32; 4];
    assert_eq!(
        test_drive_render_io_proc(4, &mut multi_rendered, multi_render_ring.clone()),
        0
    );
    assert_eq!(multi_rendered, [0.7, -0.7, 0.0, 0.0]);
}
