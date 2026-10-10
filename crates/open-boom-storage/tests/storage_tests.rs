//! Unit and integration tests for OpenBoom atomic storage and startup management.

use open_boom_storage::{AppConfig, PresetStore, StartupManager};
use std::fs;

#[test]
fn test_app_config_default_values() {
    let cfg = AppConfig::default();
    assert!(cfg.master_power);
    assert!((cfg.master_volume - 0.8).abs() < 1e-6);
    assert!((cfg.master_boost - 1.0).abs() < 1e-6);
    assert!(!cfg.is_muted);
    assert_eq!(cfg.active_output_device, None);
    assert_eq!(cfg.window_width, 840);
    assert_eq!(cfg.window_height, 560);
    assert_eq!(cfg.window_x, None);
    assert_eq!(cfg.window_y, None);
    assert!(cfg.surround_enabled);
    assert!(!cfg.bypass_active);
    assert!(!cfg.launch_on_startup);
}

#[test]
fn test_atomic_config_save_and_load() {
    let temp_dir = std::env::temp_dir().join(format!("open_boom_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let store = PresetStore::new(&temp_dir);

    // Initial load when file does not exist -> default config
    let loaded = store.load_config().expect("load default config");
    assert_eq!(loaded, AppConfig::default());

    // Modify and save config
    let mut custom_cfg = AppConfig::default();
    custom_cfg.master_power = false;
    custom_cfg.master_volume = 1.35;
    custom_cfg.is_muted = true;
    custom_cfg.active_output_device = Some("Headphones_1".to_string());
    custom_cfg.window_x = Some(150);
    custom_cfg.window_y = Some(200);
    custom_cfg.bypass_active = true;
    custom_cfg.launch_on_startup = true;

    store.save_config(&custom_cfg).expect("save config");

    // Verify file exists
    assert!(store.config_path().exists());
    // Verify no temporary files left behind
    let tmp_path = temp_dir.join(format!("config.{}.tmp", std::process::id()));
    assert!(!tmp_path.exists(), "Temporary staging file should be renamed away");

    // Reload and assert equality
    let reloaded = store.load_config().expect("reload config");
    assert_eq!(reloaded, custom_cfg);

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_corrupt_config_json_recovery() {
    let temp_dir = std::env::temp_dir().join(format!("open_boom_corrupt_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let store = PresetStore::new(&temp_dir);
    let config_file = store.config_path();

    // Write completely corrupted JSON
    fs::write(&config_file, b"INVALID_GARBAGE_JSON_DATA{{{]]]:::").unwrap();

    // Loading should not crash or panic; must recover defaults cleanly
    let recovered = store.load_config().expect("recover default config on corruption");
    assert_eq!(recovered, AppConfig::default());

    // Write partially broken JSON (missing braces)
    fs::write(&config_file, b"{\"master_power\": ").unwrap();
    let recovered2 = store.load_config().expect("recover default on partial json");
    assert_eq!(recovered2, AppConfig::default());

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_partial_json_field_defaults() {
    let temp_dir = std::env::temp_dir().join(format!("open_boom_partial_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let store = PresetStore::new(&temp_dir);
    let config_file = store.config_path();

    // JSON containing only some fields
    let partial = r#"{
        "master_volume": 1.25,
        "is_muted": true
    }"#;
    fs::write(&config_file, partial).unwrap();

    let loaded = store.load_config().expect("load partial json");
    assert!((loaded.master_volume - 1.25).abs() < 1e-6);
    assert!(loaded.is_muted);
    // Other fields must have defaults
    assert_eq!(loaded.window_width, 840);
    assert_eq!(loaded.window_height, 560);
    assert!(loaded.master_power);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_startup_manager_mock_lifecycle() {
    let startup = StartupManager::mock().with_app_name("OpenBoomTest");
    assert!(startup.is_mock());
    assert_eq!(startup.app_name(), "OpenBoomTest");

    // Initially disabled
    assert!(!startup.is_enabled().expect("check enabled"));
    assert_eq!(startup.get_registered_path().expect("get path"), None);

    // Disable on non-existent entry succeeds cleanly
    assert!(startup.disable().is_ok());

    // Enable with mock path
    let test_path = r#""C:\Program Files\OpenBoom\open-boom.exe" --minimized"#;
    startup.enable(test_path).expect("enable startup");

    // Verify enabled
    assert!(startup.is_enabled().expect("check enabled after set"));
    assert_eq!(
        startup.get_registered_path().expect("get path after set"),
        Some(test_path.to_string())
    );

    // Disable
    startup.disable().expect("disable startup");
    assert!(!startup.is_enabled().expect("check enabled after disable"));
    assert_eq!(startup.get_registered_path().expect("get path after disable"), None);

    // Disable again is idempotent and graceful
    assert!(startup.disable().is_ok());
}

#[test]
fn test_preset_store_default_dir_resolution() {
    let dir = PresetStore::default_dir();
    assert!(dir.to_string_lossy().contains("OpenBoom"));
    let store = PresetStore::default();
    assert_eq!(store.base_dir(), dir);
}
