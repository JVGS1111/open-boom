# Epic 5 Context: Per-Application Audio Session Mixer

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

The user can open a dedicated per-application mixer drawer to view all active Windows audio sessions with process icons, adjust individual application volumes independently up to 150%, mute specific apps, and persist custom volume preferences across reboots without COM apartment threading conflicts or audio glitches.

## Stories

- Story 5.1: Windows Core Audio Session Manager & Dedicated COM MTA Thread
- Story 5.2: Per-Application Mixer Drawer UI & Individual Process Faders
- Story 5.3: Persistent Application Volume Preferences & Automatic Session Recovery

## Requirements & Constraints

- Dedicated MTA Worker Thread: Windows Core Audio `IAudioSessionManager2`, `IAudioSessionEnumerator`, and `ISimpleAudioVolume` must execute strictly on a dedicated COM worker thread initialized with `CoInitializeEx(None, COINIT_MULTITHREADED)` to prevent `RPC_E_CHANGED_MODE` or thread deadlock with Tokio/Tauri runtimes (`AD-8`, `FR21`).
- Real-Time Session Discovery: Audio sessions must be discovered within 1 second of starting playback and cleanly removed when the process exits or terminates (`FR21`).
- Independent Volume & Muting: Each active session must support volume adjustment from 0% to 150% and toggle mute via `ISimpleAudioVolume::SetMasterVolume` and `SetMute` without modifying Windows global master volume or other processes (`FR22`).
- Safe Process Inspection: Never inject code, DLLs, or hooks into external process memory. Process information (executable name, PID, icon) is queried safely via standard Windows APIs (`AD-8`).
- Application Stream Mixer Drawer: Slide-out drawer accessible from the HUD header (`UX-DR9`, `EXPERIENCE.md §2`), displaying each app row with process icon, name, playback activity indicator, fader, and mute button.
- Preference Persistence: Per-application custom volumes and mute states are saved atomically to `%APPDATA%\OpenBoom\app_volumes.json` and restored automatically when processes initiate audio sessions (`FR23`, `AD-10`).
- Synthetic/Headless Fallback: When running in headless tests or environments without audio hardware, session management must gracefully provide deterministic synthetic sessions to allow 100% test automation (`AD-8`).

## Technical Decisions

- Architecture Separation: Core Audio COM logic resides in `crates/open-boom-wasapi/src/session_manager.rs`. Persistence logic resides in `crates/open-boom-storage`. UI components reside in `src-ui/src/components/AppMixerDrawer.tsx`.
- Communication Pattern: Tauri IPC commands (`get_audio_sessions`, `set_app_volume`, `set_app_mute`) communicate asynchronously with the backend, which proxies requests to the dedicated MTA thread via message channels (`std::sync::mpsc`).
- Real-Time Session Updates: Session manager emits changes or polls active sessions without blocking the audio callback or UI rendering loop.
- Atomic File Persistence: `crates/open-boom-storage` uses `.tmp` staging file followed by atomic rename (`MoveFileExW`) to safeguard application volume preferences against corruption.

## UX & Interaction Patterns

- Drawer Activation: Triggered via "App Volumes" button in the window header or shortcut `Ctrl + Shift + M`.
- Row Layout: 44px height row (`app-mixer-row`), process icon (24x24px), process name, pulsating green dot for active playback, horizontal volume slider (0% to 150%), and mute button with red strikeout when muted.
- Keyboard Navigation: Full ARIA accessibility with slider keyboard control (Arrow Left/Right, Home/End) and Enter/Space for mute.

## Cross-Story Dependencies

- Story 5.1 provides the underlying `AudioSessionManager` MTA engine and Tauri IPC endpoints consumed by Story 5.2.
- Story 5.2 delivers the frontend UI and controls for managing volumes and mute states.
- Story 5.3 integrates atomic persistence in `crates/open-boom-storage` and ties session restoration into `AudioSessionManager` when new sessions appear.
