---
title: 'Story 3.4: Decoupled Parametric AutoEq Stage & Auto-Gain Preamp Compensation'
type: 'feature'
created: '2026-10-11'
status: 'done'
baseline_revision: '1a55f8e130794a1b51495dc4bd89fe32793236fe'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-3-context.md'
  - '_bmad-output/implementation-artifacts/spec-3-1-31-band-biquad-filter-engine-direct-form-ii-transposed-topol.md'
  - '_bmad-output/implementation-artifacts/spec-3-2-dual-mode-equalizer-studio-drawer-10-band-vs-31-band-spline.md'
  - '_bmad-output/implementation-artifacts/spec-3-3-offline-autoeq-headphone-database-indexing-instant-fuzzy-sea.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** Headphone AutoEq calibration curves contain up to 12 arbitrary second-order parametric filters (peaking, low-shelf, high-shelf) that often introduce sharp positive gain boosts (e.g. +6.5 dB at 3.2 kHz or +5.0 dB sub-bass shelf), which cause harsh digital clipping distortion when applied without preamp attenuation or when colliding with custom 31-band user equalizer adjustments.

**Approach:** Implement a dedicated `ParametricStage` in `crates/open-boom-core` positioned sequentially *before* the 31-band `GraphicEqStage` (`AD-6`). Implement Robert Bristow-Johnson Audio EQ Cookbook formulas for peaking, low-shelf, and high-shelf biquad filters with Schur-Cohn stability checks. Calculate automatic negative preamp attenuation offset (`preamp_offset_db = -max_positive_gain_db`) whenever positive filter boosts are committed to guarantee headroom before subsequent stages and the master limiter (`FR15`). Dispatch atomic POD commands (`SetAutoEqProfile`, `SetAutoEqBypass`) across the lock-free SPSC queue with zero heap allocations on the audio callback thread (`AD-1`, `NFR1`, `NFR2`).

## Boundaries & Constraints

**Always:**
- Position the Parametric AutoEq Stage in `AudioProcessor` sequentially *after* the 3D Spatializer (Stage 2) and *before* the 31-band Graphic Equalizer (Stage 3B) (`AD-6`).
- Implement Robert Bristow-Johnson biquad formulas for `low_shelf` and `high_shelf` in `crates/open-boom-core/src/dsp/biquad.rs` with pole stability validation ($|z| < 1.0$).
- Calculate automatic negative preamp attenuation offset: $A_{\text{preamp}} = -\max(0, \max_i(G_i))$ where $G_i$ are the filter gains in dB, and apply this gain linearly $10^{A_{\text{preamp}} / 20}$ at the input of the parametric stage (`FR15`, `AD-6`).
- Transfer parametric filter snapshots as fixed-size Plain Old Data structs (`ParametricFilterPod` array up to `MAX_PARAMETRIC_FILTERS = 12`) via the SPSC lock-free ring buffer without heap allocations or mutexes on the audio thread (`AD-1`, `NFR1`, `NFR2`).
- Smooth filter coefficient transitions across 32-sample sub-buffers using Direct Form II Transposed topology with anti-denormal protection to prevent audible zipper noise or DC pops (`AD-4`).
- Update UI telemetry and preamp offset badge (`-5.2 dB`) in `EqualizerPreview` and `EqualizerStudioDrawer`.

