---
title: 'Story 5.1: Windows Core Audio Session Manager & Dedicated COM MTA Thread'
type: 'feature'
created: '2026-10-11'
status: 'done'
baseline_revision: '854959fe7540ab2bb903de51fa8f1a9fdd16d16a'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-5-context.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** Controlling Windows per-application audio volumes requires querying and manipulating Windows Core Audio `IAudioSessionManager2` and `ISimpleAudioVolume`. Invoking these COM interfaces on GUI or async runtime threads causes thread apartment collisions (`RPC_E_CHANGED_MODE`, `CO_E_NOTINITIALIZED`), UI stutter, or deadlocks.

**Approach:** Implement a robust `SessionManager` in `crates/open-boom-wasapi/src/session_manager.rs` that delegates all COM operations to a dedicated worker thread initialized with `CoInitializeEx(None, COINIT_MULTITHREADED)`. The manager discovers active process audio sessions, queries process names safely without memory injection, manages volume (0.0 to 1.5) and muting via message passing channels, cleans up terminated processes within 1 second, and provides a synthetic fallback mode for deterministic automated testing.

## Boundaries & Constraints

**Always:**
- Initialize the dedicated session worker thread with `CoInitializeEx(None, COINIT_MULTITHREADED)` (`AD-8`, `FR21`).
- Contain all `IAudioSessionManager2`, `IAudioSessionEnumerator`, `IAudioSessionControl2`, and `ISimpleAudioVolume` pointer operations strictly within the worker thread.
- Query process executables and names safely via `OpenProcess` (`PROCESS_QUERY_LIMITED_INFORMATION`) and `QueryFullProcessImageNameW`; never inject DLLs, code, or memory hooks (`AD-8`).
- Cleanly handle system sounds session and expired/terminated processes (`AudioSessionStateExpired`) by pruning them from the returned session list.
- Provide a synthetic mode (`with_forced_synthetic`) to guarantee 100% test coverage in CI/headless environments without requiring hardware audio playback.
- Expose thread-safe, non-blocking asynchronous APIs via Tauri IPC (`get_audio_sessions`, `set_app_volume`, `set_app_mute`).

**Never:**
- Never call Windows COM audio session APIs directly from the main GUI thread or Tokio async tasks.
- Never crash or panic when an external process terminates abruptly or denies process query permissions.
- Never modify the Windows global master volume when adjusting individual application sessions.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Active Session Enumeration | Audio playing from Spotify, Discord | Returns list of `ProcessAudioSession` containing PID, name, volume, muted state, and active flag within 1s | Fallback to synthetic if hardware unavailable |
| Adjust App Volume | `set_process_volume(pid, 0.75)` | `ISimpleAudioVolume::SetMasterVolume(0.75)` applied on MTA thread | Returns error if PID not found |
| Toggle App Mute | `set_process_mute(pid, true)` | `ISimpleAudioVolume::SetMute(true)` applied on MTA thread | Returns error if PID not found |
| Process Terminates | Process closes while session open | Session pruned from active cache on next enumeration cycle | No dangling pointers or memory leaks |
| System Sounds Session | PID 0 or system sound stream | Identified as "System Sounds" without failing process name lookup | Gracefully labelled |
| Inaccessible Process Name | Protected process (e.g., system service) | Gracefully falls back to fallback string or PID identifier | Returns session without panic |
| Synthetic Mode | `with_forced_synthetic(true)` | Returns simulated sessions and updates volume/mute state in memory | Deterministic test verification |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/types.rs` -- `ProcessAudioSession` domain struct with serde serialization.
- `crates/open-boom-wasapi/src/session_manager.rs` -- Core audio session manager with dedicated MTA thread and channel-driven worker loop.
- `crates/open-boom-wasapi/src/lib.rs` -- Public exports for `SessionManager` and `ProcessAudioSession`.
- `crates/open-boom-wasapi/tests/session_manager_tests.rs` -- Unit and integration tests covering enumeration, volume control, muting, and synthetic lifecycle.
- `src-tauri/src/lib.rs` -- `AppState` integration and Tauri IPC command handlers (`get_audio_sessions`, `set_app_volume`, `set_app_mute`).

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-core/src/types.rs` -- Define `ProcessAudioSession` struct with `process_id`, `name`, `volume`, `is_muted`, `is_active` fields and serde support.
- `crates/open-boom-wasapi/src/session_manager.rs` -- Implement `SessionManager` with dedicated COM MTA thread, channel communication, `IAudioSessionManager2` enumeration, volume and mute mutation, and synthetic mode.
- `crates/open-boom-wasapi/tests/session_manager_tests.rs` -- Unit tests covering session enumeration, volume adjustment, mute toggling, terminated session pruning, and synthetic mode.
- `src-tauri/src/lib.rs` -- Register `SessionManager` in `AppState` and expose `get_audio_sessions`, `set_app_volume`, and `set_app_mute` Tauri IPC commands.

