---
title: 'Windows Master Volume & Media Keys Bidirectional Synchronization'
type: 'feature'
created: '2026-10-12'
baseline_revision: '3af140fd999fac01b9a97bb8aedb88e3ed8dcf97'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context: []
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** When OpenBoom is running, Windows system audio routes into the virtual audio device (`Boom Audio`). Because WASAPI loopback captures unattenuated stream buffers, the Windows taskbar master volume slider and keyboard multimedia keys (Vol+, Vol-, Mute) have zero effect on output audio.

**Approach:** Implement real-time bidirectional volume synchronization between the Windows Master Endpoint Volume API (`IAudioEndpointVolume`, `IAudioEndpointVolumeCallback`) and OpenBoom's DSP dispatcher and UI, ensuring seamless hardware media key and taskbar volume control with feedback loop suppression.

## Boundaries & Constraints

**Always:**
- Register `IAudioEndpointVolumeCallback` on the active Windows default audio playback endpoint on an isolated COM MTA thread.
- Intercept OS volume adjustments (taskbar slider, Windows Settings flyout, and physical keyboard media keys: Volume Up, Volume Down, Mute).
- Dispatch volume and mute events into OpenBoom's lock-free SPSC command queue and emit `windows-volume-changed` Tauri events to keep the UI slider in sync.
- Synchronize from OpenBoom to Windows: when the user drags the OpenBoom volume slider (0% to 100%), update Windows master volume via `IAudioEndpointVolume::SetMasterVolumeLevelScalar`.
- Prevent recursive notification ping-pong loops using a synchronization guard flag and COM event context check.
- Maintain OpenBoom 0%–150% volume boost scale: 0%–100% maps 1:1 with Windows scalar volume (0.0 to 1.0); 101%–150% clamps Windows volume at 1.0 while OpenBoom applies DSP preamp gain under brickwall limiter protection.
- Support synthetic fallback mode for deterministic unit tests and headless environments without COM hardware.
- Safely unregister callbacks and release COM resources on shutdown or device switch.

**Never:**
- Block the real-time audio stream pump or create COM MTA/STA threading conflicts (`RPC_E_CHANGED_MODE`).
- Trigger infinite notification loops between Windows Core Audio callbacks and OpenBoom UI sliders.
- Rely on external kernel drivers or administrative privileges (UAC).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| User changes Windows volume slider / media keys | Windows master volume changed to 45% (scalar 0.45) | Callback receives notification; OpenBoom sets DSP master volume to 0.45; UI slider updates to 45% | Clamp value to [0.0, 1.5] defensively |
| User mutes via Windows / Keyboard Mute key | Windows endpoint muted = true | OpenBoom dispatcher receives `SetMute(true)`; UI reflects muted state | Safe boolean propagation |
| User unmutes via Windows | Windows endpoint muted = false | OpenBoom dispatcher receives `SetMute(false)`; UI reflects active state | Safe boolean propagation |
| User changes OpenBoom volume to 80% | OpenBoom slider dragged to 80% | `IAudioEndpointVolume::SetMasterVolumeLevelScalar(0.80)` called; notification callback ignores echo | COM failure logged gracefully, UI remains responsive |
| User boosts OpenBoom volume to 135% | OpenBoom slider dragged to 135% | Windows master volume sets to 1.0 (100%); OpenBoom DSP master volume sets to 1.35 | Value clamped to 1.0 for Windows endpoint |
| User mutes from OpenBoom UI | OpenBoom mute button toggled | Windows endpoint set to Mute = true; feedback loop suppressed | Non-blocking COM dispatch |
| Audio device switch / fallback | Physical DAC switched in DeviceSelector | Volume sync rebinds to new active endpoint without crashing or memory leak | Graceful re-initialization within 500ms |
| Headless / synthetic mode | `force_synthetic: true` | Volume sync runs in mock mode returning deterministic test values | Zero COM errors or crashes |

</intent-contract>

## Code Map

