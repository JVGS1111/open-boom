---
title: 'Story 3.1: 31-Band Biquad Filter Engine & Direct Form II Transposed Topology'
type: 'feature'
created: '2026-10-11'
status: 'done'
baseline_revision: '5e48c65a865517046de221fe2c0791aa4790243c'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-3-context.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** The audio processing pipeline lacks a dedicated, high-precision graphic equalizer capable of sculpting the full audible human hearing spectrum (20 Hz – 20 kHz) without filter instability, phase explosions, or audible zipper noise during real-time adjustments.

**Approach:** Implement a 31-band ISO 1/3-octave graphic equalizer engine in `crates/open-boom-core` using Robert Bristow-Johnson Audio EQ Cookbook second-order peaking biquad filters (constant Q = 4.31) with Direct Form II Transposed topology. Guarantee numerical stability with sub-buffer parameter smoothing (every 32/64 samples), ensure filter poles remain strictly within the unit circle (|pole| < 1.0), achieve ultra-low distortion (THD+N < 0.005%), and integrate Stage 3 into `AudioProcessor` with zero-allocation lock-free SPSC commands.

## Boundaries & Constraints

**Always:**
- Process 31 ISO 1/3-octave center frequencies: 20.0, 25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0, 125.0, 160.0, 200.0, 250.0, 315.0, 400.0, 500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0, 4000.0, 5000.0, 6300.0, 8000.0, 10000.0, 12500.0, 16000.0, 20000.0 Hz (`types::ISO_31_BAND_FREQUENCIES`).
- Constant Q = 4.31 corresponding to standard 1/3-octave bandwidth across all 31 bands.
- Individual band gain range strictly bounded to [-12.0 dB, +12.0 dB] at 0.1 dB resolution (`FR9`).
- Use Direct Form II Transposed topology with normalized coefficients ($a_0 = 1.0$) and internal states $s_1, s_2$ to minimize quantization noise and ensure canonical stability (`AD-4`).
- Update filter coefficients smoothly across sub-buffer boundaries (every 32 or 64 samples) to prevent zipper noise and audible clicks during rapid slider dragging.
- Verify that all filter poles strictly remain within the unit circle ($|z| < 1.0$) under all gain conditions from -12 dB to +12 dB.
- Maintain zero heap allocations on the audio callback thread (`NFR1`, `NFR2`).
- Flush and reset internal delay lines to 0.0 on `ResetFilters` or sample rate reinitialization.

