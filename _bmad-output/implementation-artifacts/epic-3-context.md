# Epic 3 Context: Studio Equalization & AutoEq Headphone Calibration

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

The user can search over 4,000 headphone models offline to apply Harman target parametric compensation with automatic headroom gain protection, sculpt frequency response across 10-band or 31-band faders with an interactive Bezier spline visualizer, and save custom genre presets.

## Stories

- Story 3.1: 31-Band Biquad Filter Engine & Direct Form II Transposed Topology
- Story 3.2: Dual-Mode Equalizer Studio Drawer (10-Band vs 31-Band) & Spline Curve Visualizer
- Story 3.3: Offline AutoEq Headphone Database Indexing & Instant Fuzzy Search
- Story 3.4: Decoupled Parametric AutoEq Stage & Auto-Gain Preamp Compensation
- Story 3.5: Genre Factory Presets & Custom Preset Persistence

## Requirements & Constraints

- 31-Band Graphic Equalizer Engine: 31 second-order IIR peaking filters calculated using Robert Bristow-Johnson Audio EQ Cookbook formulas at ISO 1/3-octave center frequencies (20 Hz to 20 kHz) with constant Q = 4.31, THD+N < 0.005%, and SIMD SSE2/AVX acceleration (`FR9`, `NFR6`, `AD-3`).
- Numerical Stability & Topology: Uses Direct Form II Transposed topology with sub-buffer parameter smoothing (every 32 or 64 samples) to ensure poles strictly remain within the unit circle, preventing filter explosions and zipper noise during rapid slider changes (`AD-4`).
- Band Adjustment Precision: Faders adjust gain within -12.0 dB to +12.0 dB at 0.1 dB resolution (`FR9`).
- Dual-Mode Studio Drawer UI: Slide-out drawer toggleable between casual "10-Band" mode (standard octave frequencies 31 Hz to 16 kHz) and "31-Band" studio mode. Switching modes preserves the underlying 31-band curve without resetting faders (`FR12`, `UX-DR7`).
- Fader Slider Dynamics: 180px stroke faders with floating dB badge tooltip, magnetic detent snap to 0.0 dB within +/-0.3 dB, 5x fine precision while holding `Shift`, and instant snap to 0.0 dB on double click (`UX-DR7`, `EXPERIENCE.md §6.2`).
- Interactive Bezier Spline Curve: Real-time canvas/SVG visualizer above faders rendering the cumulative frequency response curve with gradient fill and reactive frequency nodes (`FR10`, `UX-DR7`).
- Offline AutoEq Database Index: Bundled compressed index in `crates/open-boom-storage` containing 4,000+ headphone models measuring < 5 MB on disk, queried via instant in-memory fuzzy search with sub-50ms latency (`FR13`, `FR14`, `NFR7`, `AD-9`).
- Decoupled Parametric AutoEq Stage: Dedicated stage holding up to 12 second-order parametric IIR biquad filters positioned in the DSP pipeline sequentially *before* the 31-band graphic EQ (`AD-6`).
- Automatic Preamp Compensation: Dynamic calculation of negative preamp gain offset (`preamp_offset_db = -max_positive_gain_db`) applied atomically when committing positive filter boosts to eliminate digital clipping before the master limiter (`FR15`, `AD-6`).
- Factory & Custom Presets: 15 factory presets (Flat, Bass Boost, Treble Boost, Acoustic, Classical, Electronic, Gaming, Hip-Hop, Jazz, Movie, Pop, Rock, Vocal, Lounge, Deep) and user custom preset persistence via atomic file writes to `%APPDATA%\OpenBoom\presets\*.json` (`FR11`, `AD-10`).
- Real-Time Budget: Zero heap allocations and non-blocking operation on the MMCSS audio callback thread (`NFR1`, `NFR2`).

## Technical Decisions

- Pipeline Stage Topology: Audio passes through Stage 3 after the 3D Spatializer (Stage 2):
  1. Parametric AutoEq Correction Stage (up to 12 biquad filters + auto-preamp offset)
  2. 31-Band Graphic Equalizer Bank (31 peaking biquad filters in series/cascade, Direct Form II Transposed)
- Biquad Filter Formula (Peaking EQ):
  $A = 10^{\text{gain\_db} / 40}$
  $\omega_0 = 2\pi f_0 / f_s$
  $\alpha = \sin(\omega_0) / (2 Q)$
  $b_0 = 1 + \alpha A$, $b_1 = -2 \cos(\omega_0)$, $b_2 = 1 - \alpha A$
  $a_0 = 1 + \alpha / A$, $a_1 = -2 \cos(\omega_0)$, $a_2 = 1 - \alpha / A$
  Normalized: $b_i' = b_i / a_0$, $a_1' = a_1 / a_0$, $a_2' = a_2 / a_0$.
- Direct Form II Transposed State:
  $y[n] = b_0 x[n] + s_1[n-1]$
  $s_1[n] = b_1 x[n] - a_1 y[n] + s_2[n-1]$
  $s_2[n] = b_2 x[n] - a_2 y[n]$
- SPSC Command Dispatch: Thread-safe parameter updates dispatched from UI via lock-free queue:
  `SetEqBandGain { band_idx: usize, gain_db: f32 }`, `SetEqGains { gains_db: [f32; 31] }`, `SetAutoEqFilters { filters: Vec<ParametricFilter> }`, `SetAutoEqPreamp { preamp_db: f32 }`, `SetEqBypass { bypassed: bool }`.
- Offline AutoEq Format: Compressed JSON or bincode database embedded or loaded on startup, indexing headphones by brand, model, target curve, and filter parameters.
- Storage Persistence: `crates/open-boom-storage` manages presets with atomic tmpfile + rename replacement.

## UX & Interaction Patterns

- Equalizer Preview Dock: Mini-curve preview and preset pill on the main HUD; clicking "Expand EQ" or pressing `Ctrl + Shift + E` slides open the full Studio Drawer.
- 10-Band vs 31-Band Toggle: Segmented control in the drawer header.
- Fader Strip: Cascadia Code font, magnetic snap at 0.0 dB, double click to reset, Shift for fine-tuning.
- Spline Graph: Cyan gradient line, logarithmic frequency grid (20 Hz to 20 kHz), and horizontal gain markers (-12 dB to +12 dB).
- AutoEq Picker: Search popover with fuzzy match, displaying results with badges (brand, measurement curve, filter count).

## Cross-Story Dependencies

- Story 3.1 creates the core 31-band biquad filter engine and Direct Form II Transposed DSP structure in `crates/open-boom-core`.
- Story 3.2 builds the Studio Drawer UI with 10-band and 31-band faders and the interactive spline visualizer in `src-ui`.
- Story 3.3 implements the offline AutoEq database index and instant fuzzy search in `crates/open-boom-storage`.
- Story 3.4 connects the decoupled parametric AutoEq stage, auto-gain preamp compensation, and pipeline integration.
- Story 3.5 adds the 15 factory presets, custom preset saving/loading, and storage persistence.
