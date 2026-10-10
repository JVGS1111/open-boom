//! Unit and integration tests for audio endpoint enumeration, dynamic switching,
//! and device-lost fallback recovery in WASAPI.

use std::time::Instant;
use open_boom_core::types::AudioDeviceInfo;
use open_boom_wasapi::device::AudioDeviceManager;
use open_boom_wasapi::render_client::{RenderClient, RenderConfig};
use open_boom_wasapi::coordinator::AudioEngineCoordinator;
use open_boom_wasapi::loopback_capture::LoopbackConfig;

#[test]
fn test_synthetic_device_enumeration() {
    let manager = AudioDeviceManager::with_forced_synthetic(true);
    let devices = manager.enumerate_devices().expect("enumerate synthetic devices");
    assert_eq!(devices.len(), 2, "Should enumerate exactly two synthetic devices");

    let default_dev = devices.iter().find(|d| d.is_default);
    assert!(default_dev.is_some(), "Should designate a default device");
    assert_eq!(default_dev.unwrap().id, "synthetic_default");
}

#[test]
fn test_device_lookup_by_id() {
    let manager = AudioDeviceManager::with_forced_synthetic(true);
    let dev = manager.get_device_by_id("synthetic_default").expect("lookup device by id");
    assert!(dev.is_some());
    assert_eq!(dev.unwrap().name, "Headphones (Synthetic Audio)");

    let unknown = manager.get_device_by_id("non_existent_id").expect("lookup unknown");
    assert!(unknown.is_none());
}

#[test]
fn test_get_default_device() {
    let manager = AudioDeviceManager::with_forced_synthetic(true);
    let default_dev = manager.get_default_device().expect("get default device");
    assert!(default_dev.is_some());
    assert!(default_dev.unwrap().is_default);
}

#[test]
fn test_hardware_device_enumeration() {
    let manager = AudioDeviceManager::with_synthetic_fallback(true);
    let devices = manager.enumerate_devices().expect("enumerate devices");
    assert!(!devices.is_empty(), "Should return at least one device (hardware or fallback)");
    for dev in &devices {
        assert!(!dev.id.is_empty(), "Device ID must not be empty");
        assert!(!dev.name.is_empty(), "Device Name must not be empty");
    }
}

#[test]
fn test_no_audio_devices_graceful_handling() {
    // When synthetic fallback is enabled, never crashes and returns synthetic devices
    let manager = AudioDeviceManager::with_synthetic_fallback(true);
    let result = manager.enumerate_devices();
    assert!(result.is_ok(), "Synthetic fallback must prevent panics or crashes");
}

#[test]
fn test_render_client_device_switching_under_500ms() {
    let mut config = RenderConfig::default();
    config.allow_synthetic_fallback = true;
    config.force_synthetic = true;

    let mut client = RenderClient::with_config(config);
    client.start().expect("start render client");
    assert!(client.is_rendering());

    let start = Instant::now();
    // Switch to a new device endpoint
    client
        .switch_device(Some("synthetic_speakers".to_string()))
        .expect("switch device");
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 500,
        "Device switch must complete under 500ms (took {}ms)",
        elapsed.as_millis()
    );
    assert!(client.is_rendering());
    assert_eq!(client.device_id(), Some("synthetic_speakers"));

    // Clean stop
    client.stop().expect("stop render client");
}

#[test]
fn test_render_client_handle_device_lost_under_500ms() {
    let mut config = RenderConfig::default();
    config.allow_synthetic_fallback = true;
    config.force_synthetic = true;
    config.device_id = Some("synthetic_speakers".to_string());

    let mut client = RenderClient::with_config(config);
    client.start().expect("start render client");
    assert!(client.is_rendering());
    assert_eq!(client.device_id(), Some("synthetic_speakers"));

    // Simulate device invalidation / unplug
    let start = Instant::now();
    client.handle_device_lost().expect("handle device lost");
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 500,
        "Device lost recovery must complete under 500ms (took {}ms)",
        elapsed.as_millis()
    );
    assert!(client.is_rendering());
    assert_eq!(
        client.device_id(),
        None,
        "Device ID should revert to None (system default)"
    );

    client.stop().expect("stop render client");
}

#[test]
fn test_coordinator_device_lifecycle() {
    let mut render_config = RenderConfig::default();
    render_config.allow_synthetic_fallback = true;
    render_config.force_synthetic = true;

    let mut coord = AudioEngineCoordinator::new(
        LoopbackConfig::default(),
        render_config,
    );

    let devices = coord.enumerate_devices().expect("enumerate devices");
    assert!(!devices.is_empty());

    // Switch device via coordinator
    let switch_res = coord.switch_device(Some("synthetic_speakers".to_string()));
    assert!(switch_res.is_ok());
    assert_eq!(coord.current_device_id(), Some("synthetic_speakers"));

    // Recover from device lost
    let recovery_res = coord.handle_device_lost();
    assert!(recovery_res.is_ok());
    assert_eq!(coord.current_device_id(), None);
}

#[test]
fn test_audio_device_info_contract() {
    let info = AudioDeviceInfo::new("endpoint-1", "Realtek High Definition Audio", true);
    assert_eq!(info.id, "endpoint-1");
    assert_eq!(info.name, "Realtek High Definition Audio");
    assert!(info.is_default);

    let info_clone = info.clone();
    assert_eq!(info, info_clone);
}
