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

#[test]
fn test_set_default_device_com_lifecycle_no_access_violation() {
    let manager = AudioDeviceManager::new();
    if let Ok(Some(physical)) = manager.get_physical_default_device() {
        // Repeatedly invoke set_default_device on isolated COM worker threads
        // to ensure IUnknown guards are released before CoUninitialize() without 0xC0000005.
        for _ in 0..3 {
            let res = manager.set_default_device(&physical.id);
            assert!(
                res.is_ok(),
                "set_default_device on physical endpoint should succeed: {:?}",
                res
            );
        }
    }
}

#[test]
fn test_extract_endpoint_guid() {
    let wasapi_id = "{0.0.0.00000000}.{5bdbd9fc-3e1a-42ae-b9ab-d171a20a285d}";
    assert_eq!(
        AudioDeviceManager::extract_endpoint_guid(wasapi_id),
        Some("{5bdbd9fc-3e1a-42ae-b9ab-d171a20a285d}".to_string())
    );

    let pure_guid = "{f625f981-599a-4e3d-a57f-514e8bc2fe72}";
    assert_eq!(
        AudioDeviceManager::extract_endpoint_guid(pure_guid),
        Some("{f625f981-599a-4e3d-a57f-514e8bc2fe72}".to_string())
    );

    let invalid = "synthetic_device_without_guid";
    assert_eq!(AudioDeviceManager::extract_endpoint_guid(invalid), None);

    let malformed = "{1234-too-short}";
    assert_eq!(AudioDeviceManager::extract_endpoint_guid(malformed), None);
}

#[test]
fn test_normalize_endpoint_friendly_name() {
    // Legacy brand names must be purged and normalized with attribution
    assert_eq!(
        AudioDeviceManager::normalize_endpoint_friendly_name("Boom Audio"),
        AudioDeviceManager::ATTRIBUTED_VIRTUAL_NAME
    );
    assert_eq!(
        AudioDeviceManager::normalize_endpoint_friendly_name("CABLE Input (VB-Audio Virtual Cable)"),
        AudioDeviceManager::ATTRIBUTED_VIRTUAL_NAME
    );

    // Non-virtual physical devices must be preserved
    assert_eq!(
        AudioDeviceManager::normalize_endpoint_friendly_name("HyperX Cloud II Wireless"),
        "HyperX Cloud II Wireless"
    );
    assert_eq!(
        AudioDeviceManager::normalize_endpoint_friendly_name("Realtek High Definition Audio"),
        "Realtek High Definition Audio"
    );

    // Empty strings get safe generic fallback
    assert_eq!(
        AudioDeviceManager::normalize_endpoint_friendly_name("   "),
        "Audio Device"
    );
}

#[test]
fn test_rename_virtual_endpoint_enforces_attribution() {
    let mgr = AudioDeviceManager::with_forced_synthetic_and_virtual(true, true);

    // None defaults to full attributed name
    let default_renamed = mgr.rename_virtual_endpoint(None).unwrap();
    assert_eq!(default_renamed, AudioDeviceManager::ATTRIBUTED_VIRTUAL_NAME);

    // Custom name with attribution preserved
    let with_attr = mgr
        .rename_virtual_endpoint(Some("My Virtual Cable (powered by VB-Cable)"))
        .unwrap();
    assert_eq!(with_attr, "My Virtual Cable (powered by VB-Cable)");

    // Custom name without attribution gets mandatory suffix appended
    let without_attr = mgr
        .rename_virtual_endpoint(Some("Studio Virtual Sink"))
        .unwrap();
    assert_eq!(without_attr, "Studio Virtual Sink (powered by VB-Cable)");
}

#[test]
fn test_configure_virtual_endpoint_7_1_speaker_mask() {
    assert_eq!(AudioDeviceManager::SPEAKER_MASK_7_1_SURROUND, 0x63F);

    let mgr = AudioDeviceManager::with_forced_synthetic_and_virtual(true, true);

    // Default configuration injects 0x63F (7.1 Surround)
    let mask = mgr.configure_virtual_endpoint_7_1(None).unwrap();
    assert_eq!(mask, 0x63F);

    // Custom mask configuration
    let stereo_mask = 0x3;
    let custom_mask = mgr.configure_virtual_endpoint_7_1(Some(stereo_mask)).unwrap();
    assert_eq!(custom_mask, stereo_mask);

    // Query virtual endpoint speaker mask
    let queried_mask = mgr.get_virtual_endpoint_speaker_mask().unwrap();
    assert_eq!(queried_mask, 0x63F);
}

#[test]
fn test_check_virtual_driver_presence() {
    // When virtual device is included
    let mgr_with = AudioDeviceManager::with_forced_synthetic_and_virtual(true, true);
    let (present, info) = mgr_with.check_virtual_driver_presence().unwrap();
    assert!(present);
    assert!(info.is_some());
    assert!(AudioDeviceManager::is_virtual_device_name(&info.unwrap().name));

    // When virtual device is absent
    let mgr_without = AudioDeviceManager::with_forced_synthetic_and_virtual(true, false);
    let (not_present, none_info) = mgr_without.check_virtual_driver_presence().unwrap();
    assert!(!not_present);
    assert!(none_info.is_none());
}
