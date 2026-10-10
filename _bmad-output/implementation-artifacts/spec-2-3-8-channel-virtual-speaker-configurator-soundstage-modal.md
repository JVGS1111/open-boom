---
title: 'Story 2.3: 8-Channel Virtual Speaker Configurator Soundstage Modal'
type: 'feature'
created: '2026-10-10'
baseline_revision: '43c4a54cbe54ea12eb9ee72fbfb7aa81335ddc86'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-2-context.md'
  - '_bmad-output/implementation-artifacts/spec-2-2-interactive-circular-3d-surround-dial-control.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** Users cannot isolate dialogue channels (Center), boost stealth cues (Rear Left/Right), or calibrate virtual speaker balance for their personal audio preferences without an interactive multichannel soundstage configurator.

**Approach:** Implement `SoundstageConfiguratorModal.tsx` displaying a top-down polar soundstage with a central listener icon, concentric orbit rings, and 8 virtual speaker nodes (FL, C, FR, SL, SR, RL, RR, LFE); support individual channel mute/unmute toggling with red strikethrough indicators, channel trim faders (+/-6.0 dB at 0.1 dB resolution), shortcut `Ctrl + Shift + S`, and atomic configuration persistence.

## Boundaries & Constraints

**Always:**
- Display top-down polar soundstage with central listener head and 8 virtual speaker nodes (`FR7`, `UX-DR5`).
- Clicking any speaker node toggles mute state; muted speakers render dimmed with red strikethrough indicators and immediately drop from the spatial convolution mix (`FR7`, `UX-DR5`).
- Speaker trim fader adjusts relative channel gain smoothly within +/-6.0 dB at 0.1 dB resolution (`FR7`).
- Pressing Esc, clicking the backdrop, or clicking Done cleanly dismisses the modal and saves channel configurations (`EXPERIENCE.md §2`).
- Support keyboard shortcut `Ctrl + Shift + S` to toggle the soundstage modal from anywhere in the application (`EXPERIENCE.md §6.3`).
- Send lock-free SPSC commands `SetSpeakerMute` and `SetSpeakerGain` to the audio engine immediately without audio stutter or buffer dropouts (`AD-1`, `AD-4`).

**Never:**
- Never block the UI or real-time audio thread during modal animation or fader interaction.
- Never exceed +/-6.0 dB trim limits or produce clipping distortion; clamp values safely.
- Never lose speaker configuration on modal close; persist states in `AppConfig`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Open Configurator Modal | Click "Calibrate Soundstage" or press `Ctrl + Shift + S` | Modal opens centered with dimmed backdrop; 8 speaker nodes rendered at standard polar coordinates | Safe focus capture |
| Toggle Speaker Mute | User clicks speaker node (e.g. Center) | Speaker active state toggles; node shows red strikethrough; `SetSpeakerMute` dispatched; channel drops from mix | Instant audio update |
| Adjust Channel Gain | User moves gain slider for Rear Left | Gain adjusts smoothly between -6.0 dB and +6.0 dB; `SetSpeakerGain` dispatched; live dB readout shown | Clamped to +/-6.0 dB |
| Reset All Channels | User clicks "Reset All Channels" | All mutes reset to false, all trims reset to 0.0 dB; commands dispatched; UI updates | Instant state reset |
| Modal Dismissal | User presses Esc or clicks Done / backdrop | Modal closes smoothly; channel trim and mute preferences persisted to storage | Clean unmount |
| Shortcut Toggle | User presses `Ctrl + Shift + S` while open | Modal closes immediately | Keyboard listener cleanup |

</intent-contract>

## Code Map

- `crates/open-boom-storage/src/preset_store.rs` -- Adds `speaker_muted`, `speaker_gains_db`, and `speaker_azimuths_deg` arrays to `AppConfig`.
- `src-ui/src/components/SoundstageConfiguratorModal.tsx` -- New component rendering top-down polar soundstage, 8 speaker nodes, mute toggles, +/-6 dB faders, and reset actions.
- `src-ui/src/components/SoundstageDial.tsx` -- Connects "Calibrate Soundstage" button to modal launcher.
- `src-ui/src/App.tsx` -- Manages `isConfiguratorOpen` state, registers global `Ctrl + Shift + S` shortcut, and dispatches speaker commands.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-storage/src/preset_store.rs` -- Extend `AppConfig` with speaker mute and trim arrays -- Enables persistence (`FR7`, `AD-10`).
- `src-ui/src/components/SoundstageConfiguratorModal.tsx` -- Create 8-channel polar soundstage modal with node toggles, faders, and animations -- Delivers `FR7` and `UX-DR5`.
- `src-ui/src/App.tsx` -- Wire configurator modal, `Ctrl + Shift + S` global listener, and IPC dispatch -- Delivers seamless HUD integration.

**Acceptance Criteria:**
- Given the 3D Surround Dial on the HUD, when the user clicks the speaker calibration icon, then a modal opens displaying a top-down polar soundstage with a central listener icon and 8 virtual speaker nodes (`UX-DR5`).
- Given any virtual speaker node (e.g. Center, Rear Left, LFE Subwoofer), when the user clicks the node, then its active state toggles; muted speakers render dimmed with a red strikethrough indicator and immediately drop from the spatial convolution mix (`FR7`, `UX-DR5`).
- Given an active speaker node, when the user adjusts its gain fader, then relative channel gain adjusts smoothly within +/-6.0 dB at 0.1 dB resolution (`FR7`).
- Given the configurator modal is open, when the user presses Esc, clicks the backdrop, or clicks Done, then the modal dismisses cleanly and saves channel trim configurations (`EXPERIENCE.md §2`).

## Spec Change Log

<!-- None: clean first-pass execution -->

## Review Triage Log

<!-- Empty until first review pass -->

## Verification

**Commands:**
- `npm run build` in `src-ui` -- expected: TypeScript compiles with 0 errors and Vite bundles successfully
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test` -- expected: all 92 workspace tests pass

**Auto Run Results:**
- `npm run build` in `src-ui`: Exit 0 (vite build built in 1.77s, 0 errors, 0 warnings).
- `cargo test`: 92 unit and integration tests passed across all crates (open-boom-lib, open-boom-core, open-boom-storage, open-boom-wasapi).

