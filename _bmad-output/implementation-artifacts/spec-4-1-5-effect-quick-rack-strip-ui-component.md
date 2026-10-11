---
title: 'Story 4.1: 5-Effect Quick Rack Strip UI Component'
type: 'feature'
created: '2026-10-11'
status: 'done'
baseline_revision: 'HEAD'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/planning-artifacts/epics.md'
  - '_bmad-output/planning-artifacts/ux-designs/ux-open-boom-2026-10-10/DESIGN.md'
  - '_bmad-output/planning-artifacts/ux-designs/ux-open-boom-2026-10-10/EXPERIENCE.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** The desktop UI needs a tactile, high-contrast horizontal effects strip (`UX-DR6`) representing the 5 core psychoacoustic sweetening modules (Ambience, Fidelity, Night Mode, Spatial, and Pitch) allowing users to toggle each effect on/off, adjust depth mini-sliders with immediate percentage readouts, persist preferences across launches, and cleanly display Pitch as "Coming Soon" without breaking layout or invoking unimplemented DSP routines.

**Approach:** Build a dedicated `EffectsRackStrip` component in `src-ui/src/components/EffectsRackStrip.tsx` adhering to `DESIGN.md` and `EXPERIENCE.md`. Provide 5 modular cards styled with surface card background, cyan active borders (`#38bdf8`), subtle glow, tactile power toggle buttons, and responsive mini-sliders (0% to 100%). Bind controls to SPSC lock-free commands via Tauri IPC (`SetSweetener`), add keyboard accessibility (Space to toggle, Arrow keys to adjust), persist effect configurations in `AppConfig` via `crates/open-boom-storage`, and integrate the strip into the main HUD in `App.tsx`.

## Boundaries & Constraints

**Always:**
- Display exactly 5 modular cards in a horizontal layout: Ambience, Fidelity, Night Mode, Spatial, Pitch (`FR20`, `UX-DR6`).
- Cards follow `DESIGN.md` styling: `{colors.surface-card}` (`#1c1e24`), dark graphite borders (`#262933`), turning into `{colors.accent-cyan}` (`#38bdf8`) with cyan glow when enabled.
- Pitch card displays semitone transposition (`-12 st` to `+12 st`) with a Phase 2 "Coming Soon" badge (`FR20`, `PRD §6.2`).
- Sliders track 0% to 100% with immediate numeric readouts (`{typography.metric-mono}`).
- Support keyboard navigation (`role="slider"`, `aria-valuenow`, `aria-label`, Space/Enter toggles, Arrow Left/Right adjustments).
- Persist enabled state and intensity levels in `AppConfig` atomically (`AD-10`).

**Never:**
- Never crash or trigger DSP failures when interacting with the Pitch card.
- Never freeze or block the UI thread during slider adjustments.
- Never omit accessible ARIA attributes.

## I/O & Edge-Case Matrix

| Scenario | Input / Action | Expected Result |
|----------|---------------|-----------------|
| Toggle Effect Power | Click card toggle button or press Space on focused card | State flips between active (cyan border, glow) and bypassed (subtle border, dimmed text); dispatches `SetSweetener` POD command |
| Slider Adjustment | Drag mini-slider or use Arrow keys | Amount updates smoothly (0.00 to 1.00); monospace percentage badge reflects change; dispatches `SetSweetener` |
| Pitch Card Interaction | User views or interacts with Pitch card | Card shows "Coming Soon" badge; slider disabled; zero DSP calls dispatched |
| Window Resize | HUD resized from 780px to 920px | 5-card grid scales responsively without horizontal overflow or clipping |
| Persistence Restore | App launched | Previous effect states (enabled & amount) restored from `%APPDATA%\OpenBoom\config.json` |

</intent-contract>

## Code Map

- `src-ui/src/components/EffectsRackStrip.tsx` -- 5-card horizontal rack component.
- `src-ui/src/App.tsx` -- State management, command dispatching, and persistence integration.
- `crates/open-boom-storage/src/preset_store.rs` -- `AppConfig` fields for effect modules.
- `src-ui/src/test/EffectsRack.test.tsx` -- UI test suite for rendering, toggling, sliders, and accessibility.
