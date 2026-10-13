//! Tests for VirtualAudioBridge, endpoint detection heuristics,
//! and automatic fallback to driverless WASAPI loopback (Story 1.8, AC-1.8.1, AC-1.8.2, AC-1.8.4).

use std::time::Instant;
use open_boom_wasapi::{
    AudioDeviceManager, AudioEngineCoordinator, BridgeMode, LoopbackConfig, RenderConfig,
    VirtualAudioBridge, VirtualBridgeConfig,
};

#[test]
fn test_virtual_device_name_heuristics() {
    // Known virtual device name patterns (AC-1.8.1)
    let virtual_names = [
        "Boom Audio",
        "Boom 3D Virtual Audio Device",
        "boomvad",
        "VB-Audio Point",
        "VB-Audio Virtual Cable",
        "CABLE Input (VB-Audio Virtual Cable)",
        "VoiceMeeter Input (VB-Audio VoiceMeeter VAIO)",
        "VoiceMeeter Aux Input",
        "Virtual Audio Cable",
        "Line 1 (Virtual Audio Cable)",
        "VAC Virtual Sink",
        "OpenBoom Virtual Audio Sink",
        "OpenBoom Virtual Bridge",
    ];

    for name in &virtual_names {
        assert!(
            AudioDeviceManager::is_virtual_device_name(name),
            "Failed to identify '{}' as a virtual audio device",
            name
        );
    }

    // Physical and standard endpoints that MUST NOT be identified as virtual
    let physical_names = [
        "Realtek High Definition Audio",
        "Speakers (Realtek(R) Audio)",
        "Headphones (High Definition Audio Device)",
        "Speakers (Synaptics HD Audio)",
        "NVIDIA High Definition Audio",
        "Default Audio Render Endpoint",
        "Intel(R) Display Audio",
        "USB Audio DAC",
    ];

    for name in &physical_names {
        assert!(
            !AudioDeviceManager::is_virtual_device_name(name),
            "Incorrectly identified '{}' as a virtual audio device",
            name
        );
    }
}

#[test]
fn test_virtual_bridge_mode_a_detection_when_device_present() {
    // Configure device manager with a synthetic virtual device present
    let device_manager = AudioDeviceManager::with_forced_synthetic_and_virtual(true, true);
    let mut bridge = VirtualAudioBridge::new(VirtualBridgeConfig::default());

    let detected_mode = bridge.detect(&device_manager).expect("detect must succeed");
    assert_eq!(
        detected_mode,
        BridgeMode::VirtualSinkModeA,
        "Bridge must enter Mode A when virtual device is detected"
    );
    assert!(bridge.is_using_virtual_bridge());
    assert!(bridge.virtual_device().is_some());
    assert_eq!(
        bridge.virtual_device().unwrap().name,
        AudioDeviceManager::ATTRIBUTED_VIRTUAL_NAME
    );
}

#[test]
fn test_rename_virtual_endpoint_friendly_name() {
    let device_manager = AudioDeviceManager::with_forced_synthetic_and_virtual(true, true);
    let renamed = device_manager
        .rename_virtual_endpoint(None)
        .expect("rename virtual endpoint must succeed");
    assert_eq!(renamed, AudioDeviceManager::ATTRIBUTED_VIRTUAL_NAME);

    let custom = device_manager
        .rename_virtual_endpoint(Some("Custom OpenBoom Virtual Endpoint (powered by VB-Cable)"))
        .expect("rename with custom string must succeed");
    assert_eq!(custom, "Custom OpenBoom Virtual Endpoint (powered by VB-Cable)");
}

#[test]
fn test_virtual_bridge_mode_b_fallback_when_device_absent() {
    // Configure device manager with standard devices only (no virtual device)
    let device_manager = AudioDeviceManager::with_forced_synthetic_and_virtual(true, false);
    let mut bridge = VirtualAudioBridge::new(VirtualBridgeConfig::default());

    let detected_mode = bridge.detect(&device_manager).expect("detect must succeed");
    assert_eq!(
        detected_mode,
        BridgeMode::LoopbackFallbackModeB,
        "Bridge must automatically fall back to Mode B when no virtual device is present"
    );
    assert!(!bridge.is_using_virtual_bridge());
    assert!(bridge.virtual_device().is_none());
}

#[test]
fn test_runtime_virtual_device_invalidation_under_500ms() {
    // Simulate runtime AUDCLNT_E_DEVICE_INVALIDATED
    let mut bridge = VirtualAudioBridge::new(VirtualBridgeConfig {
        force_mode: Some(BridgeMode::VirtualSinkModeA),
        ..Default::default()
    });
    assert_eq!(bridge.active_mode(), BridgeMode::VirtualSinkModeA);
    assert!(bridge.is_using_virtual_bridge());

    let start = Instant::now();
    let fallback_mode = bridge.handle_device_invalidated();
    let elapsed = start.elapsed();

    assert_eq!(
        fallback_mode,
        BridgeMode::LoopbackFallbackModeB,
        "Bridge must transition to Mode B upon device invalidation"
    );
    assert!(!bridge.is_using_virtual_bridge());
    assert!(bridge.virtual_device().is_none());
    assert!(
        elapsed.as_millis() < 500,
        "Fallback transition must execute in < 500ms (took {:?})",
        elapsed
    );
}

