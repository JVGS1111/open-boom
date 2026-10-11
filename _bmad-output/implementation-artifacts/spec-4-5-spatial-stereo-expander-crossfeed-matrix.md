---
title: 'Story 4.5: Spatial Stereo Expander & Crossfeed Matrix'
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

**Problem:** Standard stereo headphone mixes can feel narrowly localized "inside the head", while simple widening techniques wash out center vocals or destroy mono compatibility through severe phase cancellations.

**Approach:** Implement a Mid/Side stereo expander with frequency-selective crossfeed in `crates/open-boom-core/src/dsp/spatial.rs`. Decompose the incoming signal into Mid ($M = 0.5(L + R)$) and Side ($S = 0.5(L - R)$). Boost the Side component proportionally to `width` while preserving the Mid component for unshakeable vocal anchoring. Apply frequency-selective low-frequency centering/crossfeed to maintain bass punch and prevent out-of-phase ear fatigue. Ensure that pure monaural signals ($L = R$) pass through with exactly zero volume drop and zero phase artifacts (`FR19`).

## Boundaries & Constraints

**Always:**
- Process stereo frames at 48 kHz with zero heap allocations and 0.0ms delay (`NFR1`, `NFR2`).
- Decompose to Mid/Side: $M = 0.5(L+R)$, $S = 0.5(L-R)$.
- Modulate width smoothly from 0.0 to 1.0 (Side component scaled from $1.0\times$ to up to $2.5\times$).
- Low-frequency crossfeed / centering (< 200 Hz) to keep bass punch centered in both ears.
- Pure monaural audio ($L = R$) passes through bit-identically ($S = 0 \implies \text{output} = L = R$).
- Smooth parameter interpolation for width changes to prevent clicks (`AD-4`).
- Clean state reset on `reset()`.

**Never:**
- Never attenuate or cancel center vocals when widening.
- Never cause phase cancellation or volume loss on mono tracks.
- Never allocate heap memory on the audio thread.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output |
|----------|---------------|-----------------|
| Pure Mono Signal ($L = R$) | Left = 0.5, Right = 0.5, Width = 1.0 | Side = 0.0, Output Left = 0.5, Right = 0.5 (zero phase cancellation, zero gain loss) |
| Wide Stereo Mix | Distinct L and R instruments | Stereo soundstage widens noticeably beyond earcups while center vocalist remains dead-center |
| Disabled / 0% Width | `enabled = false` or `width = 0.0` | Output equals input dry signal (unity gain pass) |
| Out-of-Phase Low Bass | L = +1.0, R = -1.0 at 50 Hz | Low frequencies centered/crossfed, preventing acoustic phase cancellation |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/dsp/spatial.rs` -- Mid/Side stereo expander and crossfeed processor.
- `crates/open-boom-core/src/dsp/pipeline.rs` -- Integration into Stage 4 of `AudioProcessor`.
- `crates/open-boom-core/tests/sweeteners_tests.rs` -- Unit tests for mono preservation, stereo widening, and frequency-selective crossfeed.