**Never:**
- Never allocate heap memory, invoke COM methods, or block on locks inside `ParametricStage::process_stereo` on the MMCSS audio thread (`NFR1`, `NFR2`).
- Never allow filter coefficient recalculation to generate unstable poles outside the unit circle ($|z| \ge 1.0$).
- Never overwrite or reset the user's custom 31-band graphic EQ curve when applying or switching AutoEq headphone profiles (`FR12`, `AD-6`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Apply AutoEq Profile | User selects profile with 8 biquad filters (+6 dB peak) | Computes -6.5 dB preamp offset; dispatches `SetAutoEqProfile`; stage processes Left/Right channels cleanly | Fallback to unity gain if invalid |
| Low Shelf Filter | Filter type `LowShelf`, 105 Hz, Q=0.71, +5.0 dB | RBJ Low Shelf biquad computed; bass boosted cleanly below 105 Hz with negative preamp compensation | Clamp Q and gain to stable ranges |
| High Shelf Filter | Filter type `HighShelf`, 10 kHz, Q=0.71, -3.0 dB | RBJ High Shelf biquad computed; treble attenuated smoothly above 10 kHz | Clamp cutoff < Nyquist (24 kHz) |
| Simultaneous 31-Band Boost | AutoEq active (-5 dB preamp) + User boosts 1 kHz band by +6 dB | Both stages cascade sequentially; signal does not clip before master limiter | Master limiter catches any post-EQ peaks |
| Reset to Flat / None | User clicks "Reset to Flat" in AutoEq popover | Dispatches `SetAutoEqBypass(true)` or empty profile; preamp resets to 0.0 dB; 31-band EQ preserved | Instant smooth transition |
| Zero or Negative Boosts Only | Profile with only negative cuts (e.g. -4 dB notch) | Preamp offset calculated as 0.0 dB (no attenuation needed); gain untouched | Do not add positive digital gain |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/dsp/biquad.rs` -- Implement `low_shelf` and `high_shelf` biquad coefficient generation formulas, Schur-Cohn stability checks, and DF-II Transposed processing.
- `crates/open-boom-core/src/types.rs` -- Define `ParametricFilterPod` and add `AudioCommand::SetAutoEqProfile` and `AudioCommand::SetAutoEqBypass` to POD enum.
- `crates/open-boom-core/src/dsp/parametric_stage.rs` -- Enhance `ParametricStage` with filter array loading, linear preamp gain, sub-buffer smoothing, and DF-II Transposed execution.
- `crates/open-boom-core/src/dsp/pipeline.rs` -- Wire `ParametricStage` into `AudioProcessor` sequentially *before* `GraphicEqStage`, draining SPSC commands with zero heap allocations.
- `src-tauri/src/dispatcher.rs` & `src-tauri/src/lib.rs` -- Implement `apply_autoeq_profile` and `clear_autoeq_profile` Tauri IPC commands and dispatch to SPSC queue.
- `src-ui/src/App.tsx` -- Calculate preamp offset, dispatch `apply_autoeq_profile` on headphone profile selection, and manage combined preamp telemetry.
- `crates/open-boom-core/tests/autoeq_parametric_tests.rs` -- Comprehensive DSP tests verifying RBJ low/high shelf formulas, stability under extreme gains, zero-allocation processing, THD+N, and anti-clipping headroom.
- `src-ui/src/test/AutoEqParametric.test.tsx` -- Frontend tests verifying auto-gain preamp calculation and IPC dispatch.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-core` -- Implement RBJ low/high shelf biquads, POD command structs, and `ParametricStage` integration in `AudioProcessor`.
- `src-tauri` -- Wire Tauri commands `apply_autoeq_profile` and `clear_autoeq_profile`.
- `src-ui` -- Compute auto-gain preamp offset and wire selection dispatch in `App.tsx`.
- Verification -- Create test suites covering DSP math, pipeline cascade, zero allocation, and UI dispatch.

**Acceptance Criteria:**
- Given a selected AutoEq profile, when the user clicks "Apply Profile", then up to 12 second-order parametric IIR biquad filters are loaded into the Parametric Correction Stage sequentially *before* the 31-band graphic equalizer (`AD-6`).
- Given an AutoEq profile with positive gain peaks (e.g. +6.5 dB peak at 3.2 kHz), when the snapshot is committed, then the UI Control Store automatically calculates an equal negative preamp attenuation offset (`preamp_offset_db = -max_positive_gain_db`) and transmits it atomically in the snapshot (`FR15`, `AD-6`).
- Given both an AutoEq profile and custom 31-band fader boosts active simultaneously, when audio processes through the pipeline, then headroom attenuation prevents intermediate digital clipping before the master limiter stage (`FR15`, `AD-6`).

## Spec Change Log
- 2026-10-11: Created specification for Story 3.4: Decoupled Parametric AutoEq Stage & Auto-Gain Preamp Compensation.
- 2026-10-11: Completed implementation and verification of Story 3.4. Implemented RBJ low/high shelf biquads with Schur-Cohn stability, ParametricFilterPod structs, lock-free SPSC AudioCommands, ParametricStage with sub-buffer block smoothing, AudioProcessor sequential cascade (Stage 3A -> Stage 3B), automatic negative preamp offset calculation, Tauri IPC commands apply_autoeq_profile / clear_autoeq_profile, and UI badge updates. All 116 Rust tests and 39 UI tests passing.

## Review Triage Log
- Blind Hunter & Edge Case Review:
  - Fixed `LoadGraphicEqSnapshot` in `App.tsx` (`restoreConfig` and `handleResetEqGains`) to pass `preamp_offset_db: 0.0` rather than the AutoEq `preampOffsetDb`, preventing double preamp attenuation across both `ParametricStage` and `GraphicEqStage`.
  - Clamped `preamp_offset_db` in `ParametricStage::load_profile` to `[-36.0, 0.0]` dB.
  - Reset internal DF-II Transposed delay lines (`s1`, `s2`) for inactive filter slots (`count..MAX_PARAMETRIC_FILTERS`) in `load_profile` and `clear_profile` to prevent stale delay state leakage when filter counts vary across profiles.
