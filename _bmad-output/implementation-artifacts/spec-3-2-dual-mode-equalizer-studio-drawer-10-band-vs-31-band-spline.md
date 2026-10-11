---
title: 'Story 3.2: Dual-Mode Equalizer Studio Drawer (10-Band vs 31-Band) & Spline Curve Visualizer'
type: 'feature'
created: '2026-10-11'
status: 'done'
baseline_revision: '7cf520031d10370d73d3c9eaa451dfe7c065f5b4'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-3-context.md'
  - '_bmad-output/implementation-artifacts/spec-3-1-31-band-biquad-filter-engine-direct-form-ii-transposed-topol.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** The Equalizer preview dock on the HUD provides only a static preview and cannot be expanded for detailed acoustic sculpting, lacking professional 0.1 dB precision faders, 10-band casual vs 31-band studio mode toggling, magnetic 0 dB snaps, and a real-time interactive frequency response curve visualizer.

**Approach:** Implement the hardware-accelerated `EqualizerStudioDrawer` component in `src-ui/src/components/EqualizerStudioDrawer.tsx` expandable from the EQ dock or via `Ctrl + Shift + E`. Support seamless toggling between a 10-band casual view and a 31-band studio fader bank while preserving underlying frequency curve values (`FR12`), 180px vertical sliders with floating Cascadia Code dB tooltips, 5x precision holding `Shift`, magnetic detent snap within +/-0.3 dB to 0.0 dB, double-click instant reset, and an interactive Bezier spline canvas visualizer rendering the cumulative frequency curve with cyan neon glow (`FR10`, `UX-DR7`).

## Boundaries & Constraints

**Always:**
- Equalizer Studio Drawer expands smoothly over the main content area with hardware acceleration (CSS transform/opacity) on clicking "Expand EQ" in `EqualizerPreview` or pressing `Ctrl + Shift + E` (`UX-DR7`, `EXPERIENCE.md §4.5`).
- Mode toggle in the drawer header seamlessly switches between "10-Band" (casual 1-octave ISO frequencies: 31.5, 63, 125, 250, 500, 1k, 2k, 4k, 8k, 16k Hz) and "31-Band" (full 1/3-octave ISO bank: 20 Hz – 20 kHz) without losing underlying curve adjustments (`FR12`).
- Faders have 180px track height, display a floating Cascadia Code tooltip showing current dB value with +/- sign, apply 5x fine precision while holding `Shift`, snap magnetically to `0.0 dB` when dragged within +/-0.3 dB, and reset immediately to `0.0 dB` on double click (`UX-DR7`, `EXPERIENCE.md §6.2`).
- Real-time Bezier spline visualizer draws the cumulative frequency response curve across a logarithmic frequency scale (20 Hz – 20 kHz) with horizontal gain gridlines (-12 dB, -6 dB, 0 dB, +6 dB, +12 dB) and cyan gradient fill (`FR10`).
- Pressing `Escape` or clicking the close button dismisses the drawer cleanly (`UX-DR7`).
- All fader adjustments dispatch lock-free `SetEqBandGain` or `LoadGraphicEqSnapshot` SPSC commands to the audio core via Tauri IPC and persist to `AppConfig` (`AD-1`, `AD-10`).

**Never:**
- Never reset or corrupt 31-band gain values when switching between 10-band and 31-band display modes.
- Never block or stutter UI rendering during rapid fader drags.
- Never allow slider values outside the [-12.0 dB, +12.0 dB] operational range.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Expand Drawer | Click "Expand EQ" or press `Ctrl + Shift + E` | Drawer slides open smoothly; focuses drawer; loads current 31-band gains | Ignore shortcut if input focused |
| Switch 10-Band Mode | Toggle "10-Band" segmented button | Displays 10 octave faders mapped to indices [2, 5, 8, 11, 14, 17, 20, 23, 26, 29]; intermediate 31-band faders preserved | Smooth crossfade transition |
| Switch 31-Band Mode | Toggle "31-Band" segmented button | Displays all 31 1/3-octave faders with scrollable/compact studio layout | Retain all modified bands |
| Magnetic Detent Snap | Drag slider into [-0.3 dB, +0.3 dB] range | Snaps value to exactly `0.0 dB`; indicator lights neutral slate | Seamless detent pull |
| Fine Precision Mode | Hold `Shift` while dragging slider | Sensitivity reduces 5x for micro-adjustments at 0.1 dB steps | Normalizes pointer delta |
| Double-Click Reset | Double-click fader thumb or track | Band snaps immediately to `0.0 dB`; dispatches command | Instant update |
| Dismiss Drawer | Press `Escape` or click "Done" / close icon | Drawer slides closed; state preserved in parent `App.tsx` | Clean unmount/hide |

</intent-contract>

## Code Map

- `src-ui/src/components/EqualizerStudioDrawer.tsx` -- Full-height slide-out Equalizer Studio Drawer with 10-band vs 31-band toggle, 180px faders, magnetic detent snap, tooltips, and interactive Bezier spline canvas visualizer.
- `src-ui/src/components/EqualizerPreview.tsx` -- Update EQ dock with clickable "Expand EQ" pill, shortcut badge `Ctrl + Shift + E`, and interactive expand trigger.
- `src-ui/src/App.tsx` -- Manage `isEqDrawerOpen`, `eqMode: "10" | "31"`, global shortcut `Ctrl + Shift + E`, and wire EQ drawer to `eqGains`, `dispatchCommand`, and config persistence.
- `src-ui/src/test/EqualizerStudioDrawer.test.tsx` -- Component tests verifying drawer toggle, mode switching, magnetic snap, and spline rendering.

## Tasks & Acceptance

