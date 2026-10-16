//! Unit and integration tests for OpenBoom atomic storage, EQ presets, and startup management.

use open_boom_core::types::GRAPHIC_EQ_BANDS;
use open_boom_storage::{AppConfig, AppVolumeProfile, PresetStore, StartupManager};
use std::collections::HashSet;
use std::fs;
use std::io;

#[test]
fn test_app_config_default_values() {
    let cfg = AppConfig::default();
    assert!(cfg.master_power);
    assert!((cfg.master_volume - 0.8).abs() < 1e-6);
    assert!((cfg.master_boost - 1.0).abs() < 1e-6);
    assert!(!cfg.is_muted);
    assert_eq!(cfg.active_output_device, None);
    assert_eq!(cfg.window_width, 1000);
    assert_eq!(cfg.window_height, 620);
    assert_eq!(cfg.window_x, None);
    assert_eq!(cfg.window_y, None);
    assert!(cfg.surround_enabled);
    assert!(!cfg.bypass_active);
    assert!(!cfg.launch_on_startup);
    assert_eq!(cfg.autoeq_profile_id, None);
    assert_eq!(cfg.active_preset_name, "Flat");
    assert!(cfg.sweetener_ambience_enabled);
    assert!((cfg.sweetener_ambience_amount - 0.35).abs() < 1e-6);
    assert!(cfg.sweetener_fidelity_enabled);
    assert!((cfg.sweetener_fidelity_amount - 0.45).abs() < 1e-6);
    assert!(!cfg.sweetener_night_mode_enabled);
    assert!((cfg.sweetener_night_mode_amount - 0.50).abs() < 1e-6);
    assert!(cfg.sweetener_spatial_enabled);
    assert!((cfg.sweetener_spatial_amount - 0.60).abs() < 1e-6);
    assert_eq!(cfg.sweetener_pitch_semitones, 0);
    assert!(cfg.buffer_guard_enabled);
    assert!(!cfg.orbit_enabled);
    assert_eq!(cfg.orbit_direction, open_boom_storage::OrbitDirection::Clockwise);
    assert!((cfg.orbit_period_sec - 12.0).abs() < 1e-6);
    assert!((cfg.orbit_depth - 0.85).abs() < 1e-6);
    assert_eq!(cfg.language, None);
    assert!(cfg.check_updates_on_startup);
    assert!(!cfg.auto_install_updates);
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
    custom_cfg.autoeq_profile_id = Some("hyperx-cloud-ii-oratory1990".to_string());
    custom_cfg.active_preset_name = "Rock".to_string();

    store.save_config(&custom_cfg).expect("save config");

    // Verify file exists
    assert!(store.config_path().exists());
    // Verify no temporary files left behind
    for entry in fs::read_dir(&temp_dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(
            !name.ends_with(".tmp"),
            "Temporary staging file should be renamed away: {}",
            name
        );
    }

    // Reload and assert equality
    let reloaded = store.load_config().expect("reload config");
    assert_eq!(reloaded, custom_cfg);
    assert_eq!(
        reloaded.autoeq_profile_id,
        Some("hyperx-cloud-ii-oratory1990".to_string())
    );
    assert_eq!(reloaded.active_preset_name, "Rock");

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
    let recovered = store
        .load_config()
        .expect("recover default config on corruption");
    assert_eq!(recovered, AppConfig::default());

    // Write partially broken JSON (missing braces)
    fs::write(&config_file, b"{\"master_power\": ").unwrap();
    let recovered2 = store
        .load_config()
        .expect("recover default on partial json");
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
    assert_eq!(loaded.window_width, 1000);
    assert_eq!(loaded.window_height, 620);
    assert!(loaded.master_power);
    assert!(!loaded.orbit_enabled);
    assert_eq!(loaded.orbit_direction, open_boom_storage::OrbitDirection::Clockwise);
    assert!((loaded.orbit_period_sec - 12.0).abs() < 1e-6);
    assert!((loaded.orbit_depth - 0.85).abs() < 1e-6);
    assert!(loaded.check_updates_on_startup);
    assert!(!loaded.auto_install_updates);
    assert_eq!(loaded.audio_bridge_mode, "auto");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_orbital_config_persistence_and_legacy_json_compatibility() {
    let temp_dir = std::env::temp_dir().join(format!("open_boom_orbit_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let store = PresetStore::new(&temp_dir);

    // 1. Deserializing legacy JSON missing all 8D fields should yield fallback defaults
    let legacy_json = r#"{
        "master_power": true,
        "master_volume": 0.8,
        "master_boost": 1.0,
        "surround_enabled": true
    }"#;
    fs::write(store.config_path(), legacy_json).unwrap();

    let loaded_legacy = store.load_config().expect("load legacy config");
    assert!(!loaded_legacy.orbit_enabled);
    assert_eq!(loaded_legacy.orbit_direction, open_boom_storage::OrbitDirection::Clockwise);
    assert!((loaded_legacy.orbit_period_sec - 12.0).abs() < 1e-6);
    assert!((loaded_legacy.orbit_depth - 0.85).abs() < 1e-6);
    assert_eq!(loaded_legacy.audio_bridge_mode, "auto");

    // 2. Custom 8D settings are saved atomically and reloaded faithfully
    let mut custom_cfg = AppConfig::default();
    custom_cfg.orbit_enabled = true;
    custom_cfg.orbit_direction = open_boom_storage::OrbitDirection::CounterClockwise;
    custom_cfg.orbit_period_sec = 6.5;
    custom_cfg.orbit_depth = 0.50;
    custom_cfg.audio_bridge_mode = "loopback".to_string();

    store.save_config(&custom_cfg).expect("save custom orbit config");

    let loaded_custom = store.load_config().expect("load custom orbit config");
    assert!(loaded_custom.orbit_enabled);
    assert_eq!(loaded_custom.orbit_direction, open_boom_storage::OrbitDirection::CounterClockwise);
    assert!((loaded_custom.orbit_period_sec - 6.5).abs() < 1e-6);
    assert!((loaded_custom.orbit_depth - 0.50).abs() < 1e-6);
    assert_eq!(loaded_custom.audio_bridge_mode, "loopback");

    // Test PingPong direction persistence as well
    custom_cfg.orbit_direction = open_boom_storage::OrbitDirection::PingPong;
    custom_cfg.orbit_period_sec = 24.0;
    custom_cfg.orbit_depth = 1.0;
    store.save_config(&custom_cfg).expect("save pingpong orbit config");

    let loaded_pingpong = store.load_config().expect("load pingpong orbit config");
    assert_eq!(loaded_pingpong.orbit_direction, open_boom_storage::OrbitDirection::PingPong);
    assert!((loaded_pingpong.orbit_period_sec - 24.0).abs() < 1e-6);
    assert!((loaded_pingpong.orbit_depth - 1.0).abs() < 1e-6);

    // 3. Test defensive sanitization against out-of-bounds numbers, NaN, and invalid/untrimmed audio_bridge_mode
    let corrupt_bounds_json = r#"{
        "orbit_enabled": true,
        "orbit_direction": "Clockwise",
        "orbit_period_sec": 999.0,
        "orbit_depth": -0.5,
        "audio_bridge_mode": "invalid_channel"
    }"#;
    fs::write(store.config_path(), corrupt_bounds_json).unwrap();
    let loaded_sanitized = store.load_config().expect("load corrupt bounds config");
    assert_eq!(loaded_sanitized.orbit_period_sec, 12.0);
    assert_eq!(loaded_sanitized.orbit_depth, 0.85);
    assert_eq!(loaded_sanitized.audio_bridge_mode, "auto");

    let untrimmed_bridge_json = r#"{
        "audio_bridge_mode": "  VIRTUAL  "
    }"#;
    fs::write(store.config_path(), untrimmed_bridge_json).unwrap();
    let loaded_untrimmed = store.load_config().expect("load untrimmed bridge mode config");
    assert_eq!(loaded_untrimmed.audio_bridge_mode, "virtual");

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
    assert_eq!(
        startup.get_registered_path().expect("get path after disable"),
        None
    );

    // Disable again is idempotent and graceful
    assert!(startup.disable().is_ok());
}

