---
title: 'Story 1.7: Application Preferences, Window Geometry & System Startup Persistence'
type: 'feature'
created: '2026-10-10'
baseline_revision: '2a888ff5bed652f527ef001641aa0628e7bfaa3e'
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

**Problem:** Listeners lose their customized volume levels, active playback devices, window screen positions, and DSP settings across application restarts, and have no way to configure automatic startup with Windows or control core playback from the system tray context menu.

**Approach:** Implement atomic JSON persistence in `crates/open-boom-storage` targeting `%APPDATA%\OpenBoom\config.json` via `.tmp` staging and `MoveFileExW` replacement; provide un-elevated `HKCU\...\Run` startup registry management; enhance the Windows system tray with a native context menu containing Master Power, Mute, Presets, and Exit; and build a dark fluent Settings modal in `src-ui`.

## Boundaries & Constraints

**Always:**
- Persist settings atomically to `%APPDATA%\OpenBoom\config.json` using a temporary `.tmp` file and `MoveFileExW` / atomic rename (`FR29`, `AD-10`).
- Manage Windows startup via `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` without requiring administrative elevation (`FR29`).
- Expose native system tray context menu on right-click providing toggles for Master Power, Mute, Presets, and Exit (`FR28`).
- Maintain zero heap allocation or blocking disk I/O on the real-time audio thread (`NFR2`).
- Provide keyboard accessibility (`Escape` key dismiss, focus trapping) in the Settings modal (`UX-DR11`).

**Never:**
- Never perform non-atomic in-place overwrites of `config.json` that could cause corrupt state on sudden power loss or process kill (`AD-10`).
- Never request administrative elevation (`UAC`) for user-level startup run keys (`FR29`).
- Never block the audio callback loop with storage or registry operations (`NFR1`, `NFR2`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Atomic Config Save | User changes volume or device endpoint | Configuration formatted as JSON written to `config.<pid>.tmp` and atomically replaced to `%APPDATA%\OpenBoom\config.json` | Returns `Result<(), String>`, preserves previous file on write error |
| Corrupt / Missing Config Recovery | `config.json` is missing or contains invalid JSON | Loads fallback `AppConfig::default()` cleanly without panic or crash | Recovers defaults and logs warning |
| Windows Startup Toggle ON | User enables "Launch on Windows Startup" in Settings modal | Adds `OpenBoom` value with executable path into `HKCU\...\Run` without UAC prompt | Returns clear error if registry write fails |
| Windows Startup Toggle OFF | User disables startup launch | Removes `OpenBoom` value from `HKCU\...\Run` cleanly | Handles non-existent key gracefully |
| Tray Context Menu Interaction | User right-clicks system tray icon | Native menu displays Open, Master Power, Mute, Presets, and Exit; clicking toggles state via dispatcher | Handled via Tauri tray menu event |
| Settings Modal Navigation | User clicks Settings icon in titlebar or presses `Escape` | Modal opens smoothly with focus trapped; `Escape` closes modal | Pure React state toggle |

</intent-contract>

## Code Map

- `crates/open-boom-storage/src/preset_store.rs` -- Implements atomic JSON configuration persistence with `AppConfig`, `PresetStore`, and `%APPDATA%` path resolution.
- `crates/open-boom-storage/src/startup.rs` -- Implements Windows Current User startup run key (`HKCU\...\Run`) management with mock/test mode.
- `crates/open-boom-storage/tests/storage_tests.rs` -- Unit tests for atomic `.tmp` swap, corrupted JSON recovery, and startup registry key contracts.
- `src-tauri/src/lib.rs` -- Exposes IPC commands for config loading/saving, startup toggling, and builds enhanced system tray context menu.
- `src-ui/src/components/SettingsModal.tsx` -- Dark fluent modal component with startup toggle, storage info, and keyboard accessibility.
- `src-ui/src/components/Titlebar.tsx` -- Adds Settings gear button to header controls.
- `src-ui/src/App.tsx` -- Wires configuration restore on mount and auto-save on user settings change.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-storage/src/preset_store.rs` -- Extend `AppConfig` and implement atomic `.tmp` write and replace with `%APPDATA%\OpenBoom\config.json` path resolution -- Delivers `FR29` and `AD-10`.
- `crates/open-boom-storage/src/startup.rs` -- Implement `StartupManager` for un-elevated `HKCU\...\Run` registry keys -- Delivers `FR29`.
- `crates/open-boom-storage/tests/storage_tests.rs` -- Unit test atomic configuration save/load, default fallback, and startup key management -- Verifies persistence edge cases.
- `src-tauri/src/lib.rs` -- Register IPC handlers (`load_app_config`, `save_app_config`, `get_launch_on_startup`, `set_launch_on_startup`) and enhance tray context menu (`FR28`) -- Bridges backend storage to desktop shell.
- `src-ui/src/components/SettingsModal.tsx` -- Build Settings modal with startup toggle, path badge, and keyboard dismiss -- Delivers `FR29` settings interface.
- `src-ui/src/components/Titlebar.tsx` -- Add Settings button opening `SettingsModal` -- Provides UI entry point.
- `src-ui/src/App.tsx` -- Restore saved preferences on startup and persist changes -- Delivers seamless session continuity.

**Acceptance Criteria:**
- Given user adjustments to volume, output endpoint, and window position, when settings change, then configurations are atomically written to `%APPDATA%\OpenBoom\config.json` via a temporary file and replace, preventing corrupt state on unexpected shutdown (`FR29`, `AD-10`).
- Given the Settings modal, when the user toggles "Launch on Windows Startup", then a clean registry run key or startup shortcut is created without requiring administrative elevation (`FR29`).
- Given OpenBoom is minimized to the system tray, when the user right-clicks the tray icon, then a native context menu appears providing quick toggles for Master Power, Preset selection, Mute, and Exit (`FR28`).

## Spec Change Log

<!-- Append-only. Populated by step-04 during review loops. -->

## Review Triage Log

<!-- Append-only. Populated by step-04 on EVERY review pass. -->

## Design Notes

- Atomic storage uses `MoveFileExW` / `fs::rename` from a process-tagged temporary file `config.<pid>.tmp` in the same directory. Because both files reside on the same filesystem volume, the rename operation is guaranteed to be atomic by NTFS and Windows VFS semantics.
- Startup registration targets `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`. User-mode registry access never invokes UAC elevation prompts.

## Verification

**Commands:**
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" check --workspace` -- expected: All workspace crates compile cleanly with zero errors and zero warnings.
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --workspace` -- expected: All tests pass including storage and startup unit tests.
- `npm --prefix src-ui run build` -- expected: Frontend builds successfully with zero TypeScript or Vite errors.