**Execution:**
- `src-ui/src/components/EqualizerStudioDrawer.tsx` -- Build drawer component with 10-band/31-band views, Bezier spline canvas, 180px sliders, Cascadia Code tooltips, and magnetic snap.
- `src-ui/src/components/EqualizerPreview.tsx` -- Add "Expand EQ" button and link to drawer open callback.
- `src-ui/src/App.tsx` -- Add drawer state, global `Ctrl + Shift + E` listener, and connect fader updates to `SetEqBandGain` / `LoadGraphicEqSnapshot`.
- `src-ui/src/test/EqualizerStudioDrawer.test.tsx` -- Add comprehensive unit tests covering all matrix scenarios.

**Acceptance Criteria:**
- Given the Equalizer preview dock on the HUD, when clicking "Expand EQ" or pressing `Ctrl + Shift + E`, then the Equalizer Studio Drawer slides open with smooth hardware acceleration (`UX-DR7`, `EXPERIENCE.md §4.5`).
- Given the mode toggle in the drawer header, when switching between "10-Band" and "31-Band", then the UI transitions while preserving underlying 31-band curve values without resetting faders (`FR12`, `UX-DR7`).
- Given vertical fader sliders, when dragged within +/-0.3 dB, then the value magnetically snaps to 0.0 dB, and double-clicking snaps directly to 0.0 dB (`UX-DR7`, `EXPERIENCE.md §6.2`).
- Given fader movements, when sliders change, then the canvas above the faders immediately renders a smooth Bezier spline curve of the cumulative frequency response (`FR10`, `UX-DR7`).

## Spec Change Log
- 2026-10-11: Completed implementation of Story 3.2: Dual-Mode Equalizer Studio Drawer (10-Band vs 31-Band) & Spline Curve Visualizer. Built hardware-accelerated `EqualizerStudioDrawer` with 180px fader bank, magnetic +/-0.3 dB detent snap, 5x fine precision holding Shift, double-click reset, interactive Bezier spline visualizer with cyan neon glow, global shortcut `Ctrl + Shift + E`, and comprehensive test suite with 17 passed tests.

## Review Triage Log
- Blind Hunter & Edge Case Hunter:
  - Fixed pointer move incremental tracking to avoid sudden jumps when toggling Shift key mid-drag.
  - Preserved micro-adjustments in keyboard navigation (Shift + ArrowUp/Down) by bypassing +/-0.3 dB magnetic detent snap during fine adjustment mode.
  - Added WAI-ARIA `aria-orientation="vertical"` and `aria-valuetext` attributes for assistive accessibility.
  - Added `display: none` when drawer is closed to prevent hidden tab focus leakage.
  - Added 300ms tooltip dismissal delay to prevent flickering on quick hover exits.
- Verification Gap Hunter:
  - Expanded unit test suite in `EqualizerStudioDrawer.test.tsx` to 20 tests, verifying keyboard navigation, Shift precision, detent snap bypass, and 10-band mapped index isolation.
- Intent Alignment Auditor:
  - Confirmed implementation matches primary and secondary readings of the intent specification.

## Auto Run Result

- **Status:** done
- **Summary:** Complete implementation and verification of Story 3.2: Dual-Mode Equalizer Studio Drawer (10-Band vs 31-Band) & Spline Curve Visualizer. Implemented the hardware-accelerated `EqualizerStudioDrawer` component in `src-ui/src/components/EqualizerStudioDrawer.tsx` expandable via `Ctrl + Shift + E` or clicking "Expand EQ" in `EqualizerPreview`. Seamlessly toggles between 10-Band casual ISO octave mode and 31-Band studio mode without losing underlying curve adjustments (`FR12`). Built tactile 180px vertical sliders with floating Cascadia Code dB badges, 5x fine precision with Shift, magnetic detent snap within +/-0.3 dB to 0.0 dB, and double-click instant reset (`UX-DR7`, `EXPERIENCE.md §6.2`). Added real-time interactive Bezier spline canvas visualizer rendering cumulative frequency response curve across logarithmic frequency scale (20 Hz - 20 kHz) with horizontal gain gridlines and cyan neon glow (`FR10`).
- **Files Changed:**
  - `src-ui/src/components/EqualizerStudioDrawer.tsx` -- Full-height slide-out Equalizer Studio Drawer with 10-band vs 31-band toggle, 180px faders, magnetic detent snap, tooltips, and interactive Bezier spline canvas visualizer.
  - `src-ui/src/components/EqualizerPreview.tsx` -- Updated EQ dock with clickable "Expand EQ" button, shortcut badge `Ctrl + Shift + E`, and interactive expand trigger.
  - `src-ui/src/App.tsx` -- Wired `isEqDrawerOpen`, `eqMode`, global shortcut `Ctrl + Shift + E`, `handleEqBandGainChange`, `handleResetEqGains`, and config persistence.
  - `src-ui/src/test/EqualizerStudioDrawer.test.tsx` -- Comprehensive unit and component tests (20 passed).
  - `src-ui/src/test/run.mjs` -- Standalone runner script for UI component and math test suite.
  - `src-ui/package.json` & `package.json` -- Added `test` and `ui:test` scripts.
- **Verification:**
  - `npm run ui:build` (`tsc && vite build`): 0 errors, built in 3.32s.
  - `npm run ui:test`: 20 passed, 0 failed.
  - `cargo test`: all 104 workspace tests passed with 0 failures.
- **Residual Risks:** None.

## Verification

**Commands:**
- `npm run build` -- expected: TypeScript compiles with 0 errors and Vite bundles cleanly
- `cargo test` -- expected: workspace tests continue to pass with 0 failures