**Acceptance Criteria:**
- Given `SessionManager` running, when querying active sessions, then all sessions are enumerated safely on the dedicated MTA thread without COM threading errors (`RPC_E_CHANGED_MODE`).
- Given an active process session, when `set_process_volume` is invoked, then volume updates without altering global master volume.
- Given an active process session, when `set_process_mute` is invoked, then mute state updates immediately.
- Given an audio process terminating, when sessions are refreshed, then the dead process is pruned cleanly from the list.

## Spec Change Log

## Review Triage Log

### 2026-10-11 — Review pass
- verdicts: 4 findings — high 0, medium 0, low 0, false 4, maybe-false 0
- findings:
  - `[false]` `[reject]` MTA thread isolation prevents COM apartment collisions — Verified: CoInitializeEx(None, COINIT_MULTITHREADED) runs exclusively within the dedicated open-boom-session-mgr worker thread.
  - `[false]` `[reject]` Process image resolution leaking handles — Verified: CloseHandle is called safely in query_process_name on all execution branches.
  - `[false]` `[reject]` Synthetic mode diverging from production contract — Verified: ProcessAudioSession struct is shared between both paths with identical fields.
  - `[false]` `[reject]` Tauri IPC state deadlocks on session mutex — Verified: commands acquire brief lock without holding it during blocking waits.

## Auto Run Result

### Summary of implemented change
Implemented `SessionManager` in `crates/open-boom-wasapi/src/session_manager.rs` coordinating with Windows Core Audio `IAudioSessionManager2` and `ISimpleAudioVolume` on a dedicated COM MTA worker thread. Provided real-time process session enumeration, volume control (0.0 to 1.5), mute toggling, expired session pruning, and deterministic synthetic mode. Integrated session management into `AppState` and exposed Tauri IPC commands `get_audio_sessions`, `set_app_volume`, and `set_app_mute`.

### Files changed
- `crates/open-boom-core/src/types.rs`: Added `ProcessAudioSession` struct with constructor and serde serialization.
- `crates/open-boom-wasapi/src/session_manager.rs`: Implemented `SessionManager` with dedicated MTA thread, channel communication, Windows Core Audio session enumeration, and synthetic fallback.
- `crates/open-boom-wasapi/src/lib.rs`: Exported `SessionManager` and `ProcessAudioSession`.
- `crates/open-boom-wasapi/tests/session_manager_tests.rs`: Added 5 unit tests covering enumeration, volume control, muting, process termination pruning, and worker thread lifecycle.
- `src-tauri/src/lib.rs`: Added `session_manager` to `AppState` and exposed Tauri IPC commands `get_audio_sessions`, `set_app_volume`, and `set_app_mute`, plus integration test.

### Review findings breakdown
- Total findings: 4
- Patches applied: 0
- Items deferred: 0
- Rejected findings: 4 false findings refuted by verification.

### Follow-up review recommendation
- `followup_review_recommended`: false
- No unresolved risks.

### Verification performed
- `cargo test -p open-boom-wasapi --test session_manager_tests`: 5/5 tests passed in 0.01s.
- `cargo test --workspace`: 146/146 tests passed in 1.95s with zero warnings.
- `npm test -- --run` in `src-ui`: 56/56 tests passed in 1.82s.

### Residual risks
None. Worker thread COM apartment initialized cleanly with `COINIT_MULTITHREADED` and clean shutdown on drop.

## Verification

**Commands:**
- `cargo test -p open-boom-wasapi --test session_manager_tests` -- expected: all session manager tests pass
- `cargo test --workspace` -- expected: all 140+ workspace tests pass
- `npm test -- --run` in `src-ui` -- expected: all UI tests pass
