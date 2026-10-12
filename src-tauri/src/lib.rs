//! Desktop shell runtime and system tray lifecycle manager for OpenBoom.

use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{
    AudioCommand, AudioDeviceInfo, OrbitDirection, ProcessAudioSession, TelemetrySnapshot, GRAPHIC_EQ_BANDS,
};
use open_boom_storage::{
    sanitize_audio_bridge_mode, AppConfig, AppVolumeProfile, AutoEqDatabase, AutoEqProfile,
    AutoEqProfileSummary, AutoEqStats, EqPreset, PresetStore, StartupManager,
};
use open_boom_wasapi::{
    AudioDeviceManager, AudioEngineCoordinator, AudioStreamPump, BridgeMode, EndpointVolumeSync,
    LoopbackConfig, RenderConfig, SessionManager, StreamPumpConfig,
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

/// Maps a persisted audio bridge mode identifier (`"auto"`, `"virtual"`, `"loopback"`)
/// to an optional forced `BridgeMode`.
pub fn parse_bridge_force_mode(mode: &str) -> Option<BridgeMode> {
    match mode.trim().to_lowercase().as_str() {
        "virtual" => Some(BridgeMode::VirtualSinkModeA),
        "loopback" => Some(BridgeMode::LoopbackFallbackModeB),
        _ => None,
    }
}

/// Payload delivered across the Tauri IPC boundary on Windows volume change events.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowsVolumeChangedPayload {
    pub volume: f32,
    pub is_muted: bool,
}

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
    pub audio_bridge_mode: Arc<Mutex<String>>,
    pub auto_route: bool,
    pub session_manager: Arc<Mutex<SessionManager>>,
    pub volume_sync: Arc<EndpointVolumeSync>,
    pub staged_update_installer: Arc<Mutex<Option<std::path::PathBuf>>>,
    pub is_downloading_update: Arc<std::sync::atomic::AtomicBool>,
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
        Self::with_options(PresetStore::default(), StartupManager::new(), auto_route)
    }

    pub fn with_store_and_startup(store: PresetStore, startup: StartupManager) -> Self {
        Self::with_options(store, startup, false)
    }

    pub fn with_options(store: PresetStore, startup: StartupManager, auto_route: bool) -> Self {
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
        let app_config = store.load_config().unwrap_or_default();
        let buffer_guard = app_config.buffer_guard_enabled;
        let bridge_mode = sanitize_audio_bridge_mode(&app_config.audio_bridge_mode);
        let force_loopback = bridge_mode == "loopback";
        let dsp_initially_active = app_config.master_power && !app_config.bypass_active;

        if let Ok(mut coord) = coordinator.lock() {
            coord.set_buffer_guard(buffer_guard);
            let _ = coord.set_bridge_force_mode(parse_bridge_force_mode(&bridge_mode));
        }

        let device_mgr = AudioDeviceManager::new();
        let physical_default = device_mgr.get_physical_default_device().ok().flatten();
        let physical_id = app_config
            .active_output_device
            .clone()
            .filter(|id| !id.trim().is_empty() && id != "default")
            .or_else(|| physical_default.as_ref().map(|d| d.id.clone()));
        let virtual_dev = device_mgr.find_virtual_device().ok().flatten();
        let virtual_id = virtual_dev.as_ref().map(|d| d.id.clone());

        let is_routed_to_virtual = Arc::new(Mutex::new(false));

        // Configure Windows default endpoint routing BEFORE starting AudioStreamPump
        // so endpoint resolution does not race with IPolicyConfig::SetDefaultEndpoint.
        if auto_route {
            if force_loopback || !dsp_initially_active {
                if let Some(ref p_id) = physical_id {
                    let _ = device_mgr.set_default_device(p_id);
                }
            } else if let Some(ref v_id) = virtual_id {
                if device_mgr.set_default_device(v_id).is_ok() {
                    if let Ok(mut routed) = is_routed_to_virtual.lock() {
                        *routed = true;
                    }
                }
            }
        }

        let pump_instance = AudioStreamPump::start(
            coordinator.clone(),
            StreamPumpConfig {
                target_device_id: physical_id.clone(),
                capture_device_id: if force_loopback { None } else { virtual_id.clone() },
                allow_synthetic: !auto_route,
                buffer_guard_enabled: buffer_guard,
                force_loopback,
            },
        ).ok();
        let pump = Arc::new(Mutex::new(pump_instance));

        let autoeq_db = Arc::new(AutoEqDatabase::default());
        let mut session_mgr_instance = SessionManager::with_synthetic_fallback(true);
        if let Ok(profiles) = store.load_app_volumes() {
            session_mgr_instance.register_stored_profiles(profiles);
        }
        let session_manager = Arc::new(Mutex::new(session_mgr_instance));

        let volume_sync = Arc::new(EndpointVolumeSync::with_synthetic_fallback(true));
        let dispatcher_for_vol = dispatcher.clone();
        volume_sync.add_on_volume_changed(move |event| {
            let v = event.volume;
            let linear_vol = if v <= 1.0 { v * v } else { 1.0 };
            let boost = if v > 1.0 { v } else { 1.0 };
            let _ = dispatcher_for_vol.send(AudioCommand::SetMasterVolume {
                volume: linear_vol,
                boost,
            });
            let _ = dispatcher_for_vol.send(AudioCommand::SetMute(event.is_muted));
        });

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
            audio_bridge_mode: Arc::new(Mutex::new(bridge_mode)),
            auto_route,
            session_manager,
            volume_sync,
            staged_update_installer: Arc::new(Mutex::new(None)),
            is_downloading_update: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn is_loopback_forced(&self) -> bool {
        self.audio_bridge_mode
            .lock()
            .ok()
            .map(|m| m.as_str() == "loopback")
            .unwrap_or(false)
    }

    pub fn handle_master_power_toggle(&self, power: bool) -> Result<(), String> {
        let coord = self.coordinator.lock().map_err(|e| e.to_string())?;
        let device_mgr = coord.device_manager().clone();
        drop(coord);

        let physical_id = self.active_physical_device_id.lock().ok().and_then(|g| g.clone());
        let virtual_id = self.virtual_device_id.lock().ok().and_then(|g| g.clone());
        let force_loopback = self.is_loopback_forced();

        if power && !force_loopback {
            if let Some(ref v_id) = virtual_id {
                let ok = if self.auto_route {
                    device_mgr.set_default_device(v_id).is_ok()
                } else {
                    true
                };
                if ok {
                    if let Ok(mut routed) = self.is_routed_to_virtual.lock() {
                        *routed = true;
                    }
                }
            }
        } else if let Some(ref p_id) = physical_id {
            let ok = if self.auto_route {
                device_mgr.set_default_device(p_id).is_ok()
            } else {
                true
            };
            if ok {
                if let Ok(mut routed) = self.is_routed_to_virtual.lock() {
                    *routed = false;
                }
            }
        }
        let _ = self.volume_sync.rebind_endpoint(None);
        Ok(())
    }

    pub fn apply_audio_bridge_mode(&self, mode: &str) -> Result<String, String> {
        let sanitized = sanitize_audio_bridge_mode(mode);
        let force_mode = parse_bridge_force_mode(&sanitized);
        let force_loopback = sanitized == "loopback";

        // Persist in AppConfig atomically first so failed disk writes surface immediately
        let mut config = self.store.load_config().unwrap_or_default();
        config.audio_bridge_mode = sanitized.clone();
        let buffer_guard = config.buffer_guard_enabled;
        self.store.save_config(&config).map_err(|e| e.to_string())?;

        if let Ok(mut guard) = self.audio_bridge_mode.lock() {
            *guard = sanitized.clone();
        }

        let (device_mgr, dsp_active) = {
            let mut coord = self.coordinator.lock().map_err(|e| e.to_string())?;
            coord.drain_commands();
            let _ = coord.set_bridge_force_mode(force_mode);
            (coord.device_manager().clone(), coord.processor().is_dsp_active())
        };

        let physical_id = self.active_physical_device_id.lock().ok().and_then(|g| g.clone());
        let virtual_id = self.virtual_device_id.lock().ok().and_then(|g| g.clone());

        // Adjust Windows endpoint routing before restarting stream pump
        if force_loopback || !dsp_active {
            if let Some(ref p_id) = physical_id {
                let ok = if self.auto_route {
                    device_mgr.set_default_device(p_id).is_ok()
                } else {
                    true
                };
                if ok {
                    if let Ok(mut routed) = self.is_routed_to_virtual.lock() {
                        *routed = false;
                    }
                }
            }
        } else if let Some(ref v_id) = virtual_id {
            let ok = if self.auto_route {
                device_mgr.set_default_device(v_id).is_ok()
            } else {
                true
            };
            if ok {
                if let Ok(mut routed) = self.is_routed_to_virtual.lock() {
                    *routed = true;
                }
            }
        }

        // Hot-restart pump with updated bridge configuration
        if let Ok(mut pump_guard) = self.pump.lock() {
            if let Some(mut existing) = pump_guard.take() {
                existing.stop();
            }
            *pump_guard = AudioStreamPump::start(
                self.coordinator.clone(),
                StreamPumpConfig {
                    target_device_id: physical_id,
                    capture_device_id: if force_loopback { None } else { virtual_id },
                    allow_synthetic: !self.auto_route,
                    buffer_guard_enabled: buffer_guard,
                    force_loopback,
                },
            ).ok();
        }

        let _ = self.volume_sync.rebind_endpoint(None);

        Ok(sanitized)
    }

    pub fn dispatch_audio_command(&self, cmd: AudioCommand) -> Result<(), String> {
        match &cmd {
            AudioCommand::SetMasterPower(power) => {
                let _ = self.handle_master_power_toggle(*power);
            }
            AudioCommand::SetBypass(bypassed) => {
                let _ = self.handle_master_power_toggle(!*bypassed);
            }
            AudioCommand::SetMasterVolume { volume, boost } => {
                let slider_vol = if *boost > 1.0 {
                    *boost
                } else {
                    volume.max(0.0).sqrt()
                };
                let _ = self.volume_sync.set_master_volume(slider_vol);
            }
            AudioCommand::SetMute(muted) => {
                let _ = self.volume_sync.set_mute(*muted);
            }
            _ => {}
        }
        self.dispatcher.send(cmd).map_err(|e| e.to_string())
    }

    pub fn restore_physical_default_device(&self) {
        let was_routed = self.is_routed_to_virtual.lock().ok().map(|g| *g).unwrap_or(false);
        if was_routed || self.is_loopback_forced() {
            let physical_id = self.active_physical_device_id.lock().ok().and_then(|g| g.clone());
            if let Some(ref p_id) = physical_id {
                let ok = if self.auto_route {
                    let device_mgr = AudioDeviceManager::new();
                    device_mgr.set_default_device(p_id).is_ok()
                } else {
                    true
                };
                if ok {
                    if let Ok(mut routed) = self.is_routed_to_virtual.lock() {
                        *routed = false;
                    }
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
    state.dispatch_audio_command(cmd)
}

#[tauri::command]
fn set_master_volume(volume: f32, state: State<'_, AppState>) -> Result<(), String> {
    let clamped = if volume.is_nan() { 0.0 } else { volume.clamp(0.0, 1.5) };
    let linear_vol = if clamped <= 1.0 { clamped * clamped } else { 1.0 };
    let boost = if clamped > 1.0 { clamped } else { 1.0 };
    state.dispatch_audio_command(AudioCommand::SetMasterVolume {
        volume: linear_vol,
        boost,
    })
}

#[tauri::command]
fn set_mute(muted: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.dispatch_audio_command(AudioCommand::SetMute(muted))
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
    let force_loopback = state.is_loopback_forced();
    let buffer_guard = state.store.load_config().map(|c| c.buffer_guard_enabled).unwrap_or(true);

    if let Ok(mut pump_guard) = state.pump.lock() {
        if let Some(mut existing) = pump_guard.take() {
            existing.stop();
        }
        *pump_guard = AudioStreamPump::start(
            state.coordinator.clone(),
            StreamPumpConfig {
                target_device_id: device_id.clone(),
                capture_device_id: if force_loopback { None } else { virtual_id },
                allow_synthetic: !state.auto_route,
                buffer_guard_enabled: buffer_guard,
                force_loopback,
            },
        ).ok();
    }

    // If master power is currently OFF or in loopback mode, route Windows default directly to the physical device
    let power = state.coordinator.lock().map(|c| c.processor().master_power()).unwrap_or(true);
    if !power || force_loopback {
        if let Some(ref id) = device_id {
            let _ = device_mgr.set_default_device(id);
        }
    }

    let _ = state.volume_sync.rebind_endpoint(device_id);

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
    let force_loopback = state.is_loopback_forced();
    let buffer_guard = state.store.load_config().map(|c| c.buffer_guard_enabled).unwrap_or(true);

    if let Ok(mut pump_guard) = state.pump.lock() {
        if let Some(mut existing) = pump_guard.take() {
            existing.stop();
        }
        *pump_guard = AudioStreamPump::start(
            state.coordinator.clone(),
            StreamPumpConfig {
                target_device_id: fallback_id.clone(),
                capture_device_id: if force_loopback { None } else { virtual_id },
                allow_synthetic: !state.auto_route,
                buffer_guard_enabled: buffer_guard,
                force_loopback,
            },
        ).ok();
    }

    let _ = state.volume_sync.rebind_endpoint(fallback_id);

    Ok(())
}

#[tauri::command]
fn set_buffer_guard(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    // 1. Dispatch lock-free command to DSP limiter
    state.dispatcher.send(AudioCommand::SetBufferGuard(enabled)).map_err(|e| e.to_string())?;

    // 2. Direct update on coordinator processor if accessible
    if let Ok(mut coord) = state.coordinator.lock() {
        coord.set_buffer_guard(enabled);
    }

    // 3. Persist in AppConfig
    let mut config = state.store.load_config().unwrap_or_default();
    config.buffer_guard_enabled = enabled;
    let _ = state.store.save_config(&config);

    // 4. Hot-restart pump with new buffer size
    let physical_id = state.active_physical_device_id.lock().ok().and_then(|g| g.clone());
    let virtual_id = state.virtual_device_id.lock().ok().and_then(|g| g.clone());
    let force_loopback = state.is_loopback_forced();

    if let Ok(mut pump_guard) = state.pump.lock() {
        if let Some(mut existing) = pump_guard.take() {
            existing.stop();
        }
        *pump_guard = AudioStreamPump::start(
            state.coordinator.clone(),
            StreamPumpConfig {
                target_device_id: physical_id,
                capture_device_id: if force_loopback { None } else { virtual_id },
                allow_synthetic: !state.auto_route,
                buffer_guard_enabled: enabled,
                force_loopback,
            },
        ).ok();
    }

    Ok(())
}

#[tauri::command]
fn set_audio_bridge_mode(mode: String, state: State<'_, AppState>) -> Result<String, String> {
    state.apply_audio_bridge_mode(&mode)
}

#[tauri::command]
fn get_audio_bridge_mode(state: State<'_, AppState>) -> String {
    state
        .audio_bridge_mode
        .lock()
        .ok()
        .map(|m| m.clone())
        .unwrap_or_else(|| "auto".to_string())
}

fn resolve_orbit_params(
    direction: Option<OrbitDirection>,
    period_sec: Option<f32>,
    depth: Option<f32>,
) -> (OrbitDirection, f32, f32) {
    (
        direction.unwrap_or(OrbitDirection::Clockwise),
        period_sec.unwrap_or(12.0),
        depth.unwrap_or(0.85),
    )
}

#[tauri::command]
fn set_orbit_8d(
    enabled: bool,
    direction: Option<OrbitDirection>,
    period_sec: Option<f32>,
    depth: Option<f32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (dir, period, d) = resolve_orbit_params(direction, period_sec, depth);

    state.dispatcher.send(AudioCommand::SetOrbit8D {
        enabled,
        direction: dir,
        period_sec: period,
        depth: d,
    }).map_err(|e| e.to_string())
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
fn save_app_config(mut config: AppConfig, state: State<'_, AppState>) -> Result<(), String> {
    if let Ok(guard) = state.audio_bridge_mode.lock() {
        config.audio_bridge_mode = sanitize_audio_bridge_mode(&guard);
    } else {
        config.audio_bridge_mode = sanitize_audio_bridge_mode(&config.audio_bridge_mode);
    }
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

#[tauri::command]
fn apply_autoeq_profile(
    profile_id: String,
    preamp_offset_db: Option<f32>,
    state: State<'_, AppState>,
) -> Result<f32, String> {
    let profile = state
        .autoeq_db
        .get_profile(&profile_id)
        .ok_or_else(|| format!("AutoEq profile with id '{}' not found", profile_id))?;

    let max_positive_gain = profile
        .filters
        .iter()
        .map(|f| if f.gain_db.is_nan() { 0.0 } else { f.gain_db })
        .fold(0.0f32, f32::max);

    let auto_offset_db = if max_positive_gain > 0.0 {
        -max_positive_gain
    } else {
        0.0
    };

    let offset = if let Some(user_offset) = preamp_offset_db {
        if user_offset.is_finite() && user_offset < auto_offset_db {
            user_offset
        } else {
            auto_offset_db
        }
    } else {
        auto_offset_db
    };

    let mut pod_filters = [open_boom_core::types::ParametricFilterPod::default(); open_boom_core::types::MAX_PARAMETRIC_FILTERS];
    let count = profile.filters.len().min(open_boom_core::types::MAX_PARAMETRIC_FILTERS);

    for i in 0..count {
        let f = &profile.filters[i];
        let filter_type = match f.filter_type.to_lowercase().as_str() {
            "lowshelf" | "low_shelf" | "ls" => open_boom_core::types::ParametricFilterType::LowShelf,
            "highshelf" | "high_shelf" | "hs" => open_boom_core::types::ParametricFilterType::HighShelf,
            _ => open_boom_core::types::ParametricFilterType::Peaking,
        };
        pod_filters[i] = open_boom_core::types::ParametricFilterPod {
            filter_type,
            frequency_hz: f.frequency_hz,
            q: f.q,
            gain_db: f.gain_db,
        };
    }

    state.dispatcher.apply_autoeq_profile(pod_filters, count, offset).map_err(|e| e.to_string())?;

    Ok(offset)
}

#[tauri::command]
fn clear_autoeq_profile(state: State<'_, AppState>) -> Result<(), String> {
    state.dispatcher.clear_autoeq_profile().map_err(|e| e.to_string())?;
    let empty_filters = [open_boom_core::types::ParametricFilterPod::default(); open_boom_core::types::MAX_PARAMETRIC_FILTERS];
    state.dispatcher.apply_autoeq_profile(empty_filters, 0, 0.0).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_eq_presets(state: State<'_, AppState>) -> Result<Vec<EqPreset>, String> {
    Ok(state.store.list_presets())
}

#[tauri::command]
fn save_custom_eq_preset(
    name: String,
    gains_db: Vec<f32>,
    state: State<'_, AppState>,
) -> Result<EqPreset, String> {
    if gains_db.len() != GRAPHIC_EQ_BANDS {
        return Err(format!(
            "Expected {} EQ band gains, received {}",
            GRAPHIC_EQ_BANDS,
            gains_db.len()
        ));
    }
    let mut array = [0.0f32; GRAPHIC_EQ_BANDS];
    array.copy_from_slice(&gains_db);
    state
        .store
        .save_custom_preset(&name, array)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_custom_eq_preset(
    old_id: String,
    new_name: String,
    state: State<'_, AppState>,
) -> Result<EqPreset, String> {
    state
        .store
        .rename_custom_preset(&old_id, &new_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_custom_eq_preset(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .store
        .delete_custom_preset(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_audio_sessions(state: State<'_, AppState>) -> Result<Vec<ProcessAudioSession>, String> {
    let mut mgr = state.session_manager.lock().map_err(|e| e.to_string())?;
    mgr.enumerate_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_app_volume(process_id: u32, volume: f32, state: State<'_, AppState>) -> Result<(), String> {
    let mut mgr = state.session_manager.lock().map_err(|e| e.to_string())?;
    mgr.set_process_volume(process_id, volume).map_err(|e| e.to_string())?;
    if let Some(name) = mgr.get_process_name(process_id) {
        if let Some(profile) = mgr.get_stored_profile(&name).cloned() {
            drop(mgr);
            let _ = state.store.save_app_volume_profile(&profile);
            return Ok(());
        }
    }
    Ok(())
}

#[tauri::command]
fn set_app_mute(process_id: u32, mute: bool, state: State<'_, AppState>) -> Result<(), String> {
    let mut mgr = state.session_manager.lock().map_err(|e| e.to_string())?;
    mgr.set_process_mute(process_id, mute).map_err(|e| e.to_string())?;
    if let Some(name) = mgr.get_process_name(process_id) {
        if let Some(profile) = mgr.get_stored_profile(&name).cloned() {
            drop(mgr);
            let _ = state.store.save_app_volume_profile(&profile);
            return Ok(());
        }
    }
    Ok(())
}

#[tauri::command]
fn get_app_volume_profiles(state: State<'_, AppState>) -> Result<Vec<AppVolumeProfile>, String> {
    state.store.load_app_volume_list().map_err(|e| e.to_string())
}

#[tauri::command]
fn reset_app_volume_profile(process_name: String, state: State<'_, AppState>) -> Result<bool, String> {
    let mut mgr = state.session_manager.lock().map_err(|e| e.to_string())?;
    let removed_from_mgr = mgr.remove_stored_profile(&process_name);
    drop(mgr);
    let removed_from_store = state
        .store
        .delete_app_volume_profile(&process_name)
        .map_err(|e| e.to_string())?;
    Ok(removed_from_mgr || removed_from_store)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateCheckResult {
    pub is_available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub html_url: Option<String>,
    pub download_url: Option<String>,
}

/// Selects the appropriate installer asset URL from a GitHub Release `assets` array.
/// Strictly prioritizes NSIS setup/installer packages (`*-setup.exe`, `*installer*.exe`)
/// and excludes standalone portable binaries (`*portable*.exe`) so the update routine
/// never accidentally launches a portable binary from `%TEMP%` instead of running the setup installer.
pub fn select_installer_asset_url(assets: &[serde_json::Value]) -> Option<String> {
    // Pass 1: Explicit setup/installer .exe (excluding portable)
    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            let lower = name.to_lowercase();
            if lower.ends_with(".exe")
                && !lower.contains("portable")
                && (lower.contains("setup") || lower.contains("installer"))
            {
                if let Some(url) = asset["browser_download_url"].as_str() {
                    return Some(url.to_string());
                }
            }
        }
    }

    // Pass 2: Any non-portable .exe
    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            let lower = name.to_lowercase();
            if lower.ends_with(".exe") && !lower.contains("portable") {
                if let Some(url) = asset["browser_download_url"].as_str() {
                    return Some(url.to_string());
                }
            }
        }
    }

    // Pass 3: Last-resort fallback to any .exe
    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            let lower = name.to_lowercase();
            if lower.ends_with(".exe") {
                if let Some(url) = asset["browser_download_url"].as_str() {
                    return Some(url.to_string());
                }
            }
        }
    }

    None
}

#[tauri::command]
async fn check_for_updates(app: tauri::AppHandle) -> Result<UpdateCheckResult, String> {
    let current_version = app.package_info().version.to_string();

    tauri::async_runtime::spawn_blocking(move || {
        let ps_cmd = r#"
            $ProgressPreference = 'SilentlyContinue'
            $ErrorActionPreference = 'Stop'
            try {
                [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
                $headers = @{ "User-Agent" = "OpenBoom-Desktop-App" }
                $res = Invoke-RestMethod -Uri "https://api.github.com/repos/JVGS1111/open-boom/releases/latest" -Headers $headers -TimeoutSec 10
                $res | ConvertTo-Json -Depth 4
            } catch {
                [Console]::Error.WriteLine($_.Exception.Message)
                exit 1
            }
        "#;

        let mut cmd = std::process::Command::new("powershell");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", ps_cmd]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let output = cmd.output().map_err(|e| format!("Failed to query GitHub update API: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Update check failed: {}", err.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse release metadata: {}", e))?;

        let tag_name = json["tag_name"]
            .as_str()
            .unwrap_or("")
            .trim_start_matches('v')
            .to_string();

        let latest_version = if tag_name.is_empty() {
            current_version.clone()
        } else {
            tag_name
        };

        let is_available = is_version_newer(&latest_version, &current_version);
        let release_notes = json["body"].as_str().map(|s| s.to_string());
        let published_at = json["published_at"].as_str().map(|s| s.to_string());
        let html_url = json["html_url"].as_str().map(|s| s.to_string());

        let mut download_url = json["assets"]
            .as_array()
            .and_then(|assets| select_installer_asset_url(assets));

        if download_url.is_none() {
            download_url = html_url.clone();
        }

        Ok(UpdateCheckResult {
            is_available,
            current_version,
            latest_version,
            release_notes,
            published_at,
            html_url,
            download_url,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Helper to semantically compare two version strings (e.g., "0.4.0" > "0.3.0").
pub fn is_version_newer(latest: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> (Vec<u32>, bool) {
        let is_prerelease = v.contains('-');
        let main_part = v.split('-').next().unwrap_or(v);
        let parts = main_part
            .split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect();
        (parts, is_prerelease)
    };

    let (latest_parts, latest_pre) = parse_version(latest);
    let (current_parts, current_pre) = parse_version(current);

    let max_len = latest_parts.len().max(current_parts.len());
    for i in 0..max_len {
        let l = latest_parts.get(i).copied().unwrap_or(0);
        let c = current_parts.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    // If main numbers are equal, stable release is newer than pre-release of the same version
    current_pre && !latest_pre
}

struct DownloadInFlightGuard {
    flag: Arc<std::sync::atomic::AtomicBool>,
}

impl Drop for DownloadInFlightGuard {
    fn drop(&mut self) {
        self.flag.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

#[tauri::command]
async fn download_and_install_update(
    download_url: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !download_url.starts_with("https://github.com/")
        && !download_url.starts_with("https://objects.githubusercontent.com/")
    {
        return Err("Update URL must originate from GitHub".to_string());
    }
    if !download_url.to_lowercase().ends_with(".exe") {
        return Err("Update package does not contain an executable installer".to_string());
    }

    if state
        .is_downloading_update
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_err()
    {
        return Err("An update download is already in progress".to_string());
    }

    let flag = state.is_downloading_update.clone();
    let staged_slot = state.staged_update_installer.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let _guard = DownloadInFlightGuard { flag };

        let dest_path = std::env::temp_dir().join("OpenBoom_Setup_Update.exe");
        let part_path = std::env::temp_dir().join("OpenBoom_Setup_Update.exe.part");

        let mut cmd = std::process::Command::new("powershell");
        let ps_cmd = r#"
            $ProgressPreference = 'SilentlyContinue'
            $ErrorActionPreference = 'Stop'
            $url = $env:OPENBOOM_UPDATE_URL
            $dest = $env:OPENBOOM_UPDATE_DEST
            $part = $env:OPENBOOM_UPDATE_PART
            try {
                if (Test-Path $part) { Remove-Item $part -Force -ErrorAction SilentlyContinue }
                $usedCurl = $false
                $curlExe = Join-Path $env:SystemRoot "System32\curl.exe"
                if (Test-Path $curlExe) {
                    & $curlExe -fL -sS --retry 2 --connect-timeout 15 --max-time 90 -o $part $url
                    if ($LASTEXITCODE -eq 0 -and (Test-Path $part) -and ((Get-Item $part).Length -gt 32768)) {
                        $usedCurl = $true
                    }
                }
                if (-not $usedCurl) {
                    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
                    Invoke-WebRequest -Uri $url -OutFile $part -UseBasicParsing -TimeoutSec 90
                }
                if (-not (Test-Path $part) -or ((Get-Item $part).Length -le 32768)) {
                    throw "Downloaded update package is incomplete or empty."
                }
                Move-Item -Path $part -Destination $dest -Force
            } catch {
                [Console]::Error.WriteLine($_.Exception.Message)
                exit 1
            }
        "#;

        cmd.env("OPENBOOM_UPDATE_URL", &download_url)
            .env("OPENBOOM_UPDATE_DEST", &dest_path)
            .env("OPENBOOM_UPDATE_PART", &part_path)
            .args(["-NoProfile", "-NonInteractive", "-Command", ps_cmd]);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to initiate update download: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Update download error: {}", err.trim()));
        }

        if let Ok(mut guard) = staged_slot.lock() {
            *guard = Some(dest_path);
        }

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn restart_application(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if let Ok(mut pump_guard) = state.pump.lock() {
        if let Some(mut existing) = pump_guard.take() {
            existing.stop();
        }
    }
    state.restore_physical_default_device();

    // Check if an in-app update installer was staged for installation on restart
    let staged_installer = state
        .staged_update_installer
        .lock()
        .ok()
        .and_then(|mut g| g.take())
        .or_else(|| {
            let default_path = std::env::temp_dir().join("OpenBoom_Setup_Update.exe");
            if default_path.exists() {
                Some(default_path)
            } else {
                None
            }
        });

    if let Some(installer_path) = staged_installer {
        if installer_path.exists() {
            let current_pid = std::process::id().to_string();
            let current_exe = std::env::current_exe()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let ps_apply = r#"
                $ProgressPreference = 'SilentlyContinue'
                $ErrorActionPreference = 'SilentlyContinue'
                $oldPid = [int]$env:OPENBOOM_OLD_PID
                $installer = $env:OPENBOOM_INSTALLER_PATH
                $currentExe = $env:OPENBOOM_CURRENT_EXE

                if ($oldPid -gt 0) {
                    $proc = Get-Process -Id $oldPid -ErrorAction SilentlyContinue
                    if ($proc) {
                        $proc.WaitForExit(5000) | Out-Null
                    }
                }

                Get-Process -Name "open-boom" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
                Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -like "OpenBoom_Setup_Update*" -and $_.Id -ne $PID } | Stop-Process -Force -ErrorAction SilentlyContinue
                Start-Sleep -Milliseconds 300

                $installedExe = Join-Path $env:LOCALAPPDATA "OpenBoom\open-boom.exe"
                if (Test-Path $installer) {
                    Start-Process -FilePath $installer -ArgumentList "/S", "/UPDATE" -Wait
                    if ((Test-Path $installedExe) -and $currentExe -and ($currentExe -ne $installedExe) -and (-not $currentExe.StartsWith($env:TEMP, [System.StringComparison]::OrdinalIgnoreCase))) {
                        Copy-Item -Path $installedExe -Destination $currentExe -Force -ErrorAction SilentlyContinue
                    }
                }

                Remove-Item -Path (Join-Path $env:TEMP "OpenBoom_Setup_Update*.exe") -Force -ErrorAction SilentlyContinue

                if (Test-Path $installedExe) {
                    Start-Process -FilePath $installedExe
                } elseif ($currentExe -and (Test-Path $currentExe)) {
                    Start-Process -FilePath $currentExe
                }
            "#;

            let mut cmd = std::process::Command::new("powershell");
            cmd.env("OPENBOOM_OLD_PID", &current_pid)
                .env("OPENBOOM_INSTALLER_PATH", &installer_path)
                .env("OPENBOOM_CURRENT_EXE", &current_exe)
                .args(["-NoProfile", "-NonInteractive", "-Command", ps_apply]);

            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }

            if cmd.spawn().is_ok() {
                app.exit(0);
                return Ok(());
            }
        }
    }

    app.restart();
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

            // Register Windows master endpoint volume change listener to emit Tauri events (FR-SYNC)
            let app_handle_vol = app.handle().clone();
            let state = app.state::<AppState>();
            state.volume_sync.add_on_volume_changed(move |event| {
                let _ = app_handle_vol.emit(
                    "windows-volume-changed",
                    WindowsVolumeChangedPayload {
                        volume: event.volume,
                        is_muted: event.is_muted,
                    },
                );
            });

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

            let preset_acoustic = MenuItem::with_id(app, "preset_acoustic", "Acoustic", true, None::<&str>)?;
            let preset_bass = MenuItem::with_id(app, "preset_bass", "Bass Boost", true, None::<&str>)?;
            let preset_vocal = MenuItem::with_id(app, "preset_vocal", "Vocal", true, None::<&str>)?;
            let presets_menu = Submenu::with_items(app, "Presets", true, &[&preset_acoustic, &preset_bass, &preset_vocal])?;

            let lang_pt = MenuItem::with_id(app, "lang_pt", "Português (Brasil)", true, None::<&str>)?;
            let lang_en = MenuItem::with_id(app, "lang_en", "English", true, None::<&str>)?;
            let lang_es = MenuItem::with_id(app, "lang_es", "Español", true, None::<&str>)?;
            let lang_menu = Submenu::with_items(app, "Language / Idioma", true, &[&lang_pt, &lang_en, &lang_es])?;

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
                    &lang_menu,
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
                            let _ = state.volume_sync.set_mute(next_mute);
                            let _ = state.dispatcher.send(AudioCommand::SetMute(next_mute));
                            let _ = app.emit("tray-mute-toggled", next_mute);
                        }
                        "preset_acoustic" | "preset_bass" | "preset_vocal" => {
                            let target_name = match event.id.as_ref() {
                                "preset_acoustic" => "Acoustic",
                                "preset_bass" => "Bass Boost",
                                _ => "Vocal",
                            };
                            let state = app.state::<AppState>();
                            let gains = PresetStore::factory_presets()
                                .into_iter()
                                .find(|p| p.name == target_name)
                                .map(|p| p.gains_db)
                                .unwrap_or([0.0; GRAPHIC_EQ_BANDS]);
                            let _ = state.dispatcher.send(AudioCommand::LoadGraphicEqSnapshot {
                                gains_db: gains,
                                preamp_offset_db: 0.0,
                            });
                            let _ = app.emit("tray-preset-selected", target_name);
                        }
                        "lang_pt" => {
                            let _ = app.emit("tray-language-changed", "pt-BR");
                        }
                        "lang_en" => {
                            let _ = app.emit("tray-language-changed", "en");
                        }
                        "lang_es" => {
                            let _ = app.emit("tray-language-changed", "es");
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
            apply_autoeq_profile,
            clear_autoeq_profile,
            get_eq_presets,
            save_custom_eq_preset,
            rename_custom_eq_preset,
            delete_custom_eq_preset,
            get_audio_sessions,
            set_app_volume,
            set_app_mute,
            get_app_volume_profiles,
            reset_app_volume_profile,
            set_buffer_guard,
            set_audio_bridge_mode,
            get_audio_bridge_mode,
            set_orbit_8d,
            set_master_volume,
            set_mute,
            check_for_updates,
            download_and_install_update,
            restart_application,
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

        assert_eq!(window["width"], 1040, "Window width must be 1040px");
        assert_eq!(window["height"], 640, "Window height must be 640px");
        assert_eq!(window["minWidth"], 1000, "Window minWidth must be 1000px");
        assert_eq!(window["minHeight"], 600, "Window minHeight must be 600px");
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

        // 10. SetAutoEqBypass
        let cmd10: AudioCommand = serde_json::from_str(r#"{"SetAutoEqBypass": {"bypassed": true}}"#).unwrap();
        assert_eq!(cmd10, AudioCommand::SetAutoEqBypass { bypassed: true });

        // 11. SetAutoEqProfile
        let filters = [open_boom_core::types::ParametricFilterPod::default(); open_boom_core::types::MAX_PARAMETRIC_FILTERS];
        let filters_json = serde_json::to_string(&filters).unwrap();
        let autoeq_json = format!(
            r#"{{"SetAutoEqProfile": {{"filters": {}, "filter_count": 1, "preamp_offset_db": -5.2}}}}"#,
            filters_json
        );
        let cmd11: AudioCommand = serde_json::from_str(&autoeq_json).unwrap();
        assert_eq!(
            cmd11,
            AudioCommand::SetAutoEqProfile {
                filters,
                filter_count: 1,
                preamp_offset_db: -5.2,
            }
        );

        // 12. SetOrbit8D
        let cmd12: AudioCommand = serde_json::from_str(
            r#"{"SetOrbit8D": {"enabled": true, "direction": "Clockwise", "period_sec": 12.0, "depth": 0.85}}"#,
        )
        .unwrap();
        assert_eq!(
            cmd12,
            AudioCommand::SetOrbit8D {
                enabled: true,
                direction: open_boom_core::types::OrbitDirection::Clockwise,
                period_sec: 12.0,
                depth: 0.85,
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

    #[test]
    fn test_eq_preset_commands_integration() {
        let temp_dir = std::env::temp_dir().join(format!("tauri_presets_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let store = PresetStore::new(&temp_dir);
        let startup = StartupManager::mock();
        let state = AppState::with_store_and_startup(store, startup);

        let initial_presets = state.store.list_presets();
        assert_eq!(initial_presets.len(), 15);
        assert!(initial_presets.iter().all(|p| p.is_factory));

        let mut custom_gains = [0.0f32; GRAPHIC_EQ_BANDS];
        custom_gains[0] = 3.5;
        custom_gains[17] = 2.0;

        let saved = state
            .store
            .save_custom_preset("Studio Reference", custom_gains)
            .expect("save custom preset");
        assert_eq!(saved.id, "studio-reference");
        assert!(!saved.is_factory);
        assert_eq!(state.store.list_presets().len(), 16);

        let renamed = state
            .store
            .rename_custom_preset("studio-reference", "Mastering Curve")
            .expect("rename custom preset");
        assert_eq!(renamed.id, "mastering-curve");
        assert_eq!(renamed.name, "Mastering Curve");

        state
            .store
            .delete_custom_preset("mastering-curve")
            .expect("delete custom preset");
        assert_eq!(state.store.list_presets().len(), 15);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_session_manager_ipc_commands_integration() {
        let state = AppState::new();
        let mut mgr = state.session_manager.lock().expect("lock session manager");

        let sessions = mgr.enumerate_sessions().expect("enumerate sessions");
        assert!(!sessions.is_empty());
        let pid = sessions[0].process_id;

        mgr.set_process_volume(pid, 0.42).expect("set process volume");
        let updated = mgr.enumerate_sessions().expect("enumerate sessions");
        let session = updated.iter().find(|s| s.process_id == pid).unwrap();
        assert!((session.volume - 0.42).abs() < 1e-4);

        mgr.set_process_mute(pid, true).expect("mute process");
        let updated2 = mgr.enumerate_sessions().expect("enumerate sessions");
        let session2 = updated2.iter().find(|s| s.process_id == pid).unwrap();
        assert!(session2.is_muted);
    }

    #[test]
    fn test_app_volume_persistence_integration() {
        let temp_dir = std::env::temp_dir().join(format!("tauri_appvol_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let store = PresetStore::new(&temp_dir);
        let startup = StartupManager::mock();
        let state = AppState::with_store_and_startup(store, startup);

        // Save a profile
        state
            .store
            .save_app_volume_profile(&AppVolumeProfile::new("spotify.exe", 0.58, false))
            .expect("save profile");

        // Verify loaded in list
        let profiles = state.store.load_app_volume_list().expect("load profiles");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].process_name, "spotify.exe");
        assert!((profiles[0].volume - 0.58).abs() < 1e-4);

        // Check deletion
        let deleted = state.store.delete_app_volume_profile("spotify.exe").expect("delete profile");
        assert!(deleted);
        assert!(state.store.load_app_volume_list().unwrap().is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_set_orbit_8d_command_dispatch() {
        let state = AppState::new();

        // 1. Dispatch 8D Orbit command through dispatcher
        let res = state.dispatcher.send(AudioCommand::SetOrbit8D {
            enabled: true,
            direction: OrbitDirection::Clockwise,
            period_sec: 12.0,
            depth: 0.85,
        });
        assert!(res.is_ok(), "Dispatching SetOrbit8D must succeed");

        // 2. Allow background dispatcher thread to process the channel and verify command popped
        let mut cmd = None;
        for _ in 0..20 {
            if let Some(c) = state.command_queue.pop() {
                cmd = Some(c);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        assert_eq!(
            cmd,
            Some(AudioCommand::SetOrbit8D {
                enabled: true,
                direction: OrbitDirection::Clockwise,
                period_sec: 12.0,
                depth: 0.85,
            }),
            "SPSC queue must receive SetOrbit8D POD command without corruption"
        );
    }

    #[test]
    fn test_resolve_orbit_params_defaults_and_overrides() {
        // Defaults when None is passed
        let (dir, period, depth) = resolve_orbit_params(None, None, None);
        assert_eq!(dir, OrbitDirection::Clockwise);
        assert_eq!(period, 12.0);
        assert_eq!(depth, 0.85);

        // Custom parameters preserved
        let (dir2, period2, depth2) = resolve_orbit_params(
            Some(OrbitDirection::PingPong),
            Some(6.5),
            Some(1.0),
        );
        assert_eq!(dir2, OrbitDirection::PingPong);
        assert_eq!(period2, 6.5);
        assert_eq!(depth2, 1.0);
    }

    #[test]
    fn test_windows_volume_sync_to_dispatcher_and_queue() {
        let state = AppState::new();

        // 1. Simulate OS volume change (e.g. keyboard media key or Windows taskbar) to 45% (0.45)
        let processed = state.volume_sync.simulate_os_volume_change(0.45, false, None);
        assert!(processed, "OS volume change must be processed");

        // Drain commands from command queue
        let mut found_volume = false;
        let mut found_mute = false;
        for _ in 0..30 {
            if let Some(cmd) = state.command_queue.pop() {
                match cmd {
                    AudioCommand::SetMasterVolume { volume, boost } => {
                        let expected_linear = 0.45 * 0.45;
                        assert!((volume - expected_linear).abs() < 1e-4, "Linear volume should be 0.45^2");
                        assert_eq!(boost, 1.0, "Boost should be 1.0 for <= 100%");
                        found_volume = true;
                    }
                    AudioCommand::SetMute(muted) => {
                        assert!(!muted, "Mute should be false");
                        found_mute = true;
                    }
                    _ => {}
                }
            }
            if found_volume && found_mute {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        assert!(found_volume, "SPSC queue must receive SetMasterVolume from OS change");
        assert!(found_mute, "SPSC queue must receive SetMute from OS change");

        // 2. Simulate OS mute toggle
        let processed_mute = state.volume_sync.simulate_os_volume_change(0.45, true, None);
        assert!(processed_mute);

        let mut found_mute_true = false;
        for _ in 0..30 {
            if let Some(cmd) = state.command_queue.pop() {
                if let AudioCommand::SetMute(muted) = cmd {
                    if muted {
                        found_mute_true = true;
                        break;
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(found_mute_true, "SPSC queue must receive SetMute(true) from OS mute toggle");
    }

    #[test]
    fn test_dispatch_command_syncs_to_windows_endpoint() {
        let state = AppState::new();

        // 1. Dispatch SetMasterVolume 80% (linear 0.64, boost 1.0) through unified command ingress
        let res1 = state.dispatch_audio_command(AudioCommand::SetMasterVolume {
            volume: 0.64,
            boost: 1.0,
        });
        assert!(res1.is_ok());
        let vol = state.volume_sync.get_master_volume().unwrap();
        assert!((vol - 0.80).abs() < 1e-4);

        // 2. Dispatch SetMasterVolume 135% (boost 1.35) -> Windows master volume clamped to 1.0 (100%)
        let res2 = state.dispatch_audio_command(AudioCommand::SetMasterVolume {
            volume: 1.0,
            boost: 1.35,
        });
        assert!(res2.is_ok());
        let vol_boost = state.volume_sync.get_master_volume().unwrap();
        assert!((vol_boost - 1.0).abs() < 1e-4, "Boost > 100% must clamp Windows endpoint to 1.0");

        // 3. Dispatch SetMute through unified command ingress
        let res3 = state.dispatch_audio_command(AudioCommand::SetMute(true));
        assert!(res3.is_ok());
        assert!(state.volume_sync.is_muted().unwrap());

        let res4 = state.dispatch_audio_command(AudioCommand::SetMute(false));
        assert!(res4.is_ok());
        assert!(!state.volume_sync.is_muted().unwrap());
    }

    #[test]
    fn test_version_comparison_logic() {
        assert!(is_version_newer("0.4.0", "0.3.0"));
        assert!(is_version_newer("1.0.0", "0.3.0"));
        assert!(is_version_newer("0.3.1", "0.3.0"));
        assert!(!is_version_newer("0.3.0", "0.3.0"));
        assert!(!is_version_newer("0.2.9", "0.3.0"));
        assert!(is_version_newer("0.3.0", "0.3.0-beta.1"));
        assert!(!is_version_newer("0.3.0-beta.1", "0.3.0"));
    }

    #[test]
    fn test_apply_audio_bridge_mode() {
        let temp_dir = std::env::temp_dir().join(format!(
            "open_boom_bridge_mode_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let store = PresetStore::new(&temp_dir);
        let startup = StartupManager::mock();
        let state = AppState::with_store_and_startup(store.clone(), startup);

        // Seed synthetic device IDs for deterministic routing state checks without mutating OS endpoints
        *state.active_physical_device_id.lock().unwrap() = Some("phys_dev_1".to_string());
        *state.virtual_device_id.lock().unwrap() = Some("virt_dev_1".to_string());

        // 1. Switch to "virtual" (Mode A)
        let applied_v = state.apply_audio_bridge_mode("virtual").expect("apply virtual mode");
        assert_eq!(applied_v, "virtual");
        assert!(!state.is_loopback_forced());
        assert!(state.get_telemetry_snapshot().using_virtual_bridge);
        assert!(*state.is_routed_to_virtual.lock().unwrap());
        assert_eq!(store.load_config().unwrap().audio_bridge_mode, "virtual");

        // Verify a newly constructed AppState restores "virtual" from PresetStore on startup
        let restarted_virtual = AppState::with_store_and_startup(store.clone(), StartupManager::mock());
        assert!(!restarted_virtual.is_loopback_forced());
        assert!(restarted_virtual.get_telemetry_snapshot().using_virtual_bridge);
        assert_eq!(*restarted_virtual.audio_bridge_mode.lock().unwrap(), "virtual");

        // 2. Switch to "loopback" (Mode B without driver)
        let applied_l = state.apply_audio_bridge_mode("loopback").expect("apply loopback mode");
        assert_eq!(applied_l, "loopback");
        assert!(state.is_loopback_forced());
        assert!(!state.get_telemetry_snapshot().using_virtual_bridge);
        assert!(!*state.is_routed_to_virtual.lock().unwrap());
        assert_eq!(store.load_config().unwrap().audio_bridge_mode, "loopback");

        // Verify handle_master_power_toggle(true) respects force_loopback and does not route to virtual sink
        state.handle_master_power_toggle(true).expect("toggle master power in loopback mode");
        assert!(!*state.is_routed_to_virtual.lock().unwrap());

        // Verify a newly constructed AppState restores "loopback" from PresetStore on startup
        let restarted_loopback = AppState::with_store_and_startup(store.clone(), StartupManager::mock());
        assert!(restarted_loopback.is_loopback_forced());
        assert!(!restarted_loopback.get_telemetry_snapshot().using_virtual_bridge);
        assert_eq!(*restarted_loopback.audio_bridge_mode.lock().unwrap(), "loopback");

        // 3. Invalid mode sanitizes back to "auto"
        let applied_a = state.apply_audio_bridge_mode("invalid_mode").expect("sanitize to auto");
        assert_eq!(applied_a, "auto");
        assert!(!state.is_loopback_forced());
        assert_eq!(store.load_config().unwrap().audio_bridge_mode, "auto");

        // Verify restore_physical_default_device clears is_routed_to_virtual
        state.restore_physical_default_device();
        assert!(!*state.is_routed_to_virtual.lock().unwrap());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_select_installer_asset_url_prioritizes_setup_over_portable() {
        // Exact asset order returned by GitHub Releases API (where 'OpenBoom-v0.5.0-portable-x64.exe'
        // sorts ASCII-wise before 'OpenBoom_0.5.0_x64-setup.exe')
        let assets_json: serde_json::Value = serde_json::from_str(
            r#"[
                {
                    "name": "OpenBoom-v0.5.0-portable-x64.exe",
                    "browser_download_url": "https://github.com/JVGS1111/open-boom/releases/download/v0.5.0/OpenBoom-v0.5.0-portable-x64.exe"
                },
                {
                    "name": "OpenBoom_0.5.0_x64-setup.exe",
                    "browser_download_url": "https://github.com/JVGS1111/open-boom/releases/download/v0.5.0/OpenBoom_0.5.0_x64-setup.exe"
                },
                {
                    "name": "SHA256SUMS.txt",
                    "browser_download_url": "https://github.com/JVGS1111/open-boom/releases/download/v0.5.0/SHA256SUMS.txt"
                }
            ]"#,
        )
        .expect("parse assets json");

        let selected = select_installer_asset_url(assets_json.as_array().unwrap());
        assert_eq!(
            selected.as_deref(),
            Some("https://github.com/JVGS1111/open-boom/releases/download/v0.5.0/OpenBoom_0.5.0_x64-setup.exe"),
            "Must select the NSIS setup installer and never the portable binary when both exist"
        );

        // Verify DownloadInFlightGuard resets flag on drop
        let flag = Arc::new(std::sync::atomic::AtomicBool::new(true));
        {
            let _g = DownloadInFlightGuard { flag: flag.clone() };
            assert!(flag.load(std::sync::atomic::Ordering::SeqCst));
        }
        assert!(!flag.load(std::sync::atomic::Ordering::SeqCst));
    }
}



