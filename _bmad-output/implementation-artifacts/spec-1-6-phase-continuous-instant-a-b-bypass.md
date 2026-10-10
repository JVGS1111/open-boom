---
title: 'Story 1.6: Phase-Continuous Instant A/B Bypass'
type: 'feature'
created: '2026-10-10'
baseline_revision: 'd2e5dfa9cf6c36f58d14f499e2e4bc5cde65468f'
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

**Problem:** When listeners compare processed audio enhancements against raw uncolored sound, hard digital toggles cause audible clicks, DC pops, volume surges, and 2ms timing/phase jumps that disrupt critical listening evaluations.

**Approach:** Implement a 5ms equal-power crossfade ($\cos/\sin$) in `crates/open-boom-core::AudioProcessor` that routes around filter stages while preserving the terminal 2ms lookahead delay buffer and peak limiter; wire `Ctrl + Shift + B` global keyboard shortcut in `src-ui`; and update `StatusBar.tsx` to display `Hardware Passthrough (Bypassed)` in muted slate text.

## Boundaries & Constraints

**Always:**
- Enforce 5ms crossfade duration (`CROSSFADE_SAMPLES = 240` samples at 48,000 Hz) using an equal-power curve ($g_A = \cos(\alpha \cdot \frac{\pi}{2})$, $g_B = \sin(\alpha \cdot \frac{\pi}{2})$ where $g_A^2 + g_B^2 = 1.0$) (`FR4`, `AD-5`).
- Retain the terminal 2ms lookahead delay (`LIMITER_LOOKAHEAD_SAMPLES = 96`) across both processed and bypassed paths to guarantee zero phase jump and zero latency drift (`FR4`, `AD-5`, `UX-DR10`).
- Ensure zero heap allocations and zero lock contention on the real-time audio thread (`NFR1`, `NFR2`).
- Prevent clicks, DC pops, or volume surges when toggling bypass rapidly or repeatedly (`FR4`, `AD-5`).
- Display `Hardware Passthrough (Bypassed)` in muted slate text in the status bar when bypass is active (`EXPERIENCE.md §3`).
- Support toggling bypass via the `[A/B Bypass]` UI button and the global `Ctrl + Shift + B` shortcut (`EXPERIENCE.md §6.3`).

