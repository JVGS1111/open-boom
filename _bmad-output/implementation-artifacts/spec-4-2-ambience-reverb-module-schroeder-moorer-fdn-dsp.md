---
title: 'Story 4.2: Ambience Reverb Module (Schroeder-Moorer / FDN DSP)'
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

**Problem:** Dry music recordings and cinema dialogue lack spatial depth and natural acoustic room reflections, sounding flat and trapped inside headphones.

**Approach:** Implement a real-time Feedback Delay Network (FDN) / Schroeder-Moorer reverberation processor in `crates/open-boom-core/src/dsp/ambience.rs`. Utilize mutually prime delay lines with low-pass feedback comb filters and cascaded all-pass diffusion stages to generate natural room decay without metallic ringing or comb-filter artifacts. Modulate wet/dry balance smoothly via linear interpolation with zero heap allocations, ensuring phase coherence for monaural center channels (`FR16`).

## Boundaries & Constraints

**Always:**
- Process stereo audio buffers at 48 kHz with zero heap allocations (`NFR1`, `NFR2`).
- 4 mutually prime feedback comb filters with low-pass damping in the feedback loops, followed by 2 cascaded all-pass diffusers.
- Modulate wet/dry mix ratio smoothly with linear interpolation between parameter steps to eliminate clicks/pops (`AD-4`).
- Maintain phase coherence on monaural / center-panned signals: identical L/R inputs produce equal-power diffuse tails without destructive cancellation (`FR16`).
- Provide clean `reset()` flushing all delay lines to 0.0.

**Never:**
- Never allocate heap memory on the audio processing path.
- Never cause metallic ringing or unstable feedback (loop gains strictly < 1.0).
- Never introduce phase cancellation on center dialogue.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output |
|----------|---------------|-----------------|
| Disabled / 0% Amount | `enabled = false` or `amount = 0.0` | Output strictly equals input dry signal (bit-identical pass) |
| Impulse Response | Single Dirac delta impulse | Generates dense, decaying reverberation tail without discrete echoes or metallic resonance |
| Pure Mono Signal | $L = R$ stereo input | Output maintains stereo room breadth while preserving central dialogue presence |
| Rapid Parameter Change | Amount swept from 0.0 to 1.0 | Smooth mix interpolation without zipper noise or audio dropouts |
| Extreme Signals | Input contains NaN or Inf | Sanitized to 0.0; feedback buffers protected against runaway explosion |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/dsp/ambience.rs` -- Core Schroeder-Moorer / FDN reverb processor.
- `crates/open-boom-core/src/dsp/pipeline.rs` -- Integration into Stage 4 of `AudioProcessor`.
- `crates/open-boom-core/tests/sweeteners_tests.rs` -- Unit tests for decay, diffusion, phase coherence, and zero allocation.