- `crates/open-boom-wasapi/Cargo.toml` -- Configured with `windows-core = "0.58.0"` and `"implement"` feature.
- `crates/open-boom-wasapi/src/volume_sync.rs` -- Real-time `EndpointVolumeSync` coordinator with `IAudioEndpointVolumeCallback` implementation, MTA worker thread, lock-free atomic state caches, feedback suppression, and synthetic simulation mode.
- `crates/open-boom-wasapi/src/lib.rs` -- Exports `EndpointVolumeSync`.
- `crates/open-boom-wasapi/tests/volume_sync_tests.rs` -- Unit and integration tests covering callback notifications, feedback filtering, synthetic mode, and 1:1 vs boost mapping.
- `src-tauri/src/lib.rs` -- Integrates `EndpointVolumeSync` into `AppState`; forwards OS volume events to frontend via `windows-volume-changed` Tauri event; updates Windows endpoint when `dispatch_audio_command`, `set_master_volume`, or `set_mute` IPC is called; rebinds endpoint on master power toggle and device switch.
- `src-ui/src/App.tsx` -- Listens to `windows-volume-changed` event and updates master volume and mute state in React.
- `src-ui/src/components/Titlebar.tsx` -- Volume slider and mute toggle reflecting live Windows volume without jitter.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-wasapi/src/volume_sync.rs` -- Create `EndpointVolumeSync` with COM `IAudioEndpointVolumeCallback` and worker thread -- Enables real-time listening and control of Windows master audio endpoint volume.
- `crates/open-boom-wasapi/src/lib.rs` -- Re-export `EndpointVolumeSync` -- Makes volume synchronization accessible across crates.
- `crates/open-boom-wasapi/tests/volume_sync_tests.rs` -- Add comprehensive test suite -- Verifies volume mapping, mute toggling, feedback suppression, and synthetic fallback.
- `src-tauri/src/lib.rs` -- Wire `EndpointVolumeSync` to `AppState`, command handlers, and window event emitter -- Synchronizes OS volume changes with Tauri frontend and OpenBoom audio engine.
- `src-ui/src/App.tsx` -- Add Tauri event listener for `windows-volume-changed` -- Reflects Windows volume adjustments instantly in the user interface.

**Acceptance Criteria:**
- Given OpenBoom is running and Master Power is ON, when the user adjusts the volume slider in Windows or presses physical Volume Up/Down keyboard keys, then OpenBoom's DSP volume and the UI volume slider update to match within 50ms.
- Given OpenBoom is running, when the user adjusts the OpenBoom volume slider between 0% and 100%, then the Windows master volume slider updates to match without triggering an infinite notification loop.
- Given OpenBoom is running, when the user sets OpenBoom volume above 100% (up to 150%), then Windows master volume is clamped at 100% and OpenBoom applies DSP preamp boost.
- Given OpenBoom is running, when the user presses keyboard Mute or toggles mute in Windows, then OpenBoom reflects the muted state and silences output.

## Review Triage Log

### 2026-10-12 — Review pass
- verdicts: 10 findings — high 1, medium 5, low 4, false 0, maybe-false 0
- findings:
  - `[high]` `[patch]` Master power toggle switches Windows default device without rebinding `EndpointVolumeSync` listener — Patched `handle_master_power_toggle` in `src-tauri/src/lib.rs` to invoke `self.volume_sync.rebind_endpoint(None)`.
  - `[medium]` `[patch]` `test_dispatch_command_syncs_to_windows_endpoint` called `volume_sync` directly instead of unified command ingress — Extracted `AppState::dispatch_audio_command` and updated test to verify command dispatch end-to-end.
  - `[medium]` `[patch]` `init_endpoint_volume` treated `"default"` as a literal hardware GUID rather than default endpoint — Patched `init_endpoint_volume` in `volume_sync.rs` to treat `"default"` and empty string as `None` (default console endpoint).
  - `[medium]` `[patch]` Synchronous channel queries in `get_master_volume` and `is_muted` could block up to 500ms — Replaced with lock-free atomic caches (`cached_volume_bits` and `cached_muted`) updated on `OnNotify` and setters.
  - `[medium]` `[patch]` Missing Spanish (`es.ts`) translation parity for update keys — Added missing translation keys in `es.ts` and verified 100% dictionary parity.
  - `[medium]` `[patch]` Updater URL validation — Added GitHub origin check (`https://github.com/`) and isolated environment variable parameter passing in `download_and_install_update`.
  - `[low]` `[patch]` Unchecked `RegisterControlChangeNotify` result on endpoint rebind — Patched `Rebind` arm in `volume_sync.rs` to check registration result and fallback to synthetic state on failure.
  - `[low]` `[patch]` Dragging volume slider while muted cleared UI mute flag without dispatching `SetMute(false)` to audio engine — Patched `handleVolumeChange` in `src-ui/src/App.tsx` to dispatch `{ SetMute: false }`.
  - `[low]` `[patch]` `set_master_volume` IPC command lacked `NaN` sanitization — Patched `set_master_volume` in `src-tauri/src/lib.rs` to sanitize `NaN` to `0.0` and delegate to `dispatch_audio_command`.
  - `[low]` `[patch]` Tautological assertion in `test_version_comparison_logic` — Replaced with strict semantic version comparison assertions.

