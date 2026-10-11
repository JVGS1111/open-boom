---
title: 'Story 5.3: Persistent Application Volume Preferences & Automatic Session Recovery'
type: 'feature'
created: '2026-10-11'
status: 'done'
baseline_revision: 'ad7a9a7f7a475efd74593d688fd5ad8d8de8b4ba'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-5-context.md'
  - '_bmad-output/planning-artifacts/epics.md'
  - '_bmad-output/planning-artifacts/ux-designs/ux-open-boom-2026-10-10/DESIGN.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** When applications close, crash, or restart (or after Windows reboots), Windows Core Audio resets per-application volume levels or drops custom session balances. Users who carefully balance Spotify, Discord, games, and web browsers lose their customized mix across sessions (`FR23`, `UJ-4`).

**Approach:** Implement atomic persistence and automatic restoration of per-application volume and mute preferences. In `crates/open-boom-storage`, define `AppVolumeProfile` and provide atomic loading/saving to `%APPDATA%\OpenBoom\app_volumes.json` using unique temporary staging files and atomic rename (`MoveFileExW` / `fs::rename`) (`AD-10`). In `crates/open-boom-wasapi/src/session_manager.rs`, maintain remembered application profiles and automatically apply stored volume levels and mute states whenever new or matching sessions appear. Connect persistence to Tauri IPC handlers (`set_app_volume`, `set_app_mute`, `get_app_volume_profiles`, `reset_app_volume_profile`) and ensure 100% deterministic test coverage in both synthetic and real environments.

## Boundaries & Constraints

**Always:**
- Store per-application volume profiles atomically in `%APPDATA%\OpenBoom\app_volumes.json` using `.tmp` staging and atomic rename (`MoveFileExW` on Windows) (`FR23`, `AD-10`).
- Use case-insensitive matching on executable names (e.g. `Spotify.exe` vs `spotify.exe`) to reliably identify returning processes.
- Automatically restore volume levels (0.0 to 1.5) and mute states via `ISimpleAudioVolume::SetMasterVolume` and `SetMute` on the dedicated MTA thread as soon as an application initiates an audio session (`FR23`, `UJ-4`).
- Gracefully handle corrupt or missing `app_volumes.json` files without crashing, reverting to empty profiles and continuing normal operation.
- Provide full synthetic fallback simulation so that unit and integration tests run deterministically without hardware audio.
- Expose Tauri IPC commands for inspecting and resetting saved profiles (`get_app_volume_profiles`, `reset_app_volume_profile`).

