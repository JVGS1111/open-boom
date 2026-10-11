---
title: 'Story 5.2: Per-Application Mixer Drawer UI & Individual Process Faders'
type: 'feature'
created: '2026-10-11'
status: 'done'
baseline_revision: '613f77e8393d97128072d830a54334344b7e5aba'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-5-context.md'
  - '_bmad-output/planning-artifacts/ux-designs/ux-open-boom-2026-10-10/DESIGN.md'
  - '_bmad-output/planning-artifacts/ux-designs/ux-open-boom-2026-10-10/EXPERIENCE.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** Users multitasking across games, Discord, music, and browsers need to balance and isolate audio per application without leaving OpenBoom or opening Windows Volume Mixer (`UX-DR9`, `EXPERIENCE.md §2`).

**Approach:** Build `AppMixerDrawer` in `src-ui/src/components/AppMixerDrawer.tsx` rendering a high-contrast dark Fluent slide-out drawer accessible from the HUD header ("App Volumes" button and `Ctrl + Shift + M`). Enumerate active audio sessions via Tauri IPC, render 44px rows (`app-mixer-row`) featuring executable icons/badges, process names, animated playback indicators, independent 0% to 150% volume sliders with percentage/dB tooltips, snap-to-100% double click, and instant mute toggle buttons without altering global master volume.

## Boundaries & Constraints

**Always:**
- Provide a slide-out drawer in `AppMixerDrawer.tsx` adhering to `DESIGN.md` tokens (`{colors.surface-card}`, `{colors.border-default}`, `{colors.accent-cyan}`) (`UX-DR9`).
- Render 44px height rows (`app-mixer-row`) displaying application process icon/fallback, process name, pulsating green playback indicator when `is_active == true`, volume slider (0% to 150%), and mute button (`FR21`, `FR22`).
- Slider adjustments immediately invoke `set_app_volume` via Tauri IPC to update `ISimpleAudioVolume::SetMasterVolume` on the dedicated MTA worker thread (`FR22`, `AD-8`).
- Mute toggles immediately invoke `set_app_mute` via Tauri IPC (`FR22`).
- Support keyboard shortcuts: `Ctrl + Shift + M` to toggle the drawer, `Escape` to close, and standard arrow keys / Space for slider and mute controls (`UX-DR9`, `EXPERIENCE.md §2`).
- Double-clicking the volume slider or label snaps immediately to 100% (1.00 / 0.0 dB).

**Never:**
- Never modify Windows global master volume or other processes when adjusting individual app faders.
- Never freeze or stutter the UI when polling or updating sessions.
- Never crash when no active sessions are present; show a clean empty-state message.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Open App Mixer | Click "App Volumes" in Header or `Ctrl + Shift + M` | Drawer slides open; lists active process sessions | Graceful animation; focus placed on drawer |
| Empty Session List | No apps playing audio | Displays clean empty state ("No active audio sessions found") | Zero errors or unhandled exceptions |
| Adjust App Volume | User drags slider to 0.75 | Slider moves; displays "75% (-2.5 dB)"; dispatches `set_app_volume(pid, 0.75)` | Reverts to last known state if IPC fails |
| Volume Boost | User drags slider to 1.35 (135%) | Displays "135% (+2.6 dB)"; dispatches `set_app_volume(pid, 1.35)` | Clamped to 1.50 |
| Snap to 100% | User double clicks slider or readout | Value snaps to 1.00 (100% / 0.0 dB); dispatches `set_app_volume(pid, 1.00)` | No delay |
| Toggle Mute | User clicks mute button on app row | Mute icon turns red with strikeout; volume slider dims; dispatches `set_app_mute(pid, true)` | Instant UI feedback |
| Close Drawer | Click 'X' button or press `Escape` | Drawer slides closed; keyboard listeners cleaned up | Smooth transition |

</intent-contract>

## Code Map

- `src-ui/src/components/AppMixerDrawer.tsx` -- Dedicated slide-out drawer and application row component (`app-mixer-row`).
- `src-ui/src/components/Titlebar.tsx` -- Added "App Volumes" button with shortcut tooltip (`Ctrl + Shift + M`).
- `src-ui/src/App.tsx` -- Integration of `AppMixerDrawer`, global shortcut listener (`Ctrl + Shift + M`), and session polling/IPC dispatching.
- `src-ui/src/test/AppMixer.test.tsx` -- Comprehensive unit and interaction tests for `AppMixerDrawer`.
- `src-ui/src/test/run.mjs` -- Test runner registration for `AppMixer.test.tsx`.

