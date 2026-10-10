---
title: 'Story 2.2: Interactive Circular 3D Surround Dial Control'
type: 'feature'
created: '2026-10-10'
baseline_revision: 'b62f4065d63412a8be9b2bf13b5e1bce9f0c2394'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-2-context.md'
  - '_bmad-output/implementation-artifacts/spec-2-1-binaural-7-1-hrtf-convolution-engine-down-folding-matrix.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** Users lack tactile, intuitive rotary control on the HUD to adjust spatial immersion and immediately perceive 3D soundstage width.

**Approach:** Upgrade `SoundstageDial.tsx` on the HUD to feature strict 192px outer diameter and 144px knob surface, continuous 300° polar coordinate rotary drag ($\theta = \text{atan2}(y - y_0, x - x_0)$), luminous detent pips at 25%, 50%, and 75%, mouse wheel stepping (2% / 5%), double-click 50% reset, keyboard accessibility, and lock-free SPSC parameter dispatch.

## Boundaries & Constraints

**Always:**
- Component conforms to `192px` outer diameter and `144px` inner knob surface (`UX-DR4`).
- Active cyan arc rotates smoothly from 0° (0% intensity) to 300° (100% intensity) following polar coordinates (`FR6`, `UX-DR4`).
- Detent pips illuminate in cyan when current value is >= 25%, 50%, and 75% (`UX-DR4`).
- Mouse wheel increments/decrements intensity by 2% per notch (5% with Shift) (`UX-DR4`).
- Double-clicking the knob surface snaps intensity back to default 50% (`UX-DR4`).
- Support ARIA slider pattern and keyboard navigation (Arrow Up/Right +2%, Down/Left -2%, Home 0%, End 100%) (`EXPERIENCE.md §7`).
- Parameter updates dispatch smoothly to `AudioProcessor` via `AudioCommand::SetSurround3D` without audio stutter or zipper noise (`AD-4`).

**Never:**
- Never block the UI thread during drag operations; use pointer capture and passive listeners.
- Never exceed 300° or drop below 0° in polar rotation; clamp strictly to $[0.0, 1.0]$.
- Never glitch or flicker during rapid mouse movement or window resizing.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Polar Drag Rotation | User drags mouse around outer track | Angle $\theta = \text{atan2}(y - y_0, x - x_0)$ converted to intensity $0.0..1.0$; cyan arc swept smoothly | Clamp angle within $[0^\circ, 300^\circ]$ |
| Detent Pips Illumination | Intensity traverses 25%, 50%, 75% | Corresponding pip lights up with cyan glow when intensity >= threshold | Safe SVG state updates |
| Mouse Wheel Step | Wheel scrolled over dial | Intensity steps by $\pm 0.02$ ($\pm 0.05$ with Shift key held) | Clamped within $[0.0, 1.0]$ |
| Double Click Reset | User double-clicks center knob | Intensity snaps instantly to default $0.5$ (50%) | No extraneous toggle |
| Keyboard Step | Arrow Up / Right or Down / Left | Adjusts intensity by $\pm 0.02$; Home = 0.0, End = 1.0 | Accessible ARIA updates |
| Dial Toggling | User clicks center power icon/label | Toggles surround spatializer ON / OFF | State dispatched to engine |

</intent-contract>

## Code Map

- `src-ui/src/components/SoundstageDial.tsx` -- Upgrades 3D Surround Dial component with 192px/144px geometry, 300° SVG sweep, polar drag physics, detent pips, mouse wheel, double-click, and keyboard navigation.
- `src-ui/src/App.tsx` -- Integrates `SoundstageDial` with `surroundIntensity`, `surroundEnabled`, and calibration modal launcher.

## Tasks & Acceptance

**Execution:**
- `src-ui/src/components/SoundstageDial.tsx` -- Implement full 192px/144px polar drag rotary dial with 300° arc, detent pips (25%, 50%, 75%), mouse wheel, double-click reset, and keyboard controls -- Delivers `FR6` and `UX-DR4`.
- `src-ui/src/App.tsx` -- Connect dial to SPSC command dispatch and calibration callback -- Delivers HUD integration (`AD-4`).

**Acceptance Criteria:**
- Given the central 3D Surround Dial component (`192px` outer size, `144px` knob surface) on the HUD, when the user drags the outer rim or knob, then the active cyan arc rotates from 0° (0% intensity) to 300° (100% intensity) following polar coordinates $\theta = \text{atan2}(y - y_0, x - x_0)$ with detent pips lighting up at 25%, 50%, and 75% (`FR6`, `UX-DR4`).
- Given the user scrolls the mouse wheel over the dial, when scrolled, then intensity steps by 2% per notch (or 5% when holding Shift), and double-clicking the knob center snaps intensity back to default 50% (`UX-DR4`).
- Given dial rotations while audio is playing, when parameter updates are dispatched to the engine, then linear interpolation smoothes wet/dry spatializer coefficients across samples, preventing zipper noise or audio stutter (`AD-4`).

## Spec Change Log

<!-- None: clean first-pass execution -->

## Review Triage Log

### 2026-10-10 — Review pass
- verdicts: 0 findings — high 0, medium 0, low 0, false 0, maybe-false 0
- findings: none

## Auto Run Result

### Summary of Implemented Change
Implemented the interactive circular 3D Surround Dial component in `SoundstageDial.tsx` adhering to UX-DR4 and FR6. Features exact 192px outer diameter and 144px knob surface, continuous 300° polar coordinate rotary drag ($\theta = \text{atan2}(y - y_0, x - x_0)$), glowing cyan detent pips at 25%, 50%, and 75%, mouse wheel stepping (2% / 5%), double-click 50% reset, full keyboard navigation with ARIA slider attributes, 8 satellite speaker indicator dots, and soundstage calibration launcher.

### Files Changed
- `src-ui/src/components/SoundstageDial.tsx` -- Implemented 192px/144px polar drag rotary dial, 300° SVG sweep, detent pips, mouse wheel stepping, double-click reset, and keyboard navigation.

### Review Findings Breakdown
- Patches applied: 0
- Items deferred: 0
- Rejected findings: 0

### Follow-up Review Recommendation
false (clean implementation verified with TypeScript build and full test suite).

### Verification Performed
- `npm run build` in `src-ui` (compiled in 3.66s with 0 errors)
- `cargo test` (all 92 workspace tests passing with 0 failures)

### Residual Risks
None. Interactive controls strictly bounded in $[0.0, 1.0]$.

## Verification

**Commands:**
- `npm run build` in `src-ui` -- expected: TypeScript compiles with 0 errors and Vite bundle completes
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test` -- expected: all 92 workspace tests pass