#[test]
fn test_preset_store_default_dir_resolution() {
    let dir = PresetStore::default_dir();
    assert!(dir.to_string_lossy().contains("OpenBoom"));
    #[cfg(target_os = "macos")]
    {
        if std::env::var("HOME").is_ok() {
            assert!(
                dir.ends_with("Library/Application Support/OpenBoom"),
                "On macOS with HOME set, default_dir must resolve to Library/Application Support/OpenBoom: {:?}",
                dir
            );
        }
    }
    #[cfg(target_os = "windows")]
    {
        if std::env::var("APPDATA").is_ok() || std::env::var("USERPROFILE").is_ok() {
            assert!(
                dir.ends_with("AppData\\Roaming\\OpenBoom") || dir.ends_with("OpenBoom"),
                "On Windows, default_dir must resolve to AppData/Roaming/OpenBoom: {:?}",
                dir
            );
        }
    }
    let store = PresetStore::default();
    assert_eq!(store.base_dir(), dir);
    assert_eq!(store.presets_dir(), dir.join("presets"));
}

#[test]
#[cfg(target_os = "macos")]
fn test_preset_store_default_dir_macos_home_resolution() {
    let dir = PresetStore::default_dir();
    if let Ok(home) = std::env::var("HOME") {
        let expected = std::path::PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("OpenBoom");
        assert_eq!(dir, expected);
    } else {
        let expected = std::env::temp_dir().join("OpenBoom");
        assert_eq!(dir, expected);
    }
}

