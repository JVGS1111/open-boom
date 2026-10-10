# Epic 2 Context: 3D Surround Soundstage & Spatial Calibration

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

The user can expand stereo headphone listening into an immersive 7.1 binaural soundstage using an interactive circular 3D dial knob on the main HUD, and open the 8-channel speaker configurator modal to mute, position, and trim individual virtual speaker levels in real time.

## Stories

- Story 2.1: Binaural 7.1 HRTF Convolution Engine & Down-Folding Matrix
- Story 2.2: Interactive Circular 3D Surround Dial Control
- Story 2.3: 8-Channel Virtual Speaker Configurator Soundstage Modal
- Story 2.4: Spatial Virtual Speaker Angular Repositioning

## Requirements & Constraints

- 7.1 Channel Spatialization Matrix: Folds 8 virtual speaker positions (Front Left, Center, Front Right, Surround Left, Surround Right, Rear Left, Rear Right, LFE Subwoofer) down to binaural stereo using calibrated KEMAR dummy-head impulse response matrices with SIMD acceleration (SSE2/AVX).
- Binaural Cues & Localization: Rear and surround channels must introduce discrete interaural time delays (ITD of 0.2-0.7ms) and pinna spectral notches, allowing distinct localization between frontal and rear audio.
- Circular 3D Dial UI (192px outer diameter, 144px inner knob surface): Outer rotary arc sweeps from 0° (0% intensity / dry stereo) to 300° (100% intensity / full binaural soundstage) using polar tracking $\theta = \text{atan2}(y - y_0, x - x_0)$ with detent pips at 25%, 50%, and 75%. Wheel scrolling adjusts by 2% (5% with Shift); double-click resets to 50%.
- Parameter Smoothing & Drop-Free Operation: Rotary intensity and speaker trim/azimuth changes must interpolate coefficients smoothly across samples (AD-4) to eliminate zipper noise and clicks.
- 8-Channel Virtual Speaker Configurator: Polar soundstage modal displaying central listener head and 8 speaker nodes. Clicking toggles mute/active with a red strike indicator; sliders trim relative channel gain within +/-6.0 dB at 0.1 dB resolution. Keyboard shortcut `Ctrl + Shift + S` or calibration icon opens/closes the modal.
- Angular Repositioning: Dragging a virtual speaker node along its orbital ring slides azimuth angle in real time, recalculating HRTF interpolation tables without dropouts. "Reset Speaker Positions" restores ITU-R BS.775 7.1 reference azimuth angles.
- Real-Time Budget & Allocation Invariants: Stage 2 processing operates strictly under MMCSS with zero heap allocations on the audio callback thread and takes < 25% of the 48 kHz buffer period.

## Technical Decisions

- Pipeline Topology (Stage 2): Sits immediately after Stage 1 (Ingress / Preamp) and before Stage 3 (Equalizer). Normalizes input to 8 channels and convolves with HRTF matrices to output 2-channel stereo.
- SIMD Convolver: AVX/SSE2 FIR convolution engine running time-domain or partition-based convolution with zero allocations on the audio thread.
- HRTF Profile Contract: Ships calibrated generic KEMAR dummy-head binaural impulse response tables for standard 7.1 layout angles (FL: -30°, C: 0°, FR: +30°, SL: -90°, SR: +90°, RL: -135°, RR: +135°, LFE: blended). Custom `.sofa` profile loading is deferred to Phase 2.
- SPSC POD Commands: Lock-free communication from UI to audio engine via `AudioCommand` structs (`SetSurroundIntensity { intensity: f32 }`, `SetSpeakerMute { channel: u8, muted: bool }`, `SetSpeakerGain { channel: u8, gain_db: f32 }`, `SetSpeakerAzimuth { channel: u8, azimuth_deg: f32 }`, `ResetSpeakerPositions`).
- State Synchronization: UI maintains speaker angles, mute states, and trim gains, persisting them in `AppConfig` via `open-boom-storage`.

## UX & Interaction Patterns

- Central 3D Surround Dial on HUD: Obsidian rotary knob (`144px`) framed by a glowing cyan arc (`192px`). Live intensity percentage readout in Cascadia Code / Segoe UI.
- Top-Down Polar Soundstage Modal: Centered overlay with orbital concentric rings, listener icon facing 0° (North), and 8 glowing speaker pills (FL, C, FR, SL, SR, RL, RR, LFE).
- Node Interactions: Click to mute/unmute; vertical drag or popover slider for gain (+/-6 dB); orbital drag along ring for azimuth angle.
- Keyboard Navigation & Hotkeys: `Ctrl + Shift + S` opens/closes speaker configurator; Esc or backdrop click closes modal; Arrow keys adjust dial and faders.

## Cross-Story Dependencies

- Story 2.1 provides the core HRTF convolution engine and 8-to-2 down-folding matrix in `open-boom-core`.
- Story 2.2 binds the central 3D dial UI on HUD to Stage 2 intensity modulation.
- Story 2.3 creates the top-down polar soundstage modal and per-channel mute/trim controls.
- Story 2.4 adds orbital angular drag and azimuth interpolation to the soundstage modal and audio engine.