## Design Notes

### Bidirectional Feedback Loop Suppression
When OpenBoom calls `SetMasterVolumeLevelScalar`, Windows fires `IAudioEndpointVolumeCallback::OnNotify`. Without suppression, OpenBoom would treat this as an external OS change and re-dispatch, creating an infinite echo loop.
To prevent this:
1. `EndpointVolumeSync` maintains an atomic `is_syncing_to_windows: AtomicBool`.
2. When setting Windows volume from OpenBoom, `is_syncing_to_windows` is set to `true` before `SetMasterVolumeLevelScalar` and reset after.
3. In `OnNotify`, if `is_syncing_to_windows` is true, the event is ignored.
4. Additionally, a dedicated event context GUID is passed to `SetMasterVolumeLevelScalar(&OPEN_BOOM_VOLUME_CONTEXT_GUID)`: in `OnNotify`, if `pNotify.guidEventContext == OPEN_BOOM_VOLUME_CONTEXT_GUID`, the notification is confirmed as originating from OpenBoom and discarded.

### Boost Range Mapping
- Windows master volume range is strictly $0.0 \le v \le 1.0$ (0% to 100%).
- OpenBoom volume range is $0.0 \le v \le 1.5$ (0% to 150%).
- Direction OS $\to$ OpenBoom: $v_{\text{openboom}} = v_{\text{windows}}$.
- Direction OpenBoom $\to$ OS: $v_{\text{windows}} = \min(1.0, v_{\text{openboom}})$.

## Verification

**Commands:**
- `cargo test -p open-boom-wasapi --test volume_sync_tests` -- expected: All volume sync unit tests pass.
- `cargo test -p open-boom-core` -- expected: Core tests pass with 0 failures.
- `cargo test -p open-boom` -- expected: Tauri integration tests pass.
- `npm test --prefix src-ui` -- expected: Frontend unit test suite passes.
- `npm --prefix src-ui run build` -- expected: Production Vite and TypeScript compilation succeeds without errors.

## Auto Run Result

- **Status:** `done`
- **Summary of Implemented Change:** Implemented real-time bidirectional synchronization between Windows Master Endpoint Volume (`IAudioEndpointVolume` and `IAudioEndpointVolumeCallback`) and OpenBoom's audio engine and UI. Hardware media keys (Volume Up, Volume Down, Mute) and the Windows taskbar volume slider now control OpenBoom's volume in real time, and adjustments in OpenBoom's UI slider (0%–100%) synchronize back to Windows with feedback loop suppression (`OPEN_BOOM_VOLUME_CONTEXT_GUID` + atomic guard) and preamp boost support (101%–150%).
- **Files Changed:**
  - `crates/open-boom-wasapi/Cargo.toml` — Added `windows-core` and `"implement"` feature for COM interface implementation.
  - `crates/open-boom-wasapi/src/volume_sync.rs` — Implemented `EndpointVolumeSync` and `EndpointVolumeCallback` with MTA worker thread, lock-free atomic state caches, feedback suppression, and synthetic fallback.
  - `crates/open-boom-wasapi/src/lib.rs` — Exported `EndpointVolumeSync`, `VolumeSyncEvent`, and `OPEN_BOOM_VOLUME_CONTEXT_GUID`.
  - `crates/open-boom-wasapi/tests/volume_sync_tests.rs` — Added 9 unit and integration tests for volume synchronization.
  - `src-tauri/src/lib.rs` — Integrated `EndpointVolumeSync` into `AppState`, unified `dispatch_audio_command`, emitted `windows-volume-changed` events, and rebound on master power toggle and device switch.
  - `src-ui/src/App.tsx` — Added listener for `windows-volume-changed` and unmute-on-drag dispatch.
  - `src-ui/src/components/Titlebar.tsx` — Added NaN input sanitization and update notification action badge.
  - `src-ui/src/test/TitlebarVolumeSync.test.tsx` & `src-ui/src/test/run.mjs` — Added frontend test suite for volume sync rendering.
- **Review Findings Breakdown:** 10 findings (1 high, 5 medium, 4 low) — all 10 patched and verified; 0 deferred; 0 rejected.
- **Follow-up Review Recommendation:** `false` (all patched items are verified by deterministic unit tests with zero remaining unverified risks).
- **Verification Performed:**
  - `cargo test --workspace`: 212 passed, 0 failed.
  - `npm test --prefix src-ui`: 104 passed, 0 failed.
  - `npm --prefix src-ui run build`: Succeeded in 1.90s with 0 errors.
- **Residual Risks:** None.