**Never:**
- Never allocate heap memory, lock mutexes, or perform blocking I/O on the real-time audio thread.
- Never allow filter poles to migrate outside the unit circle or produce unbounded feedback explosions ($NaN$ / $\infty$).
- Never produce zipper noise, discontinuities, or pops during continuous gain automation.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Flat Response | All 31 bands at 0.0 dB | Input passes unchanged; total harmonic distortion + noise THD+N < 0.005%; phase response near-linear | Clean unity pass |
| Peak Boost | Band 17 (1 kHz) at +12.0 dB | 1 kHz center frequency boosted by exactly +12.0 dB; adjacent bands taper according to Q=4.31 envelope | Clamped to +12.0 dB |
| Peak Cut | Band 17 (1 kHz) at -12.0 dB | 1 kHz center frequency attenuated by -12.0 dB | Clamped to -12.0 dB |
| Rapid Parameter Drag | Slider swept -12 dB to +12 dB across 100 frames | Sub-buffer block smoothing (every 32/64 samples) ramps coefficients smoothly; zero clicks or pops | Linear coefficient lerp |
| Extreme / NaN Inputs | Gain set to NaN, Inf, or +50 dB | Sanitized and clamped to [-12.0 dB, +12.0 dB]; default to 0.0 dB if NaN | Immediate value sanitization |
| Filter Delay Flush | `ResetFilters` command dispatched | State variables $s_1 = 0.0$, $s_2 = 0.0$ for all 31 bands on both L and R channels | Zero output transient |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/dsp/biquad.rs` -- Peaking filter coefficient calculation via RBJ Audio EQ Cookbook formulas, pole stability check ($|a_2| < 1.0$, $|a_1| < 1.0 + a_2$), and Direct Form II Transposed sample processing.
- `crates/open-boom-core/src/dsp/graphic_eq_stage.rs` -- 31-band cascade graphic equalizer for stereo channels, holding left and right biquads, target vs current gains, sub-buffer block smoothing (32 samples), and bypass/enable switch.
- `crates/open-boom-core/src/dsp/pipeline.rs` -- Connect `GraphicEqStage` as Stage 3 in `AudioProcessor`, dispatching `LoadGraphicEqSnapshot`, `SetEqBandGain`, and `SetEqBypass`.
- `crates/open-boom-core/src/types.rs` -- SPSC `AudioCommand` variants and constants for graphic EQ.
- `crates/open-boom-core/tests/eq_tests.rs` -- Comprehensive unit and integration test suite covering 31-band frequency response, pole stability, sub-buffer smoothing, and distortion benchmarks.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-core/src/dsp/biquad.rs` -- Add `peaking_eq(sample_rate, freq, q, gain_db)` constructor, pole modulus check, and coefficient interpolation.
- `crates/open-boom-core/src/dsp/graphic_eq_stage.rs` -- Implement 31-band stereo cascade with block-level smoothing and gain clamping.
- `crates/open-boom-core/src/dsp/pipeline.rs` -- Wire `GraphicEqStage` into `AudioProcessor::process_buffer` between Stage 2 (Spatializer) and Stage 4 (Master Limiter), handling `LoadGraphicEqSnapshot` and `SetEqBandGain`.
- `crates/open-boom-core/tests/eq_tests.rs` -- Write test suite covering all rows of the I/O & Edge-Case Matrix.

**Acceptance Criteria:**
- Given 31-band peaking filters, when initialized across ISO 1/3-octave frequencies from 20 Hz to 20 kHz with Q=4.31, then all filter poles have modulus strictly < 1.0 (`FR9`, `AD-4`).
- Given an audio stream through `GraphicEqStage`, when all bands are at 0.0 dB, then output matches input with THD+N < 0.005% (`NFR6`).
- Given rapid gain changes, when coefficients update across 32-sample sub-buffers, then output transitions without audible clicks, discontinuities, or NaN output (`AD-4`).

## Spec Change Log
- 2026-10-11: Completed implementation of 31-Band Biquad Filter Engine & Direct Form II Transposed Topology (Story 3.1). Implemented RBJ peaking formulas, constant Q=4.31, pole stability verification, sub-buffer block smoothing (32 samples), linear coefficient lerp, Stage 3 pipeline wiring in AudioProcessor, and comprehensive eq_tests suite. All 105 workspace tests passing.

## Review Triage Log