- Verification Gap Review:
  - Verified 11 DSP unit/integration tests in `autoeq_parametric_tests.rs`, Tauri serde command tests, and 9 UI tests in `AutoEqParametric.test.tsx`.
- Intent Alignment Auditor:
  - Confirmed implementation matches Reading 2 (Full-Stack Vertical Slice) with complete core DSP, IPC, and UI wiring.

## Auto Run Result

- **Status:** done
- **Summary:** Complete implementation and verification of Story 3.4: Decoupled Parametric AutoEq Stage & Auto-Gain Preamp Compensation. Implemented Robert Bristow-Johnson biquad formulas for Low Shelf and High Shelf filters in `crates/open-boom-core/src/dsp/biquad.rs` with Schur-Cohn stability checks ($|z| < 1.0$). Created POD structs `ParametricFilterPod` and `ParametricFilterType` in `types.rs` transferred via SPSC ring buffer without heap allocations (`AD-1`, `NFR1`, `NFR2`). Implemented `ParametricStage` in `parametric_stage.rs` evaluating up to 12 second-order biquad filters per channel with 32-sample block smoothing and linear negative preamp attenuation. Integrated `ParametricStage` sequentially *before* `GraphicEqStage` in `AudioProcessor` (`AD-6`). Added Tauri IPC commands `apply_autoeq_profile` and `clear_autoeq_profile` in `src-tauri`. Implemented `calculateAutoEqPreampOffset` in `App.tsx` and updated `EqualizerPreview` and `EqualizerStudioDrawer` preamp badges.
- **Files Changed:**
  - `crates/open-boom-core/src/dsp/biquad.rs` -- Added RBJ `low_shelf` and `high_shelf` constructors, Schur-Cohn stability checks, and wide gain dynamic range support.
  - `crates/open-boom-core/src/types.rs` -- Defined `ParametricFilterType`, `ParametricFilterPod`, `AudioCommand::SetAutoEqProfile`, `AudioCommand::SetAutoEqBypass`, and `TelemetrySnapshot` AutoEq fields.
  - `crates/open-boom-core/src/dsp/parametric_stage.rs` -- Full `ParametricStage` implementation with filter array loading, automatic negative preamp calculation, 32-sample sub-buffer smoothing, and DF-II Transposed sample processing.
  - `crates/open-boom-core/src/dsp/pipeline.rs` -- Wired `ParametricStage` sequentially *before* `GraphicEqStage` in `AudioProcessor`, handling `SetAutoEqProfile` and `SetAutoEqBypass` commands and telemetry.
  - `crates/open-boom-core/src/lib.rs` -- Re-exported `ParametricStage`, `ParametricFilterPod`, and `ParametricFilterType`.
  - `crates/open-boom-core/tests/autoeq_parametric_tests.rs` -- 11 comprehensive DSP tests verifying shelf formulas, extreme gains stability, THD+N < 0.005%, auto preamp offset calculation, cascade ordering, and anti-clipping headroom.
  - `src-tauri/src/dispatcher.rs` -- Added `apply_autoeq_profile` and `clear_autoeq_profile` helper methods to `UiDispatcher`.
  - `src-tauri/src/lib.rs` -- Implemented Tauri IPC commands `apply_autoeq_profile` and `clear_autoeq_profile`, registered in `invoke_handler!`, and added serde unit tests.
  - `src-ui/src/App.tsx` -- Added `calculateAutoEqPreampOffset`, wired dynamic `preampOffsetDb` state, updated `handleSelectAutoEqProfile` and `handleClearAutoEqProfile` with IPC dispatch and notifications.
  - `src-ui/src/test/AutoEqParametric.test.tsx` -- 9 frontend unit and component tests verifying preamp offset calculation, badge rendering, and decoupled 31-band curve preservation.
  - `src-ui/src/test/run.mjs` -- Registered `AutoEqParametric` test suite in runner.
- **Verification:**
  - `cargo test -p open-boom-core --test autoeq_parametric_tests`: 11 passed, 0 failed.
  - `cargo test`: all 116 workspace tests passed with 0 failures.
  - `npm test`: all 39 UI tests passed with 0 failures.
  - `npm run ui:build`: TypeScript compiled cleanly and Vite bundled in 1.76s with 0 errors.
  - `npm run build` (`tauri build`): full release bundles (`.msi` and `.exe`) generated successfully.
- **Residual Risks:** None.

## Verification

**Commands:**
- `cargo test -p open-boom-core` -- expected: all biquad and parametric tests pass with THD+N < 0.005% and zero clippings
- `cargo test` -- expected: workspace tests pass with 0 failures
- `npm run test` -- expected: all UI tests pass with 0 failures
- `npm run build` -- expected: TypeScript compiles with 0 errors
