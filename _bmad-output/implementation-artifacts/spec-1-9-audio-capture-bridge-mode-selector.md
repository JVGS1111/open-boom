---
title: 'Story 1.9: Audio Capture Bridge Mode Selector & Automatic Application Restart'
type: 'feature'
created: '2026-10-12'
baseline_revision: '813ec329baa030209d20fc7f9407eaeb3f86ddba'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-1-context.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** Users and developers testing OpenBoom on machines with or without the Boom 3D Virtual Audio Driver (`Boom Audio` / `boomvad.sys`) have no UI control in Settings to manually switch between automatic detection (`"auto"`), forced Virtual Audio Driver Mode A (`"virtual"`), and forced driverless WASAPI Loopback emulation Mode B (`"loopback"`).

**Approach:** Add an Audio Capture Bridge Mode selector (`<select>`) in the Settings modal (`SettingsModal.tsx` -> General tab), persist the selected mode (`"auto"`, `"virtual"`, `"loopback"`) atomically in `%APPDATA%\OpenBoom\config.json`, apply the forced bridge mode to `VirtualAudioBridge`, `AudioEngineCoordinator`, and `AudioStreamPump`, and automatically restart the application when the user switches the capture channel (cleanly restoring the physical Windows default endpoint first when switching to `"loopback"`).

## Boundaries & Constraints

**Always:**
- Persist `audio_bridge_mode` (`"auto"`, `"virtual"`, `"loopback"`) atomically in `%APPDATA%\OpenBoom\config.json` with safe backward-compatible fallback to `"auto"` via `sanitize_audio_bridge_mode`.
- Apply the configured mode on startup and via Tauri IPC (`set_audio_bridge_mode`, `get_audio_bridge_mode`) across `VirtualAudioBridge::set_force_mode`, `AudioEngineCoordinator::set_bridge_force_mode`, and `StreamPumpConfig::force_loopback`.
- Bypass virtual audio cable routing in `AudioStreamPump` when `force_loopback: true` is active so WASAPI Loopback captures directly from the physical render device even if `Boom Audio` is installed on the host machine.
- When `restart_application` is invoked while `audio_bridge_mode == "loopback"`, cleanly stop the active `AudioStreamPump` and restore the physical Windows default endpoint via `IPolicyConfig` if Windows is currently stuck on a virtual device (`Boom Audio`), before calling `app.restart()`.
- Render the selector in `SettingsModal.tsx` (General tab) with live active channel badge (`Mode A` vs `Mode B`), amber warning notice when in Loopback Mode B, and automatic invocation of `set_audio_bridge_mode` followed by `restart_application` on change.
- Maintain 100% i18n translation parity across English (`en`), Portuguese (`pt-BR`), and Spanish (`es`).

**Never:**
- Never require proprietary kernel drivers to launch or operate OpenBoom in `"auto"` or `"loopback"` mode.
- Never introduce heap allocations on the real-time audio callback thread (`NFR2`).
- Never leave Windows default audio output stranded on `Boom Audio` when the user explicitly switches to `"loopback"` and restarts.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Default / Legacy `config.json` without `audio_bridge_mode` | Missing or unrecognized string in JSON | Deserializes and sanitizes to `"auto"`; `VirtualAudioBridge::force_mode()` is `None` | Graceful fallback to `"auto"` |
| User selects `"loopback"` (Driverless Emulation Mode B) | User changes `<select>` to `"loopback"` in Settings | Saves `"loopback"` to `config.json`, sets `force_mode(Some(false))`, stops pump and restores physical default endpoint if needed, and restarts app | If IPC fails, resets restarting state and logs error |
| User selects `"virtual"` (With Boom Audio Driver Mode A) | User changes `<select>` to `"virtual"` in Settings | Saves `"virtual"` to `config.json`, sets `force_mode(Some(true))`, and restarts app | If virtual device absent on host, falls back safely to loopback capture |
| User selects `"auto"` (Recommended) | User changes `<select>` to `"auto"` in Settings | Saves `"auto"` to `config.json`, clears `force_mode(None)`, and restarts app | Automatic hardware detection on relaunch |
| Same mode re-selected in dropdown | User selects the already-active mode | Early return in `handleBridgeModeSelect`; no redundant IPC write or app restart triggered | No-op |