#[test]
fn test_coordinator_telemetry_reflects_virtual_bridge_mode() {
    // 1. Coordinator with synthetic devices without virtual adapter (Mode B)
    let render_config_mode_b = RenderConfig {
        force_synthetic: true,
        ..Default::default()
    };
    let coordinator_mode_b = AudioEngineCoordinator::new(
        LoopbackConfig::default(),
        render_config_mode_b,
    );
    let telemetry_b = coordinator_mode_b.telemetry_snapshot();
    assert!(
        !telemetry_b.using_virtual_bridge,
        "Telemetry must report using_virtual_bridge = false in Mode B fallback"
    );
    assert_eq!(coordinator_mode_b.bridge_mode(), BridgeMode::LoopbackFallbackModeB);

    // 2. Coordinator with explicit virtual bridge config (Mode A)
    let bridge_config_mode_a = VirtualBridgeConfig {
        force_mode: Some(BridgeMode::VirtualSinkModeA),
        ..Default::default()
    };
    let coordinator_mode_a = AudioEngineCoordinator::with_bridge_config(
        LoopbackConfig::default(),
        RenderConfig {
            force_synthetic: true,
            ..Default::default()
        },
        bridge_config_mode_a,
        None,
    );
    let telemetry_a = coordinator_mode_a.telemetry_snapshot();
    assert!(
        telemetry_a.using_virtual_bridge,
        "Telemetry must report using_virtual_bridge = true in Mode A"
    );
    assert_eq!(coordinator_mode_a.bridge_mode(), BridgeMode::VirtualSinkModeA);
}

#[test]
fn test_coordinator_runtime_device_lost_fallback() {
    let bridge_config = VirtualBridgeConfig {
        force_mode: Some(BridgeMode::VirtualSinkModeA),
        ..Default::default()
    };
    let mut coordinator = AudioEngineCoordinator::with_bridge_config(
        LoopbackConfig::default(),
        RenderConfig {
            force_synthetic: true,
            ..Default::default()
        },
        bridge_config,
        None,
    );

    assert!(coordinator.is_using_virtual_bridge());

    let start = Instant::now();
    let new_mode = coordinator
        .handle_virtual_device_lost()
        .expect("handle virtual device lost must succeed");
    let elapsed = start.elapsed();

    assert_eq!(new_mode, BridgeMode::LoopbackFallbackModeB);
    assert!(!coordinator.is_using_virtual_bridge());
    assert!(!coordinator.telemetry_snapshot().using_virtual_bridge);
    assert!(
        elapsed.as_millis() < 500,
        "Coordinator fallback recovery must execute in < 500ms"
    );
}

#[test]
fn test_dynamic_set_force_mode_switching() {
    let mut coordinator = AudioEngineCoordinator::new(
        LoopbackConfig::default(),
        RenderConfig {
            force_synthetic: true,
            ..Default::default()
        },
    );

    // Force Mode A (Virtual Sink)
    let mode_a = coordinator
        .set_bridge_force_mode(Some(BridgeMode::VirtualSinkModeA))
        .expect("force Mode A");
    assert_eq!(mode_a, BridgeMode::VirtualSinkModeA);
    assert_eq!(coordinator.bridge_force_mode(), Some(BridgeMode::VirtualSinkModeA));
    assert!(coordinator.is_using_virtual_bridge());
    assert!(coordinator.bridge().virtual_device().is_some());
    assert!(coordinator.telemetry_snapshot().using_virtual_bridge);

    // Force Mode B (Loopback Fallback without driver)
    let mode_b = coordinator
        .set_bridge_force_mode(Some(BridgeMode::LoopbackFallbackModeB))
        .expect("force Mode B");
    assert_eq!(mode_b, BridgeMode::LoopbackFallbackModeB);
    assert_eq!(coordinator.bridge_force_mode(), Some(BridgeMode::LoopbackFallbackModeB));
    assert!(!coordinator.is_using_virtual_bridge());
    assert!(coordinator.bridge().virtual_device().is_none());
    assert!(!coordinator.telemetry_snapshot().using_virtual_bridge);

    // Return to Auto (None)
    let mode_auto = coordinator
        .set_bridge_force_mode(None)
        .expect("restore Auto mode");
    assert_eq!(mode_auto, BridgeMode::LoopbackFallbackModeB);
    assert_eq!(coordinator.bridge().force_mode(), None);
    assert!(coordinator.bridge().virtual_device().is_none());

    // Verify Mode B overrides a present virtual device, and Mode A without synthetic fallback falls back safely when driver is absent
    let mgr_with_virtual = AudioDeviceManager::with_forced_synthetic_and_virtual(true, true);
    let mut bridge = VirtualAudioBridge::new(VirtualBridgeConfig {
        allow_synthetic_fallback: false,
        ..Default::default()
    });
    assert_eq!(bridge.detect(&mgr_with_virtual).unwrap(), BridgeMode::VirtualSinkModeA);
    assert!(bridge.virtual_device().is_some());
    assert_eq!(
        bridge.set_force_mode(Some(BridgeMode::LoopbackFallbackModeB), &mgr_with_virtual).unwrap(),
        BridgeMode::LoopbackFallbackModeB
    );
    assert!(bridge.virtual_device().is_none());

    let mgr_without_virtual = AudioDeviceManager::with_forced_synthetic_and_virtual(true, false);
    assert_eq!(
        bridge.set_force_mode(Some(BridgeMode::VirtualSinkModeA), &mgr_without_virtual).unwrap(),
        BridgeMode::LoopbackFallbackModeB
    );
    assert!(bridge.virtual_device().is_none());
}

