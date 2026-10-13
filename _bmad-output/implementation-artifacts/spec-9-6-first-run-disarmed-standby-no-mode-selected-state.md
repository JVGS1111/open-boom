---
title: 'First-Run Disarmed Standby State (No Audio Bridge Default Mode B)'
type: 'feature'
created: '2026-10-13'
status: 'done'
baseline_revision: '20bca779570c8a6142bc07f6337674854c0ef93e'
review_loop_iteration: 1
followup_review_recommended: false
context: []
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** When OpenBoom boots on a fresh host without an installed virtual audio driver (`CABLE Input` / `Boom Audio`), the previous logic automatically fell back to WASAPI Process Loopback (Mode B). Because Mode B currently exhibits acoustic duplication and slapback echo (`BUG-MODE-B-01`), any playing media was captured and echoed through the physical headphones before the user could even install the driver.

**Approach:** Implement a first-run disarmed Standby state (`Standby`). When `audio_bridge_mode == "auto"` and no virtual audio device is present:
1. `AudioStreamPump` is not started (`pump == None`), and the worker thread enters safe sleep without opening capture or rendering streams.
2. Windows default audio endpoint is left completely untouched on the physical device.
3. The UI status bar and telemetry reflect `Standby (Driver Required)` rather than claiming active loopback.
4. Mode B loopback is only activated if the user explicitly opts into it via settings (`audio_bridge_mode == "loopback"`).
5. Installing the driver via 1-click setup or starting with a virtual device detected automatically exits Standby and engages Mode A virtual sink.

## Boundaries & Constraints

**Always:**
- Keep Windows physical audio device completely untouched on first boot without driver.
- Ensure `should_start_pump = force_loopback || virtual_id.is_some()`.
- Provide full i18n dictionary parity across `en`, `pt-BR`, and `es`.
- Pass all Cargo workspace unit tests and React frontend tests.

**Never:**
- Silently activate Mode B loopback capture in "auto" mode when virtual driver is absent.
- Cause audio glitching or panics when pump is in `None` / Standby state.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| First boot, driver absent | `audio_bridge_mode = "auto"`, no virtual device | Pump is `None` (Standby), Windows audio untouched, UI shows Standby badge | Handled gracefully, no echo |
| Explicit Mode B selection | User selects `audio_bridge_mode = "loopback"` | Pump starts with `force_loopback: true`, UI shows `Loopback Mode [BETA]` | Normal Mode B startup |
| 1-Click driver install | Driver installs successfully | `virtual_id` updated, pump hot-restarts in Mode A, Standby exits | Normal Mode A activation |
| Existing driver on boot | Virtual device detected on boot | Pump starts in Mode A, virtual sink engaged | Zero regression for existing installs |

</intent-contract>

## Code Map

- `crates/open-boom-wasapi/src/stream_pump.rs` -- Standby loop in `run_windows_pump` when `!config.force_loopback && capture_device_opt.is_none()`.
- `crates/open-boom-wasapi/tests/stream_pump_tests.rs` -- Unit test verifying standby behavior when driver is absent.
- `src-tauri/src/lib.rs` -- `AppState::with_options`, `apply_audio_bridge_mode`, and pump hot-restart routines gating pump start on `should_start_pump`.
- `src-ui/src/components/StatusBar.tsx` -- Render `Standby (Driver Required)` badge when driver is missing in auto mode.
- `src-ui/src/App.tsx` -- Pass standby indicator to `StatusBar`.
- `src-ui/src/i18n/types.ts` & `src-ui/src/i18n/locales/*.ts` -- Translation keys for `standbyMode` and `standbyModeTooltip`.
- `src-ui/src/test/DriverOnboarding.test.tsx` & `SettingsModal.test.tsx` -- UI test coverage.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-wasapi/src/stream_pump.rs` -- In `run_windows_pump`, if `!config.force_loopback && capture_device_opt.is_none()`, loop in standby without capturing audio.
- `src-tauri/src/lib.rs` -- Guard pump startup in `AppState::with_options`, `apply_audio_bridge_mode`, `set_audio_device`, `trigger_device_fallback`, and `set_buffer_guard` with `should_start_pump = force_loopback || virtual_id.is_some()`.
- `src-ui/src/i18n/types.ts` & locales (`en.ts`, `pt-BR.ts`, `es.ts`) -- Add `standbyMode` and `standbyModeTooltip` strings.
- `src-ui/src/components/StatusBar.tsx` -- Add `isStandby` prop and render Standby badge when driver is missing in auto mode.
- `src-ui/src/App.tsx` -- Supply `isStandby={isDriverMissing && audioBridgeMode !== "loopback"}` to `StatusBar`.
- Unit tests in `crates/open-boom-wasapi/tests/stream_pump_tests.rs`, `src-tauri/src/lib.rs`, and `src-ui/src/test/`.

**Acceptance Criteria:**
- Given OpenBoom boots with `audio_bridge_mode = "auto"` and no virtual driver, when launched, then the audio pump remains in Standby without capturing or playing loopback audio.
- Given the application is in Standby, when the user explicitly switches to `"loopback"`, then Mode B loopback is started.
- Given the application is in Standby, when the virtual driver is installed, then Mode A is started.

## Spec Change Log

- Initial draft created for Story 9-6 disarmed standby state.
- Implementation completed across backend and frontend.

## Review Triage Log

- Self-review / verification against intent contract:
  - Disarmed Standby state verified: when virtual device is absent and loopback not forced, audio pump is not instantiated (`pump == None`) and worker enters standby loop.
  - Windows default endpoint is not modified to prevent unintended audio muting or redirection.
  - UI accurately shows `Standby (Driver Required)` badge instead of false loopback status.
  - Unit tests verify deterministic session isolation, pump standby, and status bar rendering.

## Verification

**Commands:**
- `cargo test --workspace` -- Passed (all unit and integration tests passing).
- `node src-ui/src/test/run.mjs` -- Passed (118 UI tests passing, 0 failures).
- `npm --prefix src-ui run build` -- Passed (TypeScript compilation and Vite bundle created).
