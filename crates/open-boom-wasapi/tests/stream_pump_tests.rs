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
