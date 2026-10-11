---
title: 'Story 4.4: Night Mode Dynamic Range Compressor (DRC)'
type: 'feature'
created: '2026-10-11'
status: 'done'
baseline_revision: 'HEAD'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/planning-artifacts/epics.md'
  - '_bmad-output/planning-artifacts/architecture/architecture-open-boom-2026-10-10/ARCHITECTURE-SPINE.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** High-dynamic-range cinema audio and action games contain explosive transients that wake family members or cause hearing strain, while quiet speech and whispers are too low to comprehend without continually riding the volume slider.

**Approach:** Implement a program-dependent soft-knee dynamic range compressor in `crates/open-boom-core/src/dsp/night_mode.rs`. Feature fast attack (< 5ms), smooth program-dependent release, and progressive threshold scaling (-0 dBFS down to -30 dBFS) driven by `amount` (0.0 to 1.0). Tame loud transients above -12 dBFS by up to 18 dB while applying clean upward dialogue boost (up to +6 dB) for low-level signals below -30 dBFS, preventing pumping artifacts while maintaining vocal intelligibility (`FR18`).

## Boundaries & Constraints

**Always:**
- Peak/RMS envelope detector with fast attack ($\tau_{att} \le 5\text{ms}$) and natural release ($\tau_{rel} \approx 80\text{ms} - 150\text{ms}$).
- Smooth soft-knee transition ($W = 6.0\text{ dB}$) around the compression threshold.
- Scale threshold from 0.0 dBFS down to -30.0 dBFS as `amount` increases from 0.0 to 1.0 (`AD-4`).
- Provide up to 18 dB attenuation on loud transients (> -12 dBFS) and up to +6 dB dialogue boost on low-level signals (< -30 dBFS) (`FR18`).
- Zero heap allocations on the audio thread (`NFR1`, `NFR2`).
- Envelope and gain state reset on `reset()`.

**Never:**
- Never produce audible pumping, breathing, or clicks during gain changes.
- Never amplify noise floor when input is silent.
- Never allocate heap memory on the audio thread.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output |
|----------|---------------|-----------------|
| Loud Explosion (+0 dBFS) | Peak input at 0 dBFS, amount = 1.0 | Fast attack compresses signal down by 12–18 dB without hard clipping |
| Quiet Whisper (-36 dBFS) | Low-level input at -36 dBFS, amount = 1.0 | Receives clean dialogue boost of +4 to +6 dB |
| Complete Silence | Input = 0.0 | Output = 0.0 (zero noise amplification) |
| Disabled / 0% Amount | `enabled = false` or `amount = 0.0` | Output equals input dry signal (unity gain) |
| Rapid Parameter Automation | Amount modulated dynamically | Gain envelope transitions smoothly without discontinuities |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/dsp/night_mode.rs` -- Night Mode soft-knee DRC processor.
- `crates/open-boom-core/src/dsp/pipeline.rs` -- Integration into Stage 4 of `AudioProcessor`.
- `crates/open-boom-core/tests/sweeteners_tests.rs` -- Unit tests for dynamic compression, dialogue boost, attack/release timings.