#[test]
fn test_fifteen_factory_presets_integrity() {
    let factory = PresetStore::factory_presets();
    assert_eq!(factory.len(), 15, "Must provide all 15 factory presets (FR11)");

    let required_names = [
        "Flat",
        "Bass Boost",
        "Treble Boost",
        "Acoustic",
        "Classical",
        "Electronic",
        "Gaming",
        "Hip-Hop",
        "Jazz",
        "Movie",
        "Pop",
        "Rock",
        "Vocal",
        "Lounge",
        "Deep",
    ];

    let mut seen_ids = HashSet::new();
    let mut seen_names = HashSet::new();

    for expected_name in required_names {
        let preset = factory
            .iter()
            .find(|p| p.name == expected_name)
            .unwrap_or_else(|| panic!("Missing required factory preset '{}'", expected_name));
        assert!(preset.is_factory);
        assert!(!preset.id.is_empty());
        assert!(!preset.category.is_empty());
        assert!(seen_ids.insert(preset.id.clone()), "Duplicate factory id");
        assert!(
            seen_names.insert(preset.name.clone()),
            "Duplicate factory name"
        );
        assert_eq!(preset.gains_db.len(), GRAPHIC_EQ_BANDS);
        for &g in &preset.gains_db {
            assert!(g.is_finite(), "Gain in {} must be finite", preset.name);
            assert!(
                (-12.0..=12.0).contains(&g),
                "Gain {} in {} out of [-12, +12] dB bounds",
                g,
                preset.name
            );
        }
    }

    // Flat must be strictly 0.0 dB across all 31 bands
    let flat = factory.iter().find(|p| p.name == "Flat").unwrap();
    assert_eq!(flat.gains_db, [0.0; GRAPHIC_EQ_BANDS]);
}