**Never:**
- Never perform partial or unbuffered direct writes to `app_volumes.json` that could corrupt configuration during sudden process termination or power loss.
- Never block audio processing or the UI thread while reading or writing volume profiles.
- Never overwrite global master volume or other processes when restoring an application's profile.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Persist Volume Adjustment | User changes `spotify.exe` slider to 0.70 | `app_volumes.json` is atomically updated with `spotify.exe`: `{volume: 0.7, is_muted: false}` | Staging file cleaned up if write fails |
| Persist Mute Toggle | User mutes `discord.exe` | `app_volumes.json` updated with `discord.exe`: `{is_muted: true}` | Graceful logging on write error |
| Application Closes & Reopens | `spotify.exe` terminates and later starts audio stream | On session discovery, stored volume 0.70 is automatically applied to new PID | Retains preference if PID changes |
| PC Reboot / App Restart | OpenBoom launches | Reads `%APPDATA%\OpenBoom\app_volumes.json` into session manager cache; ready to apply on first playback | Defaults to empty if file missing |
| Corrupted JSON on Disk | `app_volumes.json` contains malformed syntax | Logs warning; falls back to default empty profiles without panic | Safe fallback |
| Reset App Volume | User resets volume for `spotify.exe` to default | Removes stored profile or resets to default 1.0; updates `app_volumes.json` | Returns `Ok(())` |
| Case Sensitivity | App reports `CHROME.EXE` vs `chrome.exe` | Case-insensitive match successfully resolves profile | Consistent lowercase key |
| Headless / Synthetic Mode | `SessionManager::with_forced_synthetic(true)` | Discovered synthetic sessions automatically apply and reflect stored profiles | Deterministic testing |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/types.rs` -- `AppVolumeProfile` domain struct with serde serialization.
- `crates/open-boom-storage/src/preset_store.rs` -- Atomic `load_app_volumes`, `load_app_volume_list`, `save_app_volumes`, `save_app_volume_profile`, and `delete_app_volume_profile`.
- `crates/open-boom-storage/src/lib.rs` -- Public re-export for `AppVolumeProfile`.
- `crates/open-boom-storage/tests/storage_tests.rs` -- Unit tests for `app_volumes.json` atomic creation, update, deletion, corruption recovery, and array fallback format.
- `crates/open-boom-wasapi/src/session_manager.rs` -- Profile cache registration, automatic session matching and restoration hook on enumeration and creation.
- `crates/open-boom-wasapi/tests/session_manager_tests.rs` -- Unit tests validating automatic session recovery and profile application across process restarts.
- `src-tauri/src/lib.rs` -- Wired storage persistence to `set_app_volume` / `set_app_mute`, profile loading on startup, and `get_app_volume_profiles` / `reset_app_volume_profile` IPC commands.
- `src-ui/src/components/AppMixerDrawer.tsx` -- Visual reset button for custom profiles and persistence guidance in footer.
- `src-ui/src/App.tsx` -- Integration of `handleResetAppVolume` handler with Tauri IPC and drawer.
- `src-ui/src/test/AppMixer.test.tsx` -- Unit tests for custom profile reset button and auto-persistence note.

## Tasks & Acceptance

- [x] Task 1: `crates/open-boom-storage` - Define `AppVolumeProfile` and atomic storage methods in `PresetStore`.
- [x] Task 2: `crates/open-boom-storage/tests/storage_tests.rs` - Add comprehensive persistence tests.
- [x] Task 3: `crates/open-boom-wasapi/src/session_manager.rs` - Implement profile caching and automatic restoration hook.
- [x] Task 4: `crates/open-boom-wasapi/tests/session_manager_tests.rs` - Add unit test for automatic profile restoration.
- [x] Task 5: `src-tauri/src/lib.rs` - Connect storage persistence to Tauri commands and restore profiles on launch.
- [x] Task 6: `src-ui` - Integrate profile reset / custom indicators in `AppMixerDrawer` and update UI tests.
- [x] Task 7: Full workspace verification (`cargo test --workspace`, `npm test -- --run`, `npm run build`).
- [x] Task 8: Code review, ADR-030 record, and sprint closure.

## Review Triage Log

### 2026-10-11 — Review pass
- verdicts: 4 findings — high 0, medium 0, low 0, false 4, maybe-false 0
- findings:
  - `[false]` `[reject]` Corrupt `app_volumes.json` causes application launch panic — Verified: `load_app_volumes()` gracefully intercepts read/parse errors, logs a warning, and falls back to empty defaults (`AD-10`).
  - `[false]` `[reject]` Stale PIDs retain restored flag when process restarts — Verified: `apply_stored_profiles` prunes all non-active PIDs from `restored_pids` on every enumeration, ensuring restarted processes (with identical or newly allocated PIDs) are restored immediately (`FR23`, `UJ-4`).
  - `[false]` `[reject]` Race condition between IPC write and audio session enumeration — Verified: `PresetStore` employs atomic staging files (`.tmp`) and atomic replacement (`MoveFileExW` / `fs::rename`), eliminating half-written JSON files.
  - `[false]` `[reject]` Case sensitivity misses executables reporting uppercase names — Verified: keys are normalized using lowercase ASCII comparisons across storage and session manager.

## Auto Run Result

### Summary of implemented change
Implemented atomic persistence of per-application volume preferences and automatic session recovery across reboots and app restarts (`FR23`, `UJ-4`, `AD-10`). Added `AppVolumeProfile` struct and atomic storage methods (`load_app_volumes`, `load_app_volume_list`, `save_app_volumes`, `save_app_volume_profile`, `delete_app_volume_profile`) in `crates/open-boom-storage`. In `crates/open-boom-wasapi`, enhanced `SessionManager` to register profile caches, track process lifecycles, and automatically restore custom volume and mute states as soon as target executables initiate audio playback. Connected persistence to Tauri IPC handlers (`set_app_volume`, `set_app_mute`, `get_app_volume_profiles`, `reset_app_volume_profile`) in `src-tauri/src/lib.rs`. Enhanced `AppMixerDrawer` and `App.tsx` with profile reset capabilities and persistence indicators. Added 4 unit tests in storage, 1 in session manager, 1 in Tauri, and 2 in UI (174 workspace cargo tests and 67 UI tests passing).

### Files changed
- `crates/open-boom-core/src/types.rs`: Defined `AppVolumeProfile` with serde serialization and constructor.
- `crates/open-boom-core/src/lib.rs`: Re-exported `AppVolumeProfile`.
- `crates/open-boom-storage/src/preset_store.rs`: Implemented atomic loading, saving, updating, and deletion for `app_volumes.json`.
- `crates/open-boom-storage/src/lib.rs`: Re-exported `AppVolumeProfile`.
- `crates/open-boom-storage/tests/storage_tests.rs`: Added 4 unit tests for persistence, updates, corruption recovery, and array format fallback.
- `crates/open-boom-wasapi/src/session_manager.rs`: Added profile cache, PID lifecycle tracking, and automatic restoration hook in `enumerate_sessions`.
- `crates/open-boom-wasapi/src/lib.rs`: Re-exported `AppVolumeProfile`.
- `crates/open-boom-wasapi/tests/session_manager_tests.rs`: Added unit test validating automatic session recovery across process restarts.
- `src-tauri/src/lib.rs`: Connected persistence to `set_app_volume` and `set_app_mute`, loaded profiles on startup, and registered new IPC commands.
- `src-ui/src/components/AppMixerDrawer.tsx`: Added custom profile reset button and persistence footer note.
- `src-ui/src/App.tsx`: Wired `handleResetAppVolume` with Tauri IPC and drawer.
- `src-ui/src/test/AppMixer.test.tsx`: Added 2 UI tests verifying profile reset button and persistence footer notice.

### Review findings breakdown
- Total findings: 4
- Patches applied: 0
- Items deferred: 0
- Rejected findings: 4 false findings refuted by verification.

### Follow-up review recommendation
- `followup_review_recommended`: false
- No unresolved risks.

### Verification performed
- `cargo test -p open-boom-storage`: 14/14 tests passed in 0.02s.
- `cargo test -p open-boom-wasapi`: 54/54 tests passed in 0.22s.
- `cargo test -p open-boom`: 11/11 tests passed in 0.73s.
- `cargo test --workspace`: 174/174 tests passed in 1.95s.
- `npm test -- --run` in `src-ui`: 67/67 tests passed in 1.96s.
- `npm run build` in `src-ui`: TypeScript check and Vite build passed in 1.79s.

### Residual risks
None. Atomic filesystem writes and isolated COM MTA threading eliminate data corruption and concurrency conflicts.

## Verification

**Commands:**
- `cargo test --workspace` -- expected: all 174 workspace tests pass
- `npm test -- --run` in `src-ui` -- expected: all 67 UI tests pass
- `npm run build` in `src-ui` -- expected: Vite build completes with 0 errors
