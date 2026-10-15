//! Comprehensive unit and integration test suite for `EndpointVolumeSync`.
//!
//! Validates:
//! 1. Windows master volume 1:1 scalar mapping (0% to 100%).
//! 2. OpenBoom DSP boost clamping at 100% for Windows endpoint (101% to 150%).
//! 3. Bidirectional mute and unmute propagation.
//! 4. Windows OS Volume / Keyboard Media Keys notifications delivery.
//! 5. Feedback loop suppression via `OPEN_BOOM_VOLUME_CONTEXT_GUID` and `is_syncing` atomic guard.
//! 6. Defensive bounds clamping and NaN sanitization.
//! 7. Synthetic simulation and fallback mode resilience.

use open_boom_wasapi::{
    EndpointVolumeSync, VolumeContextGuid, VolumeSyncEvent, OPEN_BOOM_VOLUME_CONTEXT_GUID,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

#[test]
fn test_synthetic_mode_initial_state_and_defaults() {
    let sync = EndpointVolumeSync::with_forced_synthetic(true);
    assert!(sync.is_synthetic(), "Must run in synthetic simulation mode");
    assert!(!sync.is_syncing(), "Syncing flag must be false initially");

    let initial_vol = sync.get_master_volume().expect("get initial volume");
    assert!((initial_vol - 0.8).abs() < 1e-4, "Initial volume default is 0.8");

    let initial_mute = sync.is_muted().expect("get initial mute");
    assert!(!initial_mute, "Initial mute default is false");
}

#[test]
fn test_volume_mapping_1_to_1_under_100_percent() {
    let sync = EndpointVolumeSync::with_forced_synthetic(true);

    // 45% (0.45)
    sync.set_master_volume(0.45).expect("set master volume 0.45");
    let vol_45 = sync.get_master_volume().expect("get master volume");
    assert!((vol_45 - 0.45).abs() < 1e-4, "Windows scalar must map 1:1 to 0.45");

    // 80% (0.80)
    sync.set_master_volume(0.80).expect("set master volume 0.80");
    let vol_80 = sync.get_master_volume().expect("get master volume");
    assert!((vol_80 - 0.80).abs() < 1e-4, "Windows scalar must map 1:1 to 0.80");

    // 100% (1.0)
    sync.set_master_volume(1.0).expect("set master volume 1.0");
    let vol_100 = sync.get_master_volume().expect("get master volume");
    assert!((vol_100 - 1.0).abs() < 1e-4, "Windows scalar must map 1:1 to 1.0");
}

#[test]
fn test_boost_mapping_clamps_windows_volume_at_100_percent() {
    let sync = EndpointVolumeSync::with_forced_synthetic(true);

    // Boost to 135% (1.35) -> Windows endpoint clamped at 1.0 (100%)
    sync.set_master_volume(1.35).expect("set master volume 1.35");
    let vol_135 = sync.get_master_volume().expect("get master volume");
    assert!(
        (vol_135 - 1.0).abs() < 1e-4,
        "Windows scalar endpoint volume must clamp at 1.0 (100%) when boosted to 1.35"
    );

    // Boost to 150% (1.50) -> Windows endpoint clamped at 1.0 (100%)
    sync.set_master_volume(1.50).expect("set master volume 1.50");
    let vol_150 = sync.get_master_volume().expect("get master volume");
    assert!(
        (vol_150 - 1.0).abs() < 1e-4,
        "Windows scalar endpoint volume must clamp at 1.0 (100%) when boosted to 1.50"
    );
}

#[test]
fn test_mute_and_unmute_synchronization() {
    let sync = EndpointVolumeSync::with_forced_synthetic(true);

    // Mute = true
    sync.set_mute(true).expect("set mute true");
    assert!(sync.is_muted().expect("is_muted"), "Mute state must be true");

    // Mute = false
    sync.set_mute(false).expect("set mute false");
    assert!(!sync.is_muted().expect("is_muted"), "Mute state must be false");
}

#[test]
fn test_os_volume_change_notification_callback() {
    let sync = EndpointVolumeSync::with_forced_synthetic(true);

    let received_events = Arc::new(Mutex::new(Vec::<VolumeSyncEvent>::new()));
    let events_clone = received_events.clone();

    sync.set_on_volume_changed(move |event| {
        events_clone.lock().unwrap().push(event);
    });

    // Simulate user adjusting Windows volume slider / multimedia key to 45% (0.45)
    let processed = sync.simulate_os_volume_change(0.45, false, None);
    assert!(processed, "OS volume event must be processed");

    let events = received_events.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert!((events[0].volume - 0.45).abs() < 1e-4);
    assert!(!events[0].is_muted);
    drop(events);

    // Simulate user pressing keyboard Mute key
    let processed_mute = sync.simulate_os_volume_change(0.45, true, None);
    assert!(processed_mute, "OS mute event must be processed");

    let events2 = received_events.lock().unwrap();
    assert_eq!(events2.len(), 2);
    assert!((events2[1].volume - 0.45).abs() < 1e-4);
    assert!(events2[1].is_muted);
}

#[test]
fn test_feedback_loop_suppression_guid_context() {
    let sync = EndpointVolumeSync::with_forced_synthetic(true);

    let callback_count = Arc::new(AtomicU32::new(0));
    let count_clone = callback_count.clone();

    sync.set_on_volume_changed(move |_| {
        count_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Notification originating from OpenBoom carries OPEN_BOOM_VOLUME_CONTEXT_GUID
    let processed = sync.simulate_os_volume_change(
        0.80,
        false,
        Some(OPEN_BOOM_VOLUME_CONTEXT_GUID),
    );

    assert!(
        !processed,
        "Notification with OPEN_BOOM_VOLUME_CONTEXT_GUID must be suppressed"
    );
    assert_eq!(
        callback_count.load(Ordering::SeqCst),
        0,
        "Callback must never be triggered for echo notifications"
    );

    // Notification with a different external GUID (e.g. from Windows Shell / SNDVOL) is accepted
    let external_guid = VolumeContextGuid::from_u128(0x11223344_5566_7788_9900_aabbccddeeff);
    let processed_external = sync.simulate_os_volume_change(
        0.75,
        false,
        Some(external_guid),
    );

    assert!(
        processed_external,
        "External OS volume notification must be accepted"
    );
    assert_eq!(
        callback_count.load(Ordering::SeqCst),
        1,
        "Callback must be called exactly once for external event"
    );
}

#[test]
fn test_defensive_bounds_and_nan_sanitization() {
    let sync = EndpointVolumeSync::with_forced_synthetic(true);

    let last_event = Arc::new(Mutex::new(None::<VolumeSyncEvent>));
    let last_clone = last_event.clone();

    sync.set_on_volume_changed(move |ev| {
        *last_clone.lock().unwrap() = Some(ev);
    });

    // 1. Negative volume clamped to 0.0
    sync.simulate_os_volume_change(-0.5, false, None);
    assert_eq!(last_event.lock().unwrap().unwrap().volume, 0.0);

    // 2. Volume > 1.5 clamped to 1.5
    sync.simulate_os_volume_change(2.5, false, None);
    assert_eq!(last_event.lock().unwrap().unwrap().volume, 1.5);

    // 3. NaN volume sanitized to 0.0
    sync.simulate_os_volume_change(f32::NAN, false, None);
    assert_eq!(last_event.lock().unwrap().unwrap().volume, 0.0);

    // 4. Setting NaN on set_master_volume sanitizes to 0.0
    sync.set_master_volume(f32::NAN).expect("set NaN volume");
    let vol = sync.get_master_volume().expect("get master volume");
    assert_eq!(vol, 0.0);
}

#[test]
fn test_multiple_listeners_receive_events() {
    let sync = EndpointVolumeSync::with_forced_synthetic(true);

    let count1 = Arc::new(AtomicU32::new(0));
    let count2 = Arc::new(AtomicU32::new(0));
    let c1 = count1.clone();
    let c2 = count2.clone();

    sync.add_on_volume_changed(move |_| {
        c1.fetch_add(1, Ordering::SeqCst);
    });
    sync.add_on_volume_changed(move |_| {
        c2.fetch_add(1, Ordering::SeqCst);
    });

    let processed = sync.simulate_os_volume_change(0.65, false, None);
    assert!(processed);
    assert_eq!(count1.load(Ordering::SeqCst), 1);
    assert_eq!(count2.load(Ordering::SeqCst), 1);
}

#[test]
fn test_rebind_and_synthetic_fallback_constructors() {
    // Test with_synthetic_fallback constructor
    let fallback_sync = EndpointVolumeSync::with_synthetic_fallback(true);
    assert!(fallback_sync.allows_synthetic_fallback());

    // Rebind endpoint should succeed without panic
    let rebind_res = fallback_sync.rebind_endpoint(Some("new_device_id".to_string()));
    assert!(rebind_res.is_ok(), "Rebind must succeed");

    let rebind_default = fallback_sync.rebind_endpoint(None);
    assert!(rebind_default.is_ok(), "Rebind default must succeed");
}