#[test]
fn test_custom_preset_atomic_crud_lifecycle() {
    let temp_dir =
        std::env::temp_dir().join(format!("open_boom_presets_crud_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let store = PresetStore::new(&temp_dir);

    // Initially only 15 factory presets exist
    let initial = store.list_presets();
    assert_eq!(initial.len(), 15);
    assert!(store.list_custom_presets().is_empty());

    // 1. Save custom preset
    let mut gains = [0.0f32; GRAPHIC_EQ_BANDS];
    gains[0] = 4.5;
    gains[17] = -2.0;
    gains[30] = 3.5;

    let saved = store
        .save_custom_preset("My Late Night EQ", gains)
        .expect("save custom preset");
    assert_eq!(saved.id, "my-late-night-eq");
    assert_eq!(saved.name, "My Late Night EQ");
    assert_eq!(saved.category, "Custom");
    assert!(!saved.is_factory);
    assert_eq!(saved.gains_db, gains);

    let expected_file = store.presets_dir().join("my-late-night-eq.json");
    assert!(expected_file.exists(), "Custom preset JSON must exist on disk");

    // Verify no orphaned .tmp files in presets directory
    for entry in fs::read_dir(store.presets_dir()).unwrap().flatten() {
        let fname = entry.file_name().to_string_lossy().to_string();
        assert!(
            !fname.ends_with(".tmp"),
            "Orphaned .tmp file found: {}",
            fname
        );
    }

    // 2. Load custom preset
    let loaded = store
        .load_custom_preset("my-late-night-eq")
        .expect("load custom preset");
    assert_eq!(loaded, saved);

    // 3. List all presets -> 15 factory + 1 custom = 16
    let all = store.list_presets();
    assert_eq!(all.len(), 16);
    assert_eq!(all[15], saved);

    // 4. Rename custom preset
    let renamed = store
        .rename_custom_preset("my-late-night-eq", "Warm Midnight")
        .expect("rename custom preset");
    assert_eq!(renamed.id, "warm-midnight");
    assert_eq!(renamed.name, "Warm Midnight");
    assert_eq!(renamed.gains_db, gains);
    assert!(
        !expected_file.exists(),
        "Old preset file must be removed after rename"
    );
    assert!(store.presets_dir().join("warm-midnight.json").exists());

    // 5. Delete custom preset
    store
        .delete_custom_preset("warm-midnight")
        .expect("delete custom preset");
    assert!(!store.presets_dir().join("warm-midnight.json").exists());
    assert_eq!(store.list_presets().len(), 15);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_factory_preset_protection_and_path_sanitization() {
    let temp_dir = std::env::temp_dir().join(format!(
        "open_boom_presets_protect_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let store = PresetStore::new(&temp_dir);
    let gains = [1.0f32; GRAPHIC_EQ_BANDS];

    // 1. Reject overwriting factory presets (case-insensitive & slug match)
    for forbidden in ["Rock", "rock", "BASS BOOST", "bass_boost", "Hip-Hop", "flat"] {
        let err = store
            .save_custom_preset(forbidden, gains)
            .expect_err("must reject factory preset overwrite");
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
    }

    // 2. Reject deleting factory presets
    for forbidden in ["Rock", "Gaming", "flat", "bass-boost"] {
        let err = store
            .delete_custom_preset(forbidden)
            .expect_err("must reject factory preset deletion");
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
    }

    // 3. Reject renaming a factory preset or renaming a custom preset to a factory name
    store
        .save_custom_preset("Custom One", gains)
        .expect("save Custom One");
    let err_from_factory = store
        .rename_custom_preset("Rock", "My Rock")
        .expect_err("must reject renaming factory preset");
    assert_eq!(err_from_factory.kind(), io::ErrorKind::PermissionDenied);

    let err_to_factory = store
        .rename_custom_preset("custom-one", "Classical")
        .expect_err("must reject renaming custom preset to factory name");
    assert_eq!(err_to_factory.kind(), io::ErrorKind::PermissionDenied);

    // 4. Sanitize path traversal & special characters safely
    let traversal = store
        .save_custom_preset("../../evil\\path/Preset!@#", gains)
        .expect("sanitize path traversal");
    assert_eq!(traversal.id, "evil-path-preset");
    assert!(store.presets_dir().join("evil-path-preset.json").exists());

    // 5. Reject empty, punctuation-only, and Windows reserved device names
    for invalid in ["", "   ", "...///\\\\", "CON", "nul", "COM1", "lpt9"] {
        let err = store
            .save_custom_preset(invalid, gains)
            .expect_err(&format!("must reject invalid preset name '{}'", invalid));
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_corrupt_preset_json_and_gain_clamping_resilience() {
    let temp_dir = std::env::temp_dir().join(format!(
        "open_boom_presets_corrupt_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let store = PresetStore::new(&temp_dir);
    fs::create_dir_all(store.presets_dir()).unwrap();

    // Write a corrupt JSON file into presets_dir
    fs::write(
        store.presets_dir().join("broken-preset.json"),
        b"{ invalid json [[[",
    )
    .unwrap();

    // Write a file attempting to shadow factory preset "rock.json"
    fs::write(
        store.presets_dir().join("rock.json"),
        r#"{"id":"rock","name":"Rock","category":"Custom","is_factory":false,"gains_db":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]}"#,
    )
    .unwrap();

    // Save a valid custom preset with out-of-range / NaN gains
    let mut extreme_gains = [0.0f32; GRAPHIC_EQ_BANDS];
    extreme_gains[0] = 25.0; // > +12.0 dB
    extreme_gains[1] = -30.0; // < -12.0 dB
    extreme_gains[2] = f32::NAN;

    let saved = store
        .save_custom_preset("Valid Studio Curve", extreme_gains)
        .expect("save clamped preset");
    assert_eq!(saved.gains_db[0], 12.0);
    assert_eq!(saved.gains_db[1], -12.0);
    assert_eq!(saved.gains_db[2], 0.0);

    // Listing presets must skip the corrupt file and the shadowed factory file without crashing
    let presets = store.list_presets();
    assert_eq!(presets.len(), 16, "15 factory + 1 valid custom preset");
    let rock = presets.iter().find(|p| p.id == "rock").unwrap();
    assert!(rock.is_factory, "Factory 'rock' must not be shadowed by disk file");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_app_volume_persistence_atomic_save_and_load() {
    let temp_dir = std::env::temp_dir().join(format!("open_boom_appvol_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let store = PresetStore::new(&temp_dir);

    // Initial load when file does not exist -> empty map and list
    let map = store.load_app_volumes().expect("load initial empty map");
    assert!(map.is_empty());
    let list = store.load_app_volume_list().expect("load initial empty list");
    assert!(list.is_empty());

    // Save individual profiles
    store
        .save_app_volume_profile(&AppVolumeProfile::new("Spotify.exe", 0.75, false))
        .expect("save spotify profile");
    store
        .save_app_volume_profile(&AppVolumeProfile::new("discord.exe", 0.60, true))
        .expect("save discord profile");
    store
        .save_app_volume_profile(&AppVolumeProfile::new("Chrome.exe", 1.80, false)) // Should clamp to 1.50
        .expect("save chrome profile");

    // Verify file exists
    assert!(store.app_volumes_path().exists());

    // Verify no temporary files left behind
    for entry in fs::read_dir(&temp_dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(
            !name.ends_with(".tmp"),
            "Temporary staging file must be renamed away: {}",
            name
        );
    }

    // Verify case-insensitive lookup
    let spotify = store
        .get_app_volume_profile("spotify.exe")
        .expect("get spotify")
        .expect("found spotify");
    assert_eq!(spotify.process_name, "Spotify.exe");
    assert!((spotify.volume - 0.75).abs() < 1e-6);
    assert!(!spotify.is_muted);

    let spotify_upper = store
        .get_app_volume_profile("SPOTIFY.EXE")
        .expect("get SPOTIFY.EXE")
        .expect("found case-insensitive");
    assert_eq!(spotify_upper.process_name, "Spotify.exe");

    let chrome = store
        .get_app_volume_profile("chrome.exe")
        .expect("get chrome")
        .expect("found chrome");
    assert!((chrome.volume - 1.50).abs() < 1e-6, "Volume should clamp to 1.5");

    let discord = store
        .get_app_volume_profile("Discord.exe")
        .expect("get discord")
        .expect("found discord");
    assert!(discord.is_muted);

    // Verify sorted list
    let list = store.load_app_volume_list().expect("load list");
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].process_name, "Chrome.exe");
    assert_eq!(list[1].process_name, "discord.exe");
    assert_eq!(list[2].process_name, "Spotify.exe");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_app_volume_update_and_delete() {
    let temp_dir = std::env::temp_dir().join(format!("open_boom_appvol_update_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let store = PresetStore::new(&temp_dir);

    store
        .save_app_volume_profile(&AppVolumeProfile::new("game.exe", 1.0, false))
        .expect("save game");

    // Update volume and mute
    store
        .save_app_volume_profile(&AppVolumeProfile::new("GAME.EXE", 0.4, true))
        .expect("update game");

    let updated = store
        .get_app_volume_profile("game.exe")
        .expect("query game")
        .expect("game exists");
    assert!((updated.volume - 0.4).abs() < 1e-6);
    assert!(updated.is_muted);

    // Delete profile
    let deleted = store.delete_app_volume_profile("Game.exe").expect("delete game");
    assert!(deleted);

    let query_after = store.get_app_volume_profile("game.exe").expect("query after delete");
    assert_eq!(query_after, None);

    let delete_again = store.delete_app_volume_profile("game.exe").expect("delete again");
    assert!(!delete_again);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_app_volume_corrupt_file_recovery() {
    let temp_dir = std::env::temp_dir().join(format!("open_boom_appvol_corrupt_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let store = PresetStore::new(&temp_dir);

    // Write completely invalid corrupt JSON to app_volumes.json
    fs::write(store.app_volumes_path(), b"INVALID CORRUPTED JSON {{{(((").unwrap();

    // Loading should not crash; must recover gracefully to empty map
    let loaded = store.load_app_volumes().expect("load corrupt file");
    assert!(loaded.is_empty());

    // Subsequent save should safely overwrite corrupt file atomically
    store
        .save_app_volume_profile(&AppVolumeProfile::new("recovered.exe", 0.9, false))
        .expect("save after corruption");

    let profile = store
        .get_app_volume_profile("recovered.exe")
        .expect("get recovered")
        .expect("profile exists");
    assert!((profile.volume - 0.9).abs() < 1e-6);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_app_volume_array_fallback_format() {
    let temp_dir = std::env::temp_dir().join(format!("open_boom_appvol_array_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let store = PresetStore::new(&temp_dir);

    // Write JSON formatted as an array instead of a map
    let json_array = r#"[
        {"process_name": "player.exe", "volume": 1.2, "is_muted": false},
        {"process_name": "voice.exe", "volume": 0.8, "is_muted": true}
    ]"#;
    fs::write(store.app_volumes_path(), json_array.as_bytes()).unwrap();

    let profiles = store.load_app_volumes().expect("parse array format");
    assert_eq!(profiles.len(), 2);

    let player = profiles.get("player.exe").expect("player exists");
    assert!((player.volume - 1.2).abs() < 1e-6);
    assert!(!player.is_muted);

    let voice = profiles.get("voice.exe").expect("voice exists");
    assert!((voice.volume - 0.8).abs() < 1e-6);
    assert!(voice.is_muted);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_macos_launch_agent_plist_path() {
    let startup = StartupManager::new();
    #[cfg(target_os = "macos")]
    {
        let path = startup.macos_launch_agent_plist_path();
        let path_str = path.to_string_lossy();
        assert!(path_str.ends_with("Library/LaunchAgents/com.openboom.launcher.plist"));
    }
    let _ = startup;
}