</intent-contract>

## Code Map

- `crates/open-boom-storage/src/preset_store.rs` -- Adds `audio_bridge_mode: String` to `AppConfig` and `sanitize_audio_bridge_mode` helper.
- `crates/open-boom-storage/src/lib.rs` -- Re-exports `sanitize_audio_bridge_mode`.
- `crates/open-boom-wasapi/src/virtual_bridge.rs` -- Adds `force_mode: Option<BridgeMode>`, `set_force_mode`, and `force_mode()` to `VirtualAudioBridge`.
- `crates/open-boom-wasapi/src/coordinator.rs` -- Adds `set_bridge_force_mode` and `bridge_force_mode` to `AudioEngineCoordinator`.
- `crates/open-boom-wasapi/src/stream_pump.rs` -- Adds `force_loopback: bool` to `StreamPumpConfig` and bypasses virtual cable discovery in `AudioStreamPump::run_windows_pump` when `force_loopback` is true.
- `crates/open-boom-wasapi/tests/virtual_bridge_tests.rs` -- Unit tests for `VirtualAudioBridge::set_force_mode` and `AudioEngineCoordinator::set_bridge_force_mode`.
- `src-tauri/src/lib.rs` -- Adds `audio_bridge_mode` to `AppState`, `with_options`, `apply_audio_bridge_mode`, `is_loopback_forced`, IPC commands `set_audio_bridge_mode` and `get_audio_bridge_mode`, and physical endpoint restoration in `restart_application`.
- `src-ui/src/i18n/types.ts`, `src-ui/src/i18n/locales/en.ts`, `src-ui/src/i18n/locales/pt-BR.ts`, `src-ui/src/i18n/locales/es.ts` -- Adds localization keys for the Audio Capture Bridge Mode selector card, options, badges, and warnings.
- `src-ui/src/components/SettingsModal.tsx` -- Renders the Audio Capture Bridge Mode card, `<select>`, active channel badge, Mode B warning banner, and triggers `onChangeAudioBridgeMode`.
- `src-ui/src/App.tsx` -- Manages `audioBridgeMode` and `isSwitchingBridge` state, loads initial value from `load_app_config`, and implements `handleAudioBridgeModeChange`.
- `src-ui/src/test/SettingsModal.test.tsx` -- Unit tests verifying rendering, selected option state, Mode A / Mode B active badges, loopback warning banner, and disabled state while switching.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-storage/src/preset_store.rs` -- Add `audio_bridge_mode` field and `sanitize_audio_bridge_mode` -- Persists user's bridge mode selection atomically in `config.json`.
- `crates/open-boom-wasapi/src/virtual_bridge.rs` -- Add `set_force_mode` and `force_mode` to `VirtualAudioBridge` -- Allows overriding auto-detection between Mode A (`Some(VirtualSinkModeA)`) and Mode B (`Some(LoopbackFallbackModeB)`).
- `crates/open-boom-wasapi/src/coordinator.rs` -- Expose `set_bridge_force_mode` and `bridge_force_mode` on `AudioEngineCoordinator` -- Propagates mode override to the coordinator and bridge.
- `crates/open-boom-wasapi/src/stream_pump.rs` -- Add `force_loopback` to `StreamPumpConfig` and respect it in `run_windows_pump` -- Ensures Mode B captures via WASAPI Loopback from physical render device even when Boom Audio is installed.
- `src-tauri/src/lib.rs` -- Implement `set_audio_bridge_mode` and `get_audio_bridge_mode` IPC commands and physical endpoint cleanup in `restart_application` -- Connects UI selection to storage, audio engine, and clean process restart.
- `src-ui/src/i18n/` -- Add bridge mode translation keys across `types.ts`, `en.ts`, `pt-BR.ts`, and `es.ts` -- Ensures 100% localization parity.
- `src-ui/src/components/SettingsModal.tsx` & `src-ui/src/App.tsx` -- Render bridge mode `<select>` and trigger `set_audio_bridge_mode` + `restart_application` -- Delivers user-facing control with automatic app restart.
- `crates/open-boom-wasapi/tests/virtual_bridge_tests.rs`, `crates/open-boom-storage/tests/storage_tests.rs`, `src-tauri/src/lib.rs` & `src-ui/src/test/SettingsModal.test.tsx` -- Add unit tests for Rust bridge mode override, storage sanitization, startup restoration, and React Settings selector -- Verifies all I/O matrix scenarios.

**Acceptance Criteria:**
- Given the Settings modal is open on the General tab, when the user views the Audio Capture Bridge Mode card, then a `<select>` dropdown displays `"auto"`, `"virtual"`, and `"loopback"` options along with a badge showing the currently active channel (`Mode A` or `Mode B`).
- Given the user selects a different mode in the `<select>` dropdown, when the change event fires, then OpenBoom persists `audio_bridge_mode` to `%APPDATA%\OpenBoom\config.json` via `set_audio_bridge_mode` and immediately calls `restart_application`.
- Given OpenBoom starts or restarts with `audio_bridge_mode: "loopback"`, when the audio engine initializes, then `AudioStreamPump` bypasses virtual cable routing and captures directly via WASAPI Loopback (Mode B) even if `Boom Audio` is installed on the host.
- Given OpenBoom is switching to `"loopback"` and restarting, when `restart_application` executes, then the active stream pump stops and restores the physical default Windows endpoint before relaunching.

## Spec Change Log

<!-- Append-only. Populated by step-04 during review loops. -->

## Review Triage Log

### 2026-10-12 — Review pass
- verdicts: 12 findings — high 1, medium 7, low 4, false 0, maybe-false 0
- findings:
  - `[high]` `[patch]` `AppState::with_store_and_startup` called `Self::new()` before replacing `state.store`, reading host `%APPDATA%\OpenBoom\config.json` and hardcoding `bridge_mode = "auto"` when `auto_route == false` — Patched `AppState` in `src-tauri/src/lib.rs` by introducing `AppState::with_options(store, startup, auto_route)` so injected stores are used during initialization and `audio_bridge_mode` is always loaded and sanitized from `store`.
  - `[medium]` `[patch]` `restart_application` only called `restore_physical_default_device()` when `is_loopback_forced()` was true, and `restore_physical_default_device()` was a no-op if `is_routed_to_virtual` had already been cleared by `apply_audio_bridge_mode("loopback")` — Patched `restart_application` in `src-tauri/src/lib.rs` to call `state.restore_physical_default_device()` unconditionally before `app.restart()`, and updated `restore_physical_default_device()` to restore the physical endpoint whenever `was_routed || self.is_loopback_forced()`.
  - `[medium]` `[patch]` `VirtualAudioBridge::detect` fabricated a synthetic virtual device when `force_mode == Some(VirtualSinkModeA)` even when `allow_synthetic_fallback` was `false` and no virtual driver existed on the host, and `set_force_mode` only cleared `virtual_device` when `force_mode.is_none()` — Patched `crates/open-boom-wasapi/src/virtual_bridge.rs` so forced Mode A falls back to `LoopbackFallbackModeB` when no driver is present and `allow_synthetic_fallback` is `false`, and `set_force_mode` unconditionally resets `self.virtual_device = None` before re-detecting.
  - `[medium]` `[patch]` `handleAudioBridgeModeChange` in `src-ui/src/App.tsx` lacked an early-return guard for re-selecting the active mode and called `handleRestartApplication()` outside the `try` block even when `set_audio_bridge_mode` failed — Patched `App.tsx` and `SettingsModal.tsx` to return early when `sanitized === audioBridgeMode || isSwitchingBridge`, move `await handleRestartApplication()` inside the `try` block, roll back optimistic state on IPC error, and disable the `<select>` while switching.
  - `[medium]` `[patch]` Auto-save `useEffect` in `src-ui/src/App.tsx` scheduled a 250ms `save_app_config` timer when `audioBridgeMode` state changed, racing with `set_audio_bridge_mode`, and `save_app_config` in `src-tauri/src/lib.rs` did not synchronize `config.audio_bridge_mode` with `state.audio_bridge_mode` — Patched `App.tsx` to skip auto-save when `isSwitchingBridge` is true, and patched `save_app_config` in `src-tauri/src/lib.rs` to synchronize and sanitize `config.audio_bridge_mode` from `state.audio_bridge_mode`.
  - `[medium]` `[patch]` `SettingsModal.tsx` hid the active capture channel badge (`t.settings.bridgeModeActiveLoopback`) when `audioBridgeMode === "loopback"` and omitted the Mode B warning banner when `"auto"` mode fell back to Mode B (`usingVirtualBridge === false`) — Patched `src-ui/src/components/SettingsModal.tsx` to always render the active channel badge and render the amber warning banner whenever `audioBridgeMode === "loopback" || !usingVirtualBridge`.
  - `[medium]` `[patch]` `AppState::with_options` started `AudioStreamPump` before calling `device_mgr.set_default_device(...)` and ignored `bypass_active` and `active_output_device` on startup — Patched `AppState::with_options` in `src-tauri/src/lib.rs` to configure Windows default endpoint routing before `AudioStreamPump::start`, check `app_config.master_power && !app_config.bypass_active`, and respect `app_config.active_output_device`.
  - `[medium]` `[patch]` `apply_audio_bridge_mode` and `handle_master_power_toggle` in `src-tauri/src/lib.rs` did not drain pending coordinator commands to check `is_dsp_active()`, ignored `save_config` errors, and `handle_master_power_toggle` called `device_mgr.set_default_device` without checking `self.auto_route` — Patched `src-tauri/src/lib.rs` to propagate `save_config` errors, call `coord.drain_commands()` + `coord.processor().is_dsp_active()`, and guard OS endpoint mutation behind `self.auto_route`.
  - `[low]` `[patch]` Missing `AudioEngineCoordinator::bridge_force_mode` accessor specified in the Code Map — Added `pub fn bridge_force_mode(&self) -> Option<BridgeMode>` in `crates/open-boom-wasapi/src/coordinator.rs`.
  - `[low]` `[patch]` Verification gap in `src-ui/src/test/SettingsModal.test.tsx` where `"Mode A"` matched static `<option>` text rather than the active badge or `selected=""` attribute — Patched `SettingsModal.test.tsx` to assert exact `"Active Channel: Mode A"` / `"Active Channel: Mode B"` badges, `<option ... selected="">`, auto-fallback warning banner, and `disabled=""` state.
  - `[low]` `[patch]` Verification gap in `crates/open-boom-storage/tests/storage_tests.rs` for `audio_bridge_mode` default and invalid/untrimmed JSON sanitization on `load_config()` — Added assertions in `storage_tests.rs` covering missing `audio_bridge_mode`, `"invalid_channel"`, and `"  VIRTUAL  "`.
  - `[low]` `[patch]` Verification gap in `crates/open-boom-wasapi/tests/virtual_bridge_tests.rs` and `src-tauri/src/lib.rs` for `virtual_device()` clearing, Mode B override of a present virtual device, startup restoration of `audio_bridge_mode`, and `handle_master_power_toggle` in `"loopback"` mode — Added assertions in `virtual_bridge_tests.rs` and `test_apply_audio_bridge_mode` in `src-tauri/src/lib.rs`.

## Design Notes

- **Three-State Bridge Mode Override:**
  - `"auto"` $\to$ `force_mode = None`: Uses `VirtualAudioBridge::detect()` hardware presence (`Boom Audio` if installed, else WASAPI Loopback).
  - `"virtual"` $\to$ `force_mode = Some(VirtualSinkModeA)`: Forces Mode A (`VirtualSinkModeA`) when a virtual endpoint exists (or synthetic fallback is enabled), falling back safely to `LoopbackFallbackModeB` if no virtual device is present on a production host.
  - `"loopback"` $\to$ `force_mode = Some(LoopbackFallbackModeB)` & `StreamPumpConfig.force_loopback = true`: Forces Mode B (`LoopbackFallbackModeB`) and instructs `AudioStreamPump` to skip `find_virtual_render_device()`, capturing WASAPI Loopback directly from `render_device`.

## Verification

**Commands:**
- `cargo test --workspace` -- expected: All workspace unit and integration tests pass (214 passed, 0 failed).
- `npm --prefix src-ui test` -- expected: All frontend unit tests pass including `SettingsModal.test.tsx` (106 passed, 0 failed).
- `npm --prefix src-ui run build` -- expected: Production Vite and TypeScript build succeeds with 0 errors.
- `cargo build -p open-boom` -- expected: Local desktop application binary compiles with 0 errors.

## Auto Run Result

- **Status:** `done`
- **Summary of Implemented Change:** Implemented the Audio Capture Bridge Mode selector (`"auto"`, `"virtual"`, `"loopback"`) in `SettingsModal.tsx` (General tab) with full 3-locale i18n parity (`en`, `pt-BR`, `es`), atomic persistence in `%APPDATA%\OpenBoom\config.json` (`AppConfig.audio_bridge_mode`), three-state bridge mode override in `VirtualAudioBridge` and `AudioEngineCoordinator`, forced physical WASAPI Loopback capture in `AudioStreamPump` (`StreamPumpConfig.force_loopback`), and automatic application restart via `restart_application` with pre-restart physical default endpoint restoration.
- **Files Changed:**
  - `crates/open-boom-storage/src/preset_store.rs` & `crates/open-boom-storage/src/lib.rs` — Added `audio_bridge_mode: String` to `AppConfig` and `sanitize_audio_bridge_mode` helper.
  - `crates/open-boom-storage/tests/storage_tests.rs` — Added unit test assertions for `audio_bridge_mode` default fallback and JSON load sanitization.
  - `crates/open-boom-wasapi/src/virtual_bridge.rs` — Added `force_mode: Option<BridgeMode>`, `set_force_mode`, and `force_mode()` with safe fallback when no virtual driver is present.
  - `crates/open-boom-wasapi/src/coordinator.rs` — Added `set_bridge_force_mode` and `bridge_force_mode`.
  - `crates/open-boom-wasapi/src/stream_pump.rs` — Added `force_loopback: bool` to `StreamPumpConfig` and bypassed virtual device lookup in `run_windows_pump` when `force_loopback` is true.
  - `crates/open-boom-wasapi/tests/virtual_bridge_tests.rs` — Added `test_dynamic_set_force_mode_switching` verifying Mode A, Mode B override, Auto reset, and `virtual_device()` clearing.
  - `src-tauri/src/lib.rs` — Added `AppState::with_options`, `apply_audio_bridge_mode`, `is_loopback_forced`, `set_audio_bridge_mode` / `get_audio_bridge_mode` IPC commands, unconditional `restore_physical_default_device()` in `restart_application`, and `test_apply_audio_bridge_mode`.
  - `src-ui/src/i18n/types.ts`, `src-ui/src/i18n/locales/en.ts`, `src-ui/src/i18n/locales/pt-BR.ts`, `src-ui/src/i18n/locales/es.ts` — Added translation keys for the Audio Capture Bridge Mode selector across all 3 locales.
  - `src-ui/src/components/SettingsModal.tsx` & `src-ui/src/App.tsx` — Added the Audio Capture Bridge Mode selector card, active channel badge, Mode B warning banner, early-return and error rollback guards, and automatic restart trigger.
  - `src-ui/src/test/SettingsModal.test.tsx` — Added unit tests for Mode A / Mode B badges, selected option attribute, warning banner, and disabled switching state.
- **Review Findings Breakdown:** 12 findings (1 high, 7 medium, 4 low) — all 12 patched and verified; 0 deferred; 0 rejected.
- **Follow-up Review Recommendation:** `false` (all patched items are covered by deterministic Rust and React unit tests with zero remaining unverified risks).
- **Verification Performed:**
  - `cargo test --workspace`: 214 passed, 0 failed.
  - `npm --prefix src-ui test`: 106 passed, 0 failed.
  - `npm --prefix src-ui run build`: Succeeded in 1.90s with 0 errors.
  - `cargo build -p open-boom` + live smoke launch of `target\debug\open-boom.exe`: Process started cleanly (`PID=18752`, `HasExited=False`, `Responding=True`).
- **Residual Risks:** None.
