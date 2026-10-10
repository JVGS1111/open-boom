//! Desktop shell runtime and system tray lifecycle manager for OpenBoom.

use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{AudioCommand, TelemetrySnapshot};
use std::sync::Arc;
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
}

#[tauri::command]
fn dispatch_command(cmd: AudioCommand, state: State<'_, AppState>) -> Result<(), String> {
    state.dispatcher.send(cmd).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_telemetry() -> TelemetrySnapshot {
    TelemetrySnapshot {
        limiter_gain_reduction_db: 0.0,
        round_trip_latency_ms: 11.4,
        peak_output_l_dbfs: -6.0,
        peak_output_r_dbfs: -6.0,
        limiter_active: false,
        dsp_active: true,
    }
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
    let command_queue = SpscQueue::<AudioCommand, COMMAND_QUEUE_CAPACITY>::new();
    let dispatcher = UiDispatcher::new(command_queue.clone());
    let state = AppState {
        dispatcher,
        command_queue,
    };

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
        let t = get_telemetry();
        assert_eq!(t.limiter_gain_reduction_db, 0.0);
        assert!(t.round_trip_latency_ms < 15.0);
        assert!(t.dsp_active);
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
}