**Never:**
- Never perform hard abrupt sample slicing between processed and dry signals when master power is active (`FR4`, `AD-5`).
- Never introduce timing delay mismatches between processed and bypassed signal paths (`AD-5`).
- Never allocate memory, resize buffers, or execute blocking locks inside the real-time audio callback (`NFR2`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Instant A/B Bypass Toggle | Engine processing audio, user clicks `[A/B Bypass]` or presses `Ctrl + Shift + B` | $\alpha$ ramps from 0.0 to 1.0 over exactly 240 samples (5ms); audio transitions seamlessly via equal-power crossfade; 2ms lookahead delay preserved | Ramp clamped to [0.0, 1.0] |
| Instant Bypass Disengage | Engine bypassed ($\alpha = 1.0$), user clicks toggle | $\alpha$ ramps from 1.0 to 0.0 over 240 samples (5ms); processed audio restored seamlessly without clicks | Smooth ramp guaranteed |
| Rapid Chattering / Hammering Toggle | Bypass toggled 100 times in rapid succession within 20ms | Crossfade position smoothly reverses direction without discontinuity or click; zero pop or glitch | Bounded in-flight direction reversal |
| Status Bar Telemetry in Bypass | Bypass is engaged | Status bar displays `Hardware Passthrough (Bypassed)` in muted slate styling; limiter badge indicates bypass | Responsive UI reflection |
| Master Power Switched OFF | Master power toggled OFF while bypass is engaged or active | Direct bit-perfect passthrough takes precedence with 0.0% DSP CPU; on power-on, crossfade state is cleanly restored | Bit-perfect passthrough preserved |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/types.rs` -- Exports `CROSSFADE_SAMPLES` constant (240 samples = 5ms at 48 kHz).
- `crates/open-boom-core/src/dsp/pipeline.rs` -- Implements equal-power crossfader and delay-aligned dry signal routing in `AudioProcessor::process_interleaved`.
- `crates/open-boom-core/tests/pipeline_tests.rs` -- Unit tests verifying 5ms equal-power crossfade smoothness, 2ms delay preservation, and rapid toggle stability.
- `crates/open-boom-wasapi/src/coordinator.rs` -- Coordinates bypass state updates and telemetry.
- `src-ui/src/App.tsx` -- Registers `Ctrl + Shift + B` keyboard shortcut and wires bypass state to Tauri IPC dispatcher.
- `src-ui/src/components/StatusBar.tsx` -- Renders `Hardware Passthrough (Bypassed)` in muted slate text and updates bypass button visual states.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-core/src/types.rs` -- Define `CROSSFADE_SAMPLES: usize = 240` (5ms at 48 kHz) -- Establishes architectural constant for crossfade duration (`AD-5`).
- `crates/open-boom-core/src/dsp/pipeline.rs` -- Add `crossfade_pos: f32`, equal-power $\cos/\sin$ interpolation, and delay-aligned bypass processing in `AudioProcessor` -- Delivers click-free phase-continuous bypass with 2ms lookahead retention (`FR4`, `AD-5`).
- `crates/open-boom-core/tests/pipeline_tests.rs` -- Add unit tests for 5ms crossfade transition, equal-power continuity, and rapid toggle edge cases -- Validates acoustic safety and zero pops.
- `src-ui/src/App.tsx` -- Add `Ctrl + Shift + B` global hotkey listener -- Delivers `EXPERIENCE.md §6.3` keyboard shortcut requirement.
- `src-ui/src/components/StatusBar.tsx` -- Update status bar to render `Hardware Passthrough (Bypassed)` in muted slate styling when bypassed -- Delivers `EXPERIENCE.md §3` status readout.

**Acceptance Criteria:**
- Given OpenBoom is actively processing audio, when the user clicks the `[A/B Bypass]` button or presses `Ctrl + Shift + B`, then audio transitions between processed and unprocessed sound via a 5ms equal-power crossfade retaining the 2ms lookahead delay (`FR4`, `AD-5`, `UX-DR10`).
- Given the bypass state is toggled rapidly, when listening through sensitive headphones, then zero audible DC pops, clicks, or volume jumps occur (`FR4`, `AD-5`).
- Given bypass is engaged, when inspecting the status bar, then the indicator displays `Hardware Passthrough (Bypassed)` in muted slate text (`EXPERIENCE.md §3`).

## Spec Change Log

<!-- Append-only. Populated by step-04 during review loops. -->

## Review Triage Log

### 2026-10-10 — Review pass
- verdicts: 10 findings — high 1, medium 4, low 3, false 2, maybe-false 0
- findings:
  - `[high]` `[patch]` Tautological and unverified output audio mixing during crossfade in `test_crossfade_duration_and_equal_power_ramp` — Patched: configured non-unity volume (0.5), verified intermediate output sample levels and that dry audio emerges at unity level 0.5 upon full bypass in `crates/open-boom-core/tests/pipeline_tests.rs`.
  - `[medium]` `[patch]` Missing verification that uncolored dry signal bypasses volume attenuation when bypass is engaged — Patched: verified that an impulse injected at master volume 0.5 emerges at full uncolored level 0.8 in `test_bypass_retains_2ms_lookahead_delay_and_phase_alignment`.
  - `[medium]` `[patch]` Missing HTML `disabled` attribute and styling guard on A/B Bypass button when master power is switched off — Patched: added `disabled={!masterPower}` and conditional styling in `src-ui/src/components/StatusBar.tsx`.
  - `[medium]` `[patch]` Holding shortcut keys triggers repeated keydown events and unthrottled IPC command spamming — Patched: guarded with `if (e.repeat) return;` and `if (!masterPower) return;` in `src-ui/src/App.tsx`.
  - `[medium]` `[patch]` React state updater contains impure side-effect command dispatch — Patched: decoupled command dispatch from functional state updater in `src-ui/src/App.tsx`.
  - `[low]` `[patch]` Non-finite values at trailing odd sample in odd-length buffers — Patched: added finite check `if mix_val.is_finite()` before clamping in `crates/open-boom-core/src/dsp/pipeline.rs`.
  - `[low]` `[patch]` Separate `cos()` and `sin()` transcendental calls on real-time audio thread — Patched: optimized to single `(sin, cos) = theta.sin_cos()` in `crates/open-boom-core/src/dsp/pipeline.rs`.
  - `[low]` `[patch]` Telemetry hides limiter gain reduction during bypass mode — Patched: reported true `limiter.gain_reduction_db()` under master power regardless of bypass state.
  - `[false]` `[reject]` Sample rate fixed at 48 kHz in `CROSSFADE_SAMPLES` causes 2.5ms crossfade at 96 kHz — Refuted: by architectural invariant `AD-3`, the internal DSP pipeline operates at fixed 48 kHz float32; the resampler converts all external formats to 48 kHz.
  - `[false]` `[reject]` Bypass delay of 2ms lookahead violates "zero timing delays" — Refuted: relative differential delay between stream A and stream B must be exactly zero to prevent comb filtering and acoustic phase jumps during A/B switching (`AD-5`, Reading 2).

## Design Notes

- Equal-power crossfade law: $g_A = \cos(\alpha \cdot \frac{\pi}{2})$ and $g_B = \sin(\alpha \cdot \frac{\pi}{2})$. Because $\cos^2(\theta) + \sin^2(\theta) = 1$, the combined acoustic power remains invariant across the 5ms transition, avoiding perceived volume dips or surges.
- By routing dry uncolored audio into the `MasterLimiter` delay line alongside processed audio during the crossfade, both signals emerge after exactly $D = 96$ samples (2ms), completely eliminating phase jumps.

## Verification

**Commands:**
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" check --workspace` -- expected: Workspace compiles with zero errors and zero warnings.
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --workspace` -- expected: All unit and integration tests pass, including bypass crossfade tests.
- `npm --prefix src-ui run build` -- expected: Frontend builds successfully with zero TypeScript or Vite errors.
