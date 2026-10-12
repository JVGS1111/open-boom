---
title: 'In-App Auto-Update Mechanism with GitHub Releases & Settings Controls'
type: 'feature'
created: '2026-10-12'
baseline_revision: '3f29696c34c6419364049170a04866c532ed3e1b'
status: 'done'
review_loop_iteration: 1
followup_review_recommended: false
context: []
warnings: []
deferred:
  - "Cross-platform Linux/macOS binary updater helpers (deferred to Epic 7 multi-platform rollout; Windows NSIS silent installer and Portable executable fallback implemented)"
  - "Cryptographic signature verification of downloaded installer assets against SHA256SUMS.txt (deferred; HTTPS GitHub Releases TLS transport is enforced)"
---

<intent-contract>

## Intent

**Problem:** Users currently have no convenient, in-app way to know when a new version of OpenBoom is available on GitHub Releases, requiring manual web checks and manual installer downloads.

**Approach:** Implement a zero-cost update checking and installation system integrated with GitHub Releases. Check for updates on application startup (if enabled), show a distinct update badge button in the Titlebar header when an update is available that links to Settings -> Updates, and provide a full update management panel in Settings with manual check, release info display, auto-check toggle, auto-install switch, progress indicators, and an instant application restart action upon update completion.

## Boundaries & Constraints

**Always:**
- Use public GitHub Releases endpoints (`https://api.github.com/repos/JVGS1111/open-boom/releases/latest` or Tauri release assets) without requiring paid servers or services.
- Provide a responsive update indicator button in the header Titlebar only when a newer version exists (`is_available == true`).
- Clicking the Titlebar update button directly opens Settings with the Updates view active (`onOpenSettings("updates")`).
- In Settings, provide:
  1. Current version vs Latest version display with release notes / published date.
  2. "Check for Updates" / "Download & Install" / "Restart Application" action button.
  3. Toggle: "Check for updates on application startup" (persisted in `AppConfig`).
  4. Toggle: "Automatically install updates when available" (persisted in `AppConfig`).
- When automatic update on startup finishes downloading/installing, display a clear "Restart Application" button in the update action slot so the user can relaunch at their convenience.
- Persist updater preferences atomically in `AppConfig` (`%APPDATA%\OpenBoom\config.json`) with safe defaults (`check_updates_on_startup: true`, `auto_install_updates: false`).
- Support headless and offline environments gracefully without uncaught exceptions or UI freezing.
- Provide full i18n localization parity across English (`en`), Portuguese (`pt-BR`), and Spanish (`es`).

**Never:**
- Block application startup or audio processing with network requests. Update checks must be asynchronous and non-blocking.
- Introduce paid services, third-party backend proxies, or analytics trackers.
- Force-restart the application while the user is actively listening without explicit user interaction or confirmation.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| App opens with `check_updates_on_startup: true` | Startup check finds newer version (e.g., v0.4.0 > v0.3.0) | Sets `hasUpdate: true`; Titlebar renders neon update badge; updates state cached in UI | Network errors logged quietly without alerting or blocking |
| App opens with `check_updates_on_startup: false` | Startup event triggers | Skips initial check entirely | No network requests initiated |
| User clicks Titlebar update badge | User clicks badge with neon icon | Opens Settings modal with Updates tab focused | Smooth transition |
| Manual check finds no new version | User clicks "Check for Updates" in Settings | Status displays "OpenBoom is up to date (v0.3.0)" | Friendly inline status |
| Manual check finds new version | User clicks "Check for Updates" | Status displays new version, release date, and "Download & Install" button | Display release notes safely |
| Auto-install enabled on startup | App starts, finds new version, and auto-downloads | Downloads in background; when ready, updates button to "Restart Application" | Failures display retry prompt |
| Offline / Network failure | No internet connectivity during check | Status displays "Unable to reach update server" with a "Retry" button | No crash, non-blocking |

</intent-contract>

## Code Map