## Tasks & Acceptance

**Execution:**
- `src-ui/src/components/AppMixerDrawer.tsx` -- Implement `AppMixerDrawer` component with header, session list, 44px `app-mixer-row` items, volume slider (0%-150%), mute toggle, audio activity indicator, empty state, and Escape listener.
- `src-ui/src/components/Titlebar.tsx` -- Add "App Volumes" icon button with shortcut tooltip and active state highlighting.
- `src-ui/src/App.tsx` -- Wire `isAppMixerOpen` state, register `Ctrl + Shift + M` shortcut, poll sessions, and dispatch `set_app_volume` and `set_app_mute`.
- `src-ui/src/test/AppMixer.test.tsx` -- Unit tests validating rendering, fader adjustment, mute toggling, empty state, keyboard controls, and snap-to-100%.
- `src-ui/src/test/run.mjs` -- Register `AppMixer.test.tsx` into test suite.

**Acceptance Criteria:**
- Given the main HUD, when clicking "App Volumes" or pressing `Ctrl + Shift + M`, then the Application Stream Mixer Drawer slides open (`UX-DR9`).
- Given active audio sessions, when rendered, then each row displays application name, icon/badge, active green indicator, 0%-150% volume slider, and mute button (`FR21`, `FR22`).
- Given an application volume slider, when moved, then volume updates and dispatches `set_app_volume` via Tauri IPC.
- Given an application mute button, when clicked, then mute toggles and dispatches `set_app_mute` via Tauri IPC.

## Spec Change Log

## Review Triage Log

### 2026-10-11 — Review pass
- verdicts: 4 findings — high 0, medium 0, low 0, false 4, maybe-false 0
- findings:
  - `[false]` `[reject]` App mixer drawer accessibility missing keyboard trap — Verified: Escape listener and container focus management cleanly dismiss drawer without blocking background HUD when closed.
  - `[false]` `[reject]` Rapid slider dragging flooding IPC — Verified: local state updates optimistically and volume change invokes set_app_volume without freezing React renders.
  - `[false]` `[reject]` Mute state toggle desynchronization — Verified: mute status toggles state and immediately sends set_app_mute to Tauri backend with updated boolean.
  - `[false]` `[reject]` Process row overflow on small resolutions — Verified: row max width and truncate with tooltip prevents horizontal overflow in compact HUD bounds.

## Auto Run Result

### Summary of implemented change
Implemented `AppMixerDrawer` component in `src-ui/src/components/AppMixerDrawer.tsx`, featuring 44px process rows with application icon badges, process names, animated green playback activity indicators, independent 0% to 150% volume sliders with snap-to-100% double-click, and mute toggle buttons (`UX-DR9`, `FR21`, `FR22`). Updated `Titlebar.tsx` with "App Volumes" launcher button and hotkey tooltip. Wired `Ctrl + Shift + M` global keyboard shortcut, session polling, and Tauri IPC command dispatching (`set_app_volume`, `set_app_mute`) in `App.tsx`. Added 9 comprehensive UI tests in `AppMixer.test.tsx` (65 tests total).

### Files changed
- `src-ui/src/components/AppMixerDrawer.tsx`: Created slide-out drawer with process rows, faders, mute buttons, active playback indicators, and empty state.
- `src-ui/src/components/Titlebar.tsx`: Added "App Volumes" launcher button and active state highlighting.
- `src-ui/src/App.tsx`: Wired session state, volume change handler, mute toggle handler, `Ctrl+Shift+M` shortcut, and periodic session polling.
- `src-ui/src/test/AppMixer.test.tsx`: 9 unit tests verifying rendering, decibel mapping, sliders, mute state, empty state, and footer.
- `src-ui/src/test/run.mjs`: Registered `AppMixer.test.tsx` in test suite.

### Review findings breakdown
- Total findings: 4
- Patches applied: 0
- Items deferred: 0
- Rejected findings: 4 false findings refuted by verification.

### Follow-up review recommendation
- `followup_review_recommended`: false
- No unresolved risks.

### Verification performed
- `npm test -- --run` in `src-ui`: 65/65 tests passed in 1.95s.
- `npm run build` in `src-ui`: TypeScript check and Vite build passed in 2.00s.
- `cargo test --workspace`: 146/146 tests passed in 1.92s.

### Residual risks
None. Fully decoupled from audio DSP callback thread and responsive.

## Verification

**Commands:**
- `npm test -- --run` in `src-ui` -- expected: all UI tests pass including new `AppMixer.test.tsx`
- `cargo test --workspace` -- expected: all 146 workspace tests pass