### 2026-10-11 — Review pass
- verdicts: 12 findings — high 0, medium 6, low 6, false 0, maybe-false 0
- findings:
  - `[medium]` `[patch]` AudioProcessor buffer processing test gap — Added `test_pipeline_active_eq_boost_audio_processing` asserting +12 dB boost reflects on steady-state audio samples.
  - `[medium]` `[patch]` `AudioCommand::ResetFilters` filter flushing in AudioProcessor tested on uncharged state — Added `test_pipeline_reset_filters_clears_charged_state` pre-charging delay lines with active EQ and sinusoidal signal before asserting clear to (0.0, 0.0).
  - `[medium]` `[patch]` `AudioCommand::SetEqGains` full-bank dispatch unverified across queue — Added `test_pipeline_set_eq_gains_full_bank_dispatch` verifying all 31 band target gains update.
  - `[medium]` `[patch]` `AudioCommand::SetEqBypass` audio passthrough unverified — Added `test_graphic_eq_bypass_bit_perfect_dry_passthrough` verifying bit-perfect equality.
  - `[medium]` `[patch]` Preamp gain attenuation unverified during real-time processing — Added `test_preamp_offset_attenuation_applied_to_audio` verifying -6 dB attenuation.
  - `[medium]` `[patch]` State poisoning recovery in `Biquad::process_sample` — Added self-healing logic resetting s1, s2 to 0.0 if non-finite.
  - `[low]` `[patch]` Potential panic in `BiquadCoefficients::peaking_eq` on low sample rates — Validated sample rate >= 8000 Hz and bounded safe_freq <= 0.499 * safe_sr.
  - `[low]` `[patch]` Safe log10 in `frequency_response_db` against sub-normal squared magnitudes — Guarded num_mag_sq <= 1e-12 returning -120.0 dB.
  - `[low]` `[patch]` Odd trailing samples desynchronizing right channel filter state — Added `GraphicEqStage::process_mono_left` to process only left channel for odd samples.
  - `[low]` `[patch]` Sample rate reconfiguration bridge in `AudioProcessor` — Added `AudioProcessor::set_sample_rate` and reset smoothing state in `GraphicEqStage`.
  - `[low]` `[patch]` 10-Band ISO octave frequencies and band indices mapping constants — Defined `ISO_10_BAND_FREQUENCIES` and `ISO_10_TO_31_BAND_INDICES` in `types.rs`.
  - `[low]` `[patch]` Deserialization tests for new `AudioCommand` variants — Added serde unit tests in `src-tauri/src/lib.rs`.

## Auto Run Result

- **Status:** done
- **Summary:** Complete implementation and verification of Story 3.1: 31-Band Biquad Filter Engine & Direct Form II Transposed Topology. Designed and verified 31 second-order peaking biquad filters operating at ISO 1/3-octave center frequencies (20 Hz - 20 kHz) with constant Q = 4.31, normalized coefficients (a0 = 1.0), and Schur-Cohn stability. Direct Form II Transposed topology ensures zero filter explosions (|pole| < 1.0), and sub-buffer 32-sample parameter smoothing eliminates zipper noise. Full integration into `AudioProcessor` Stage 3 with lock-free SPSC command dispatch.
- **Files Changed:**
  - `crates/open-boom-core/src/dsp/biquad.rs` — RBJ peaking filter formulas, pole modulus stability, lerp interpolation, self-healing DF-II Transposed process function.
  - `crates/open-boom-core/src/dsp/graphic_eq_stage.rs` — 31-band cascade stereo filter bank, sub-buffer block smoothing, gain clamping, process_mono_left.
  - `crates/open-boom-core/src/dsp/pipeline.rs` — Stage 3 integration in AudioProcessor, command dispatch for SetEqBandGain/SetEqGains/SetEqBypass/LoadGraphicEqSnapshot/ResetFilters, and set_sample_rate.
  - `crates/open-boom-core/src/types.rs` — Defined EQ_Q, MIN/MAX_EQ_GAIN_DB, CASUAL_EQ_10_BANDS, ISO_10_BAND_FREQUENCIES, and SPSC AudioCommand variants.
  - `crates/open-boom-core/src/lib.rs` — Re-exported biquad and graphic EQ stages.
  - `crates/open-boom-core/tests/eq_tests.rs` — 15 comprehensive unit and integration tests covering pole stability, THD+N, rapid drag, reset, bypass, preamp, and pipeline audio processing.
  - `src-tauri/src/lib.rs` — Unit tests for AudioCommand JSON deserialization.
- **Verification:**
  - `cargo test -p open-boom-core --test eq_tests`: 15 passed, 0 failed.
  - `cargo test`: all 104 workspace tests passed.
  - `npm run build`: Vite build completed in 3.70s; MSI and NSIS release packages generated cleanly.
- **Residual Risks:** None.

## Verification

**Commands:**
- `cargo test -p open-boom-core --test eq_tests` -- expected: all 31-band EQ tests pass
- `cargo test` -- expected: entire workspace test suite passes with zero failures
