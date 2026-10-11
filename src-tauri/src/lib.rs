//! Desktop shell runtime and system tray lifecycle manager for OpenBoom.

use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{AudioCommand, AudioDeviceInfo, TelemetrySnapshot, GRAPHIC_EQ_BANDS};
use open_boom_storage::{
    AppConfig, AutoEqDatabase, AutoEqProfile, AutoEqProfileSummary, AutoEqStats, PresetStore,
    StartupManager,
};
use open_boom_wasapi::{
    AudioDeviceManager, AudioEngineCoordinator, AudioStreamPump, LoopbackConfig, RenderConfig,
    StreamPumpConfig,
};
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};

pub mod dispatcher;
use dispatcher::{UiDispatcher, COMMAND_QUEUE_CAPACITY};

pub struct AppState {
    pub dispatcher: UiDispatcher,
    pub command_queue: Arc<SpscQueue<AudioCommand, COMMAND_QUEUE_CAPACITY>>,
    pub coordinator: Arc<Mutex<AudioEngineCoordinator>>,
    pub store: PresetStore,
    pub startup: StartupManager,
    pub autoeq_db: Arc<AutoEqDatabase>,
    pub pump: Arc<Mutex<Option<AudioStreamPump>>>,
    pub active_physical_device_id: Arc<Mutex<Option<String>>>,
    pub virtual_device_id: Arc<Mutex<Option<String>>>,
    pub is_routed_to_virtual: Arc<Mutex<bool>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::with_routing_options(false)
    }

    pub fn new_runtime() -> Self {
        Self::with_routing_options(true)
    }

    pub fn with_routing_options(auto_route: bool) -> Self {
        let command_queue = SpscQueue::<AudioCommand, COMMAND_QUEUE_CAPACITY>::new();
        let dispatcher = UiDispatcher::new(command_queue.clone());
        let render_config = RenderConfig {
            allow_synthetic_fallback: true,
            ..Default::default()
        };
        let coordinator = Arc::new(Mutex::new(AudioEngineCoordinator::with_command_queue(
            LoopbackConfig::default(),
            render_config,
            command_queue.clone(),
        )));
        let store = PresetStore::default();
        let startup = StartupManager::new();

        let device_mgr = AudioDeviceManager::new();
        let physical_default = device_mgr.get_physical_default_device().ok().flatten();
        let physical_id = physical_default.as_ref().map(|d| d.id.clone());
        let virtual_dev = device_mgr.find_virtual_device().ok().flatten();
        let virtual_id = virtual_dev.as_ref().map(|d| d.id.clone());

        let pump_instance = AudioStreamPump::start(
            coordinator.clone(),
            StreamPumpConfig {
                target_device_id: physical_id.clone(),
                capture_device_id: virtual_id.clone(),
                allow_synthetic: true,
            },
        ).ok();
        let pump = Arc::new(Mutex::new(pump_instance));

        let is_routed_to_virtual = Arc::new(Mutex::new(false));

        // If auto_route requested, virtual device present and master power is ON (default true), route to virtual sink
        if auto_route {
            if let Some(ref v_id) = virtual_id {
                if device_mgr.set_default_device(v_id).is_ok() {
                    if let Ok(mut routed) = is_routed_to_virtual.lock() {
                        *routed = true;
                    }
                }
            }
        }

        let autoeq_db = Arc::new(AutoEqDatabase::default());

        Self {
            dispatcher,
            command_queue,
            coordinator,
            store,
            startup,
            autoeq_db,
            pump,
            active_physical_device_id: Arc::new(Mutex::new(physical_id)),
            virtual_device_id: Arc::new(Mutex::new(virtual_id)),
            is_routed_to_virtual,
        }
    }

    pub fn with_store_and_startup(store: PresetStore, startup: StartupManager) -> Self {
        let mut state = Self::new();
        state.store = store;
        state.startup = startup;
        state
    }

    pub fn handle_master_power_toggle(&self, power: bool) -> Result<(), String> {
        let coord = self.coordinator.lock().map_err(|e| e.to_string())?;
        let device_mgr = coord.device_manager().clone();
        drop(coord);

        let physical_id = self.active_physical_device_id.lock().ok().and_then(|g| g.clone());
        let virtual_id = self.virtual_device_id.lock().ok().and_then(|g| g.clone());

        if power {
            if let Some(ref v_id) = virtual_id {
                let _ = device_mgr.set_default_device(v_id);
                if let Ok(mut routed) = self.is_routed_to_virtual.lock() {
                    *routed = true;
                }
            }
        } else {
            if let Some(ref p_id) = physical_id {
                let _ = device_mgr.set_default_device(p_id);
                if let Ok(mut routed) = self.is_routed_to_virtual.lock() {
                    *routed = false;
                }
            }
        }
        Ok(())
    }

    pub fn restore_physical_default_device(&self) {
        let was_routed = self.is_routed_to_virtual.lock().ok().map(|g| *g).unwrap_or(false);
        if was_routed {
            let physical_id = self.active_physical_device_id.lock().ok().and_then(|g| g.clone());
            if let Some(ref p_id) = physical_id {
                let device_mgr = AudioDeviceManager::new();
                let _ = device_mgr.set_default_device(p_id);
                if let Ok(mut routed) = self.is_routed_to_virtual.lock() {
                    *routed = false;
                }
            }
        }
    }

    pub fn get_telemetry_snapshot(&self) -> TelemetrySnapshot {
        if let Ok(mut coord) = self.coordinator.lock() {
            coord.drain_commands();
            let mut snapshot = coord.telemetry_snapshot();
            let pump_running = self
                .pump
                .lock()
                .ok()
                .and_then(|p| p.as_ref().map(|x| x.is_running()))
                .unwrap_or(false);
            snapshot.dsp_active = (coord.is_running() || pump_running) && coord.processor().is_dsp_active();
            snapshot
        } else {
            TelemetrySnapshot::default()
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        self.restore_physical_default_device();
    }
}

