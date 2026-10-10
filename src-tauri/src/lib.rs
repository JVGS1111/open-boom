//! Desktop shell runtime and system tray lifecycle manager for OpenBoom.

use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{AudioCommand, AudioDeviceInfo, TelemetrySnapshot};
use open_boom_wasapi::{AudioEngineCoordinator, LoopbackConfig, RenderConfig};
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State, WindowEvent,
};

pub mod dispatcher;
use dispatcher::{UiDispatcher, COMMAND_QUEUE_CAPACITY};

pub struct AppState {
    pub dispatcher: UiDispatcher,
    pub command_queue: Arc<SpscQueue<AudioCommand, COMMAND_QUEUE_CAPACITY>>,
    pub coordinator: Arc<Mutex<AudioEngineCoordinator>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let command_queue = SpscQueue::<AudioCommand, COMMAND_QUEUE_CAPACITY>::new();
        let dispatcher = UiDispatcher::new(command_queue.clone());
        let mut render_config = RenderConfig::default();
        render_config.allow_synthetic_fallback = true;
        let coordinator = Arc::new(Mutex::new(AudioEngineCoordinator::with_command_queue(
            LoopbackConfig::default(),
            render_config,
            command_queue.clone(),
        )));
        Self {
            dispatcher,
            command_queue,
            coordinator,
        }
    }

    pub fn get_telemetry_snapshot(&self) -> TelemetrySnapshot {
        if let Ok(mut coord) = self.coordinator.lock() {
            coord.drain_commands();
            let mut snapshot = coord.telemetry_snapshot();
            if !coord.is_running() {
                snapshot.dsp_active = coord.processor().is_dsp_active();
            }
            snapshot
        } else {
            TelemetrySnapshot::default()
        }
    }
}

#[tauri::command]
fn dispatch_command(cmd: AudioCommand, state: State<'_, AppState>) -> Result<(), String> {
    state.dispatcher.send(cmd).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_telemetry(state: State<'_, AppState>) -> TelemetrySnapshot {
    state.get_telemetry_snapshot()
}

#[tauri::command]
fn get_audio_devices(state: State<'_, AppState>) -> Result<Vec<AudioDeviceInfo>, String> {
    let coord = state.coordinator.lock().map_err(|e| e.to_string())?;
    coord.enumerate_devices().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_audio_device(device_id: Option<String>, state: State<'_, AppState>) -> Result<(), String> {
    let mut coord = state.coordinator.lock().map_err(|e| e.to_string())?;
    coord.switch_device(device_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn trigger_device_fallback(state: State<'_, AppState>) -> Result<(), String> {
    let mut coord = state.coordinator.lock().map_err(|e| e.to_string())?;
    coord.handle_device_lost().map_err(|e| e.to_string())
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new();

    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
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

            // Create tray context menu allowing the user to restore or quit
            let open_item = MenuItem::with_id(app, "open", "Open OpenBoom", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit OpenBoom", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

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
                        "quit" => {
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
}