- `crates/open-boom-storage/src/preset_store.rs` -- Stores updater preferences in `AppConfig` (`check_updates_on_startup: bool`, `auto_install_updates: bool`) with atomic JSON persistence.
- `src-tauri/src/lib.rs` -- Exposes Tauri IPC commands for updater checks, fetching latest release info, launching update download, and restarting the app.
- `src-ui/src/i18n/types.ts` -- Types for update dictionary keys in `TranslationDictionary`.
- `src-ui/src/i18n/locales/en.ts`, `src-ui/src/i18n/locales/pt-BR.ts`, `src-ui/src/i18n/locales/es.ts` -- Localized strings for all updater UI elements and states.
- `src-ui/src/components/Titlebar.tsx` -- Renders the update indicator button with distinct neon color when `hasUpdate` is true.
- `src-ui/src/components/SettingsModal.tsx` -- Renders the Updates section/tab with release notes, check button, switches, progress bar, and restart button.
- `src-ui/src/App.tsx` -- Manages updater state, performs non-blocking startup check, coordinates auto-install, and passes props down to Titlebar and SettingsModal.
- `src-ui/src/test/Updater.test.tsx` -- Unit tests validating update checking, UI state transitions, and component rendering.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-storage/src/preset_store.rs` -- Add updater configuration fields to `AppConfig` -- Persists user preferences for startup checking and auto-installation.
- `crates/open-boom-storage/tests/storage_tests.rs` -- Add storage test for updater preferences -- Ensures backward-compatible JSON deserialization.
- `src-tauri/src/lib.rs` -- Implement `check_for_updates`, `download_and_install_update`, and `restart_application` IPC commands -- Provides backend integration with GitHub Releases API.
- `src-ui/src/i18n/` -- Add updater translation keys across `types.ts`, `en.ts`, `pt-BR.ts`, and `es.ts` -- Maintains strict 100% dictionary parity.
- `src-ui/src/components/Titlebar.tsx` -- Add update notification badge button with distinct neon color -- Provides clear visual cue in header when update is available.
- `src-ui/src/components/SettingsModal.tsx` -- Implement Updates configuration panel with check button, toggles, progress bar, and restart action -- Allows user to inspect and control app updates.
- `src-ui/src/App.tsx` -- Wire update state machine, startup check, and navigation -- Connects frontend components to Tauri IPC.
- `src-ui/src/test/Updater.test.tsx` -- Add frontend test suite for update UI and settings -- Verifies all edge cases and states.

**Acceptance Criteria:**
- Given OpenBoom is opened and `check_updates_on_startup` is true, when a newer release exists on GitHub, then a distinct update badge button is displayed in the Titlebar header within 3 seconds of launch.
- Given the update badge is displayed in the Titlebar, when the user clicks the badge, then the Settings modal opens directly to the Updates section.
- Given the Updates section in Settings, when the user clicks "Check for Updates", then the application queries GitHub Releases and displays the latest version details.
- Given an update has been installed (either manually or via auto-install), then the action button changes to "Restart Application", allowing an immediate relaunch.
- Given `check_updates_on_startup` and `auto_install_updates` switches are toggled, then the preferences are atomically saved to `%APPDATA%\OpenBoom\config.json`.

## Spec Change Log

## Review Triage Log

### Iteration 1
- **[PATCH] PowerShell Console Window Flash (`CREATE_NO_WINDOW`)**: Subprocess spawns in `query_github_latest_release` and `download_and_run_installer` lacked `.creation_flags(0x08000000)`, causing brief console window flashes on Windows GUI starts. Fixed by adding `CommandExt::creation_flags(0x08000000)`.
- **[PATCH] PowerShell Error Exit Code**: `Write-Error` inside `catch` exited with code `0`, causing network errors to be misinterpreted as empty JSON. Fixed by using `[Console]::Error.WriteLine($_.Exception.Message); exit 1`.
- **[PATCH] SemVer Prerelease Comparison**: `is_version_newer` previously stripped suffixes after `-`, failing to recognize `0.3.0` as newer than `0.3.0-beta.1`. Fixed by comparing core tuple first and treating stable release as newer than prerelease of the same core version, plus adding unit test assertions.
- **[PATCH] Startup Check Re-Trigger on Config Toggle**: Toggling `checkUpdatesOnStartup` in Settings re-triggered the startup `useEffect`. Fixed by guarding the initial startup check with `hasRunStartupCheckRef`.
- **[PATCH] Hardcoded Strings in SettingsModal**: Replaced hardcoded `"General"` tab label and ready message fallback with localized `t.settings.generalTab` and `t.settings.updateReadyMessage` across `en`, `pt-BR`, and `es`.
- **[PATCH] Temp Installer Filename Collision**: Added `std::process::id()` to downloaded installer temp filename (`OpenBoom-Update-{pid}.exe`).
- **[DEFER] Cross-platform Linux/macOS updater implementation**: Deferred to Epic 7 multi-platform rollout.
- **[DEFER] SHA256SUMS signature verification**: Deferred; downloads occur strictly over HTTPS from official GitHub Releases API assets.

## Verification

**Commands:**
- `cargo test --workspace` -- passed (194+ unit & integration tests green).
- `npm --prefix src-ui test` -- passed (104 frontend unit tests green across 11 suites).
- `npm --prefix src-ui run build` -- passed (Vite + TypeScript build succeeded in 1.81s).

## Auto Run Result

- **Outcome:** Completed (`status: done`)
- **Summary:** Implemented zero-cost in-app auto-update checking and installation via GitHub Releases (`check_for_updates`, `download_and_install_update`, `restart_application`), added the amber pulsing update badge button in `Titlebar.tsx` that navigates directly to `Settings -> Updates`, built the tabbed `SettingsModal.tsx` with version comparison, release notes, startup check toggle, auto-install toggle, and unified action slot (`Check for Updates` / `Download & Install` / `Restart Application`), persisted preferences in `AppConfig`, and verified with full Rust and React test suites.