#[tauri::command]
fn dispatch_command(cmd: AudioCommand, state: State<'_, AppState>) -> Result<(), String> {
    match &cmd {
        AudioCommand::SetMasterPower(power) => {
            let _ = state.handle_master_power_toggle(*power);
        }
        AudioCommand::SetBypass(bypassed) => {
            let _ = state.handle_master_power_toggle(!*bypassed);
        }
        _ => {}
    }
    state.dispatcher.send(cmd).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_telemetry(state: State<'_, AppState>) -> TelemetrySnapshot {
    state.get_telemetry_snapshot()
}

#[tauri::command]
fn get_audio_devices(state: State<'_, AppState>) -> Result<Vec<AudioDeviceInfo>, String> {
    let coord = state.coordinator.lock().map_err(|e| e.to_string())?;
    let mut devices = coord.device_manager().enumerate_physical_devices().map_err(|e| e.to_string())?;
    drop(coord);

    let active_id = state.active_physical_device_id.lock().ok().and_then(|g| g.clone());
    if let Some(ref id) = active_id {
        for dev in &mut devices {
            dev.is_default = &dev.id == id;
        }
    }

    Ok(devices)
}

#[tauri::command]
fn set_audio_device(device_id: Option<String>, state: State<'_, AppState>) -> Result<(), String> {
    let mut coord = state.coordinator.lock().map_err(|e| e.to_string())?;
    coord.switch_device(device_id.clone()).map_err(|e| e.to_string())?;
    let device_mgr = coord.device_manager().clone();
    drop(coord);

    if let Ok(mut target_guard) = state.active_physical_device_id.lock() {
        *target_guard = device_id.clone();
    }

    let virtual_id = state.virtual_device_id.lock().ok().and_then(|g| g.clone());

    if let Ok(mut pump_guard) = state.pump.lock() {
        if let Some(mut existing) = pump_guard.take() {
            existing.stop();
        }
        *pump_guard = AudioStreamPump::start(
            state.coordinator.clone(),
            StreamPumpConfig {
                target_device_id: device_id.clone(),
                capture_device_id: virtual_id,
                allow_synthetic: true,
            },
        ).ok();
    }

    // If master power is currently OFF, route Windows default directly to the new physical device
    let power = state.coordinator.lock().map(|c| c.processor().master_power()).unwrap_or(true);
    if !power {
        if let Some(ref id) = device_id {
            let _ = device_mgr.set_default_device(id);
        }
    }

    Ok(())
}

#[tauri::command]
fn trigger_device_fallback(state: State<'_, AppState>) -> Result<(), String> {
    let mut coord = state.coordinator.lock().map_err(|e| e.to_string())?;
    coord.handle_device_lost().map_err(|e| e.to_string())?;
    let device_mgr = coord.device_manager().clone();
    drop(coord);

    let physical_default = device_mgr.get_physical_default_device().ok().flatten();
    let fallback_id = physical_default.map(|d| d.id);

    if let Ok(mut target_guard) = state.active_physical_device_id.lock() {
        *target_guard = fallback_id.clone();
    }

    let virtual_id = state.virtual_device_id.lock().ok().and_then(|g| g.clone());

    if let Ok(mut pump_guard) = state.pump.lock() {
        if let Some(mut existing) = pump_guard.take() {
            existing.stop();
        }
        *pump_guard = AudioStreamPump::start(
            state.coordinator.clone(),
            StreamPumpConfig {
                target_device_id: fallback_id,
                capture_device_id: virtual_id,
                allow_synthetic: true,
            },
        ).ok();
    }
    Ok(())
}


#[tauri::command]
fn minimize_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn close_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn load_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    state.store.load_config().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_app_config(config: AppConfig, state: State<'_, AppState>) -> Result<(), String> {
    state.store.save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_launch_on_startup(state: State<'_, AppState>) -> Result<bool, String> {
    state.startup.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_launch_on_startup(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    if enabled {
        state.startup.enable_current_exe().map_err(|e| e.to_string())
    } else {
        state.startup.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn search_autoeq_profiles(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<AutoEqProfileSummary>, String> {
    Ok(state.autoeq_db.search(&query))
}

#[tauri::command]
fn get_autoeq_profile(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<AutoEqProfile>, String> {
    Ok(state.autoeq_db.get_profile(&id).cloned())
}

#[tauri::command]
fn get_autoeq_stats(state: State<'_, AppState>) -> Result<AutoEqStats, String> {
    Ok(state.autoeq_db.stats())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new_runtime();

    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            // Restore window geometry if previously saved
            let initial_config = app.state::<AppState>().store.load_config().unwrap_or_default();
            if let Some(window) = app.get_webview_window("main") {
                if let (Some(x), Some(y)) = (initial_config.window_x, initial_config.window_y) {
                    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                }
            }

            // Load tray icon
            let icon = app
                .default_window_icon()
                .cloned()
                .unwrap_or_else(|| {
                    const DEFAULT_ICON: [u8; 16] = [
                        56, 189, 248, 255, 56, 189, 248, 255,
                        56, 189, 248, 255, 56, 189, 248, 255,
                    ];
                    Image::new(&DEFAULT_ICON, 2, 2)
                });

            // Enhanced native system tray context menu (FR28)
            let open_item = MenuItem::with_id(app, "open", "Open OpenBoom", true, None::<&str>)?;
            let power_item = MenuItem::with_id(app, "toggle_power", "Master Power", true, None::<&str>)?;
            let mute_item = MenuItem::with_id(app, "toggle_mute", "Mute", true, None::<&str>)?;

            let preset_acoustic = MenuItem::with_id(app, "preset_acoustic", "Harman Acoustic", true, None::<&str>)?;
            let preset_bass = MenuItem::with_id(app, "preset_bass", "Bass Boost", true, None::<&str>)?;
            let preset_vocal = MenuItem::with_id(app, "preset_vocal", "Vocal Clarity", true, None::<&str>)?;
            let presets_menu = Submenu::with_items(app, "Presets", true, &[&preset_acoustic, &preset_bass, &preset_vocal])?;

            let separator1 = PredefinedMenuItem::separator(app)?;
            let separator2 = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Exit OpenBoom", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &open_item,
                    &separator1,
                    &power_item,
                    &mute_item,
                    &presets_menu,
                    &separator2,
                    &quit_item,
                ],
            )?;

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("OpenBoom - Pro Audio DSP")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "open" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "toggle_power" => {
                            let state = app.state::<AppState>();
                            let current_power = if let Ok(mut coord) = state.coordinator.lock() {
                                coord.drain_commands();
                                coord.processor().master_power()
                            } else {
                                true
                            };
                            let next_power = !current_power;
                            let _ = state.dispatcher.send(AudioCommand::SetMasterPower(next_power));
                            let _ = app.emit("tray-power-toggled", next_power);
                        }
                        "toggle_mute" => {
                            let state = app.state::<AppState>();
                            let current_mute = if let Ok(mut coord) = state.coordinator.lock() {
                                coord.drain_commands();
                                coord.processor().is_muted()
                            } else {
                                false
                            };
                            let next_mute = !current_mute;
                            let _ = state.dispatcher.send(AudioCommand::SetMute(next_mute));
                            let _ = app.emit("tray-mute-toggled", next_mute);
                        }
                        "preset_acoustic" => {
                            let state = app.state::<AppState>();
                            let _ = state.dispatcher.send(AudioCommand::LoadGraphicEqSnapshot {
                                gains_db: [0.0; GRAPHIC_EQ_BANDS],
                                preamp_offset_db: 0.0,
                            });
                            let _ = app.emit("tray-preset-selected", "Harman Acoustic");
                        }
                        "preset_bass" => {
                            let state = app.state::<AppState>();
                            let mut gains = [0.0; GRAPHIC_EQ_BANDS];
                            for (i, gain) in gains.iter_mut().enumerate().take(6) {
                                *gain = 4.0 - (i as f32 * 0.5);
                            }
                            let _ = state.dispatcher.send(AudioCommand::LoadGraphicEqSnapshot {
                                gains_db: gains,
                                preamp_offset_db: -2.0,
                            });
                            let _ = app.emit("tray-preset-selected", "Bass Boost");
                        }
                        "preset_vocal" => {
                            let state = app.state::<AppState>();
                            let mut gains = [0.0; GRAPHIC_EQ_BANDS];
                            for gain in gains.iter_mut().take(20).skip(12) {
                                *gain = 2.5;
                            }
                            let _ = state.dispatcher.send(AudioCommand::LoadGraphicEqSnapshot {
                                gains_db: gains,
                                preamp_offset_db: -1.0,
                            });
                            let _ = app.emit("tray-preset-selected", "Vocal Clarity");
                        }
                        "quit" => {
                            let state = app.state::<AppState>();
                            state.restore_physical_default_device();
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Intercept close button: hide to tray instead of terminating application
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            dispatch_command,
            get_telemetry,
            get_audio_devices,
            set_audio_device,
            trigger_device_fallback,
            minimize_window,
            close_window,
            load_app_config,
            save_app_config,
            get_launch_on_startup,
            set_launch_on_startup,
            search_autoeq_profiles,
            get_autoeq_profile,
            get_autoeq_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenBoom desktop shell");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_config_geometry_and_theme() {
        let conf_str = include_str!("../tauri.conf.json");
        let v: serde_json::Value = serde_json::from_str(conf_str).expect("parse tauri.conf.json");
        let window = &v["app"]["windows"][0];

        assert_eq!(window["width"], 840, "Window width must be 840px");
        assert_eq!(window["height"], 560, "Window height must be 560px");
        assert_eq!(window["minWidth"], 780, "Window minWidth must be 780px");
        assert_eq!(window["minHeight"], 520, "Window minHeight must be 520px");
        assert_eq!(window["decorations"], false, "Window must be frameless");
        assert_eq!(window["backgroundColor"], "#121316", "Background must be obsidian canvas token");
    }

    #[test]
    fn test_telemetry_snapshot() {
        let state = AppState::new();
        let t = state.get_telemetry_snapshot();
        assert_eq!(t.limiter_gain_reduction_db, 0.0);
        assert!(t.round_trip_latency_ms < 15.0);
        assert!(t.dsp_active);
        assert_eq!(t.sample_rate, 48_000);
        assert_eq!(t.bit_depth, 32);
        assert_eq!(t.limiter_headroom_db, 20.0);
        let expected_bridge = state.coordinator.lock().unwrap().is_using_virtual_bridge();
        assert_eq!(t.using_virtual_bridge, expected_bridge);

        // Dispatch SetMasterPower(false)
        state
            .dispatcher
            .send(AudioCommand::SetMasterPower(false))
            .expect("send power off");

        // Allow worker thread to push to queue
        std::thread::sleep(std::time::Duration::from_millis(20));

        let t_off = state.get_telemetry_snapshot();
        assert!(!t_off.dsp_active, "DSP should be inactive after SetMasterPower(false)");

        // Dispatch SetMasterPower(true)
        state
            .dispatcher
            .send(AudioCommand::SetMasterPower(true))
            .expect("send power on");
        std::thread::sleep(std::time::Duration::from_millis(20));

        let t_on = state.get_telemetry_snapshot();
        assert!(t_on.dsp_active, "DSP should be active after SetMasterPower(true)");
    }

    #[test]
    fn test_dispatcher_and_spsc_queue() {
        let queue = SpscQueue::<AudioCommand, COMMAND_QUEUE_CAPACITY>::new();
        let dispatcher = UiDispatcher::new(queue.clone());

        dispatcher
            .send(AudioCommand::SetMasterVolume { volume: 0.8, boost: 1.25 })
            .expect("send command");

        // Allow background dispatcher thread to process the channel
        let mut popped = None;
        for _ in 0..20 {
            popped = queue.pop();
            if popped.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        assert!(popped.is_some(), "Command should be transferred to queue");
        match popped.unwrap() {
            AudioCommand::SetMasterVolume { volume, boost } => {
                assert!((volume - 0.8).abs() < 1e-6);
                assert!((boost - 1.25).abs() < 1e-6);
            }
            _ => panic!("Unexpected command popped"),
        }
    }

    #[test]
    fn test_audio_command_json_deserialization() {
        // Shapes produced by src-ui/src/App.tsx:
        // 1. SetMasterPower
        let cmd1: AudioCommand = serde_json::from_str(r#"{"SetMasterPower": true}"#).unwrap();
        assert_eq!(cmd1, AudioCommand::SetMasterPower(true));

        let cmd1_off: AudioCommand = serde_json::from_str(r#"{"SetMasterPower": false}"#).unwrap();
        assert_eq!(cmd1_off, AudioCommand::SetMasterPower(false));

        // 2. SetMasterVolume
        let cmd2: AudioCommand = serde_json::from_str(
            r#"{"SetMasterVolume": {"volume": 0.8, "boost": 1.25}}"#,
        )
        .unwrap();
        assert_eq!(
            cmd2,
            AudioCommand::SetMasterVolume {
                volume: 0.8,
                boost: 1.25
            }
        );

        // 3. SetMute
        let cmd3: AudioCommand = serde_json::from_str(r#"{"SetMute": true}"#).unwrap();
        assert_eq!(cmd3, AudioCommand::SetMute(true));

        // 4. SetSurround3D
        let cmd4: AudioCommand = serde_json::from_str(
            r#"{"SetSurround3D": {"enabled": true, "intensity": 0.72}}"#,
        )
        .unwrap();
        assert_eq!(
            cmd4,
            AudioCommand::SetSurround3D {
                enabled: true,
                intensity: 0.72
            }
        );

        // 5. SetSweetener
        let cmd5: AudioCommand = serde_json::from_str(
            r#"{"SetSweetener": {"module_id": 1, "enabled": true, "amount": 0.45}}"#,
        )
        .unwrap();
        assert_eq!(
            cmd5,
            AudioCommand::SetSweetener {
                module_id: 1,
                enabled: true,
                amount: 0.45
            }
        );

        // 6. SetBypass
        let cmd6: AudioCommand = serde_json::from_str(r#"{"SetBypass": true}"#).unwrap();
        assert_eq!(cmd6, AudioCommand::SetBypass(true));

        // 7. SetEqBandGain
        let cmd7: AudioCommand = serde_json::from_str(
            r#"{"SetEqBandGain": {"band_idx": 17, "gain_db": 6.0}}"#,
        )
        .unwrap();
        assert_eq!(
            cmd7,
            AudioCommand::SetEqBandGain {
                band_idx: 17,
                gain_db: 6.0
            }
        );

        // 8. SetEqBypass
        let cmd8: AudioCommand = serde_json::from_str(r#"{"SetEqBypass": {"bypassed": true}}"#).unwrap();
        assert_eq!(cmd8, AudioCommand::SetEqBypass { bypassed: true });

        // 9. LoadGraphicEqSnapshot
        let snapshot_json = format!(
            r#"{{"LoadGraphicEqSnapshot": {{"gains_db": {:?}, "preamp_offset_db": -3.0}}}}"#,
            [0.0f32; 31]
        );
        let cmd9: AudioCommand = serde_json::from_str(&snapshot_json).unwrap();
        assert_eq!(
            cmd9,
            AudioCommand::LoadGraphicEqSnapshot {
                gains_db: [0.0; 31],
                preamp_offset_db: -3.0
            }
        );
    }

    #[test]
    fn test_device_info_serialization() {
        let dev = AudioDeviceInfo::new("dev_123", "Headphones (Test)", true);
        let serialized = serde_json::to_string(&dev).expect("serialize AudioDeviceInfo");
        let deserialized: AudioDeviceInfo =
            serde_json::from_str(&serialized).expect("deserialize AudioDeviceInfo");
        assert_eq!(dev, deserialized);
        assert_eq!(deserialized.id, "dev_123");
        assert_eq!(deserialized.name, "Headphones (Test)");
        assert!(deserialized.is_default);
    }

    #[test]
    fn test_coordinator_device_switching_and_fallback() {
        let state = AppState::new();
        let mut coord = state.coordinator.lock().expect("lock coordinator");

        // Enumerate devices (synthetic or hardware)
        let devices = coord.enumerate_devices().expect("enumerate devices");
        assert!(!devices.is_empty(), "Devices list must not be empty");

        // Switch to explicit synthetic or specific device
        let target_id = devices[0].id.clone();
        let switch_res = coord.switch_device(Some(target_id));
        assert!(switch_res.is_ok(), "Switch device must succeed");

        // Handle device lost recovery
        let fallback_res = coord.handle_device_lost();
        assert!(fallback_res.is_ok(), "Fallback recovery must succeed");
        assert_eq!(coord.current_device_id(), None, "Current device must reset to None (system default)");
    }

    #[test]
    fn test_app_state_storage_and_startup() {
        let temp_dir = std::env::temp_dir().join(format!("tauri_state_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let store = PresetStore::new(&temp_dir);
        let startup = StartupManager::mock();
        let state = AppState::with_store_and_startup(store, startup);

        // Verify config load & save
        let initial_cfg = state.store.load_config().expect("load config");
        assert_eq!(initial_cfg, AppConfig::default());

        let mut mod_cfg = initial_cfg;
        mod_cfg.master_volume = 1.2;
        mod_cfg.active_output_device = Some("dev_test".to_string());
        mod_cfg.launch_on_startup = true;
        state.store.save_config(&mod_cfg).expect("save config");

        let reloaded = state.store.load_config().expect("reload config");
        assert_eq!(reloaded, mod_cfg);

        // Verify startup manager
        assert!(!state.startup.is_enabled().expect("initially disabled"));
        state.startup.enable("test_exe.exe").expect("enable");
        assert!(state.startup.is_enabled().expect("enabled"));
        state.startup.disable().expect("disable");
        assert!(!state.startup.is_enabled().expect("disabled"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_autoeq_commands_integration() {
        let state = AppState::new();

        // 1. Stats command
        let stats = state.autoeq_db.stats();
        assert!(stats.total_profiles >= 4000);
        assert!(stats.total_brands >= 30);

        // 2. Empty search -> top recommendations
        let popular = state.autoeq_db.search("");
        assert!(!popular.is_empty());
        assert!(popular.iter().any(|p| p.brand == "HyperX"));

        // 3. Search query
        let query_res = state.autoeq_db.search("HyperX Cloud II");
        assert!(!query_res.is_empty());
        let top = &query_res[0];
        assert_eq!(top.brand, "HyperX");
        assert_eq!(top.score, 100);

        // 4. Get profile details
        let profile = state.autoeq_db.get_profile(&top.id).expect("profile exists");
        assert_eq!(profile.name, "HyperX Cloud II");
        assert!(!profile.filters.is_empty());
    }
}


