---
title: 'Story 4.3: Fidelity Psychoacoustic Harmonic Exciter Module'
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

**Problem:** Compressed streaming audio (YouTube, Spotify, Bluetooth SBC) suffers high-frequency loss and dullness above 8–10 kHz, lacking presence and harmonic sparkle.

**Approach:** Implement a psychoacoustic harmonic exciter in `crates/open-boom-core/src/dsp/fidelity.rs`. Isolate high-frequency energy with a dedicated high-pass filter (> 3 kHz), apply non-linear polynomial waveshaping ($x^2$ for even 2nd harmonics and $x^3$ for odd 3rd harmonics), remove DC offsets, apply high-shelf air enhancement, and blend the synthesized overtones into the dry signal proportional to `amount` (0.0 to 1.0). Ensure 0.0ms algorithmic latency and zero heap allocations (`FR17`, `NFR1`, `NFR2`).

## Boundaries & Constraints

**Always:**
- Process stereo frames at 48 kHz with zero heap allocations and 0.0ms delay.
- Synthesize both 2nd (even) and 3rd (odd) harmonic overtones specifically in the upper-mid and high frequencies (> 3 kHz) so bass/mids remain uncolored.
- High-pass / DC blocking filter to prevent DC drift from quadratic terms ($x^2$).
- Linear parameter interpolation for `amount` to eliminate zipper noise.
- Smooth saturation / soft limiting to prevent harsh sibilance or aliasing blowup (`NFR9`).
- Full state reset on `reset()`.

**Never:**
- Never introduce muddy distortion into low or midrange fundamentals (< 2 kHz).
- Never introduce unbounded output or digital clipping.
- Never allocate heap memory on the audio thread.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output |
|----------|---------------|-----------------|
| Pure Low Sine (100 Hz) | 100 Hz sine wave | Output contains strictly 100 Hz (no distortion generated below crossover) |
| High Sine (5 kHz) | 5 kHz sine wave | Synthesizes subtle 10 kHz (2nd harmonic) and 15 kHz (3rd harmonic) presence |
| Disabled / 0% Amount | `enabled = false` or `amount = 0.0` | Output equals dry input bit-identically |
| Extreme Amplitude | Input peak at +10 dBFS | Waveshaper soft-saturates gracefully without numeric overflow or NaN |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/dsp/fidelity.rs` -- Psychoacoustic harmonic exciter module.
- `crates/open-boom-core/src/dsp/pipeline.rs` -- Integration into Stage 4 of `AudioProcessor`.
- `crates/open-boom-core/tests/sweeteners_tests.rs` -- Harmonic analysis and frequency isolation tests.
