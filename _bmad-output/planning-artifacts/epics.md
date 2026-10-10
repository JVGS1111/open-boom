---
status: final
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - _bmad-output/planning-artifacts/prds/prd-open-boom-2026-10-10/prd.md
  - _bmad-output/planning-artifacts/architecture/architecture-open-boom-2026-10-10/ARCHITECTURE-SPINE.md
  - _bmad-output/planning-artifacts/ux-designs/ux-open-boom-2026-10-10/DESIGN.md
  - _bmad-output/planning-artifacts/ux-designs/ux-open-boom-2026-10-10/EXPERIENCE.md
---

# open-boom - Implementation Roadmap

## Overview

This document provides the complete epic and story breakdown for open-boom, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Audio Device Discovery & Hot-Plug Monitoring - Enumerate active Windows audio render endpoints via MMDeviceAPI and dynamically handle device connect/disconnect events within 500ms with graceful fallback.
FR2: Driverless User-Mode Loopback Capture - User-mode audio capture via WASAPI Process Loopback (AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS) on Windows 11 and Windows 10 (build 19041+/20348+) without kernel drivers, with optional virtual cable bridge fallback for legacy environments.
FR3: Format Negotiation & Resampling - Ingest input audio streams at arbitrary sample rates (44.1 kHz, 48 kHz, 96 kHz, 192 kHz) and channel configurations (Stereo 2.0 to 7.1) with artifact-free resampling to 48 kHz 32-bit float.
FR4: Master Volume & Instant A/B Bypass - Global volume control (0% to 150%) and instant sub-10ms phase-continuous A/B bypass toggling between processed DSP and uncolored direct audio without clicks or volume surges.
FR5: 7.1 Channel Spatialization Matrix - Fold 8 virtual speaker positions (FL, C, FR, SL, SR, RL, RR, LFE) into binaural stereo using HRTF filter matrices with interaural time delays (0.2–0.7ms) and spectral elevation cues.
FR6: Interactive Circular 3D Dial UI - Rotary UI knob modulating 3D spatializer intensity from 0% to 100% with smooth continuous parameter updates without audio stutter.
FR7: Individual Virtual Speaker Toggling & Gain - Speaker configuration interface to individually mute/solo and adjust relative trim gain (±6 dB) for all 8 virtual speaker nodes.
FR8: Custom HRTF / SOFA Profile Loading - Import Spatially Oriented Format for Acoustics (.sofa) files (Phase 2 deferred; MVP delivers calibrated generic KEMAR dummy-head binaural impulse matrix).
FR9: 31-Band Biquad Filter Engine - ISO 1/3-octave equalizer bands (20 Hz to 20,000 Hz) using Robert Bristow-Johnson biquad peaking filters with constant Q (4.31) and range from -12.0 dB to +12.0 dB at 0.1 dB resolution with THD+N < 0.005%.
FR10: Interactive Frequency Response Curve Visualizer - Real-time Bezier spline curve visualizer reflecting combined filter frequency response with interactive control points.
FR11: Genre Presets & User Custom Presets - At least 15 factory genre presets and user ability to save, rename, and delete custom presets stored as formatted JSON in %APPDATA%\OpenBoom\presets\.
FR12: Fast 10-Band / 31-Band Toggle - Switch between streamlined 10-band octave view and 31-band studio view without losing filter profile settings.
FR13: Integrated Offline AutoEq Database - Bundled offline compressed database index (<5 MB) covering 4,000+ headphone models from the AutoEq repository.
FR14: Searchable Headphone Profile Selector - Instant fuzzy-matched search query input (<50ms) to select headphone model calibrations offline.
FR15: Auto-Gain Compensation - Preamp attenuation calculation (preamp_offset_db = -max_positive_gain_db) when positive gain is present in AutoEq or EQ profiles to prevent digital clipping before the limiter stage.
FR16: Ambience (Reverberation & Acoustic Space) - Feedback delay network (FDN) / Schroeder-Moorer reverb simulating acoustic room reflections and stage width with 0% to 100% wet/dry mix.
FR17: Fidelity (Harmonic Exciter) - Psychoacoustic exciter generating subtle even/odd harmonics and high-shelf air (0% to 100% depth) without harsh sibilance or aliasing.
FR18: Night Mode (Dynamic Range Compressor) - Dynamic range compressor with fast attack (<5ms), soft knee, and program-dependent release, taming explosive spikes while boosting low-level dialogue (0% to 100% leveling intensity).
FR19: Spatial (Stereo Expander & Crossfeed) - Mid/Side matrixing with frequency-selective crossfeed to expand the stereo field beyond earcups while strictly preserving mono center integrity (0% to 100% width).
FR20: Pitch (Real-Time Pitch Shifter) - Real-time pitch transposition by ±12 semitones without tempo modification (Phase 2 deferred).
FR21: Process Enumeration & Icon Display - Query IAudioSessionManager2 on a dedicated COM MTA thread to display active Windows audio sessions, process names, executable icons, and real-time playback indicators.
FR22: Independent Process Volume Faders - Individual volume faders (0% to 100%, up to 150% boost) and mute toggles per application process.
FR23: Persistent Application Settings - Automatic persistence of custom application volume offsets and mute states across system restarts.
FR24: Lookahead Peak Limiter - Terminal brickwall peak limiter with 2ms lookahead delay and transparent release envelope clamping inter-sample peaks at -0.2 dBFS (0% digital clipping distortion).
FR25: Real-Time Limiter Gain Reduction Indicator - Status bar visual telemetry indicator pulsing amber/red to indicate real-time dB gain reduction applied by the limiter.
FR26: Dark Fluent Visual Theme - Cohesive dark aesthetic (#121316 background, #16171b surface, #38bdf8 cyan and #3b82f6 blue accents, #ec4899 magenta accents) with smooth state transitions.
FR27: Master Power Switch - Prominent illuminated master toggle switch enabling the DSP chain or dropping into zero-latency passthrough with 0.0% DSP CPU consumption.
FR28: Windows System Tray & Background Run - System tray minimization on window close, tray icon restoration in <200ms, and tray right-click context menu (Master Power, Preset selector, Mute, Exit).
FR29: Windows Startup & Session Persistence - Optional launch on Windows startup, window geometry persistence, active preset restoration, and volume state persistence.

### NonFunctional Requirements

NFR1: Real-Time Audio Priority & Budget - Audio callback processing must execute within Windows MMCSS ("Pro Audio" profile via AvSetMmThreadCharacteristicsW) with callback processing time strictly < 60% of buffer duration.
NFR2: Zero Heap Allocation on Audio Thread - Dynamic memory allocation (malloc, new, vector resizing) and blocking synchronization primitives (Mutex, condition variables) are strictly prohibited inside the real-time audio callback loop.
NFR3: Round-Trip Latency - Total buffer round-trip latency between audio generation and headphone output is < 15ms at 48 kHz.
NFR4: OS & Architecture Compatibility - Native 64-bit Windows support (x64 and ARM64) on Windows 10 (build 19041+) and Windows 11 (build 22000+).
NFR5: Resource Footprint - Idle RAM consumption < 40 MB; active DSP processing CPU usage < 2.0% on Intel Core i5 / AMD Ryzen 5 or equivalent.
NFR6: SIMD Acceleration - DSP mathematical routines (biquad filtering, convolution, FFT) must utilize SSE2/AVX vector intrinsics on x64 to minimize battery and CPU impact.
NFR7: Privacy & Security - 100% local execution, zero outbound network traffic during audio processing, zero telemetry beacons, crash analytics, or user tracking cookies.
NFR8: Zero Kernel Crashes - 100% user-mode execution with 0 operating system BSODs or kernel panics (no proprietary .sys drivers).
NFR9: Digital Clipping Protection - Master lookahead brickwall limiter clamps true output peaks at -0.2 dBFS; max measured peak never exceeds -0.1 dBFS across all audio tracks.
NFR10: AutoEq Calibration & Startup Speed - Headphone model search and biquad profile application completes in < 100ms; application cold start to audio readiness in < 800ms.

### Additional Requirements

- Architecture Greenfield Workspace Scaffold: Cargo workspace root with crates/open-boom-core (pure DSP math and engine), crates/open-boom-wasapi (WASAPI capture/render and session management), crates/open-boom-storage (AutoEq and presets), and desktop shell src-tauri with src-ui (Tauri v2 + React 18 + TypeScript + Tailwind CSS).
- AD-1 Lock-Free POD Audio Boundary: Single-Producer Single-Consumer (SPSC) lock-free ring buffer transferring fixed-size, flat #[repr(C)] POD command structs (AudioCommand enum) from the UI Dispatcher to the audio thread; zero heap allocation and zero JSON deserialization on the audio thread.
- AD-2 Driverless Routing & Slapback Echo Prevention: Dual-mode capture strategy: Mode A (Virtual loopback bridge ingress) and Mode B (Driverless WASAPI Process Loopback with PROCESS_LOOPBACK_MODE_EXCLUDE_PROCESS_TREE and programmatic session attenuation via ISimpleAudioVolume to prevent double playback).
- AD-3 Unified 48 kHz Float32 Pipeline Contract: All internal DSP stages process continuous 32-bit floating point PCM (f32) normalized to [-1.0, +1.0] at fixed 48,000 Hz. Stage 1 normalizes ingress to 7.1 (8 channels), Stage 2 (3D Spatializer) down-folds to stereo (2 channels), Stages 3-5 process stereo. Ingress boundary handles resampling for 44.1, 96, 192 kHz streams.
- AD-4 Single Source of Truth & Block-Based Parameter Smoothing: UI maintains single source of truth; parameter adjustments apply linear interpolation; biquad coefficients recalculated at sub-buffer block boundaries (every 32 or 64 samples) using Direct Form II Transposed topology to prevent filter pole blowup.
- AD-5 Phase-Continuous Bypass & Limiter Invariance: Master Peak Limiter is the mandatory terminal stage (2ms lookahead delay buffer, -0.2 dBFS ceiling). A/B Bypass routes around filter stages while retaining the 2ms lookahead delay and limiter, crossfading over 5ms to guarantee phase continuity without audible clicks or latency jumps.
- AD-6 Decoupled Dual-Stage Equalization: Sequential 2-stage EQ: 1) Parametric Correction Stage (up to 12 arbitrary second-order IIR biquad filters from AutoEq) followed by 2) Graphic EQ Stage (fixed 31-band ISO 1/3-octave peaking filters with constant Q 4.31). Automatic preamp digital attenuation offset (preamp_offset_db = -max_positive_gain_db) transmitted atomically.
- AD-7 Modular Engine / Shell Partitioning: Partition into crates/open-boom-core (Rust C-ABI shared lib / crate) and src-tauri / src-ui (Tauri v2 desktop shell with WebView2) to meet <40 MB RAM budget.
- AD-8 COM MTA Worker Thread for Session Mixer: Process-level volume and mute management via Windows Core Audio IAudioSessionManager2 strictly executed on a dedicated COM MTA worker thread (CoInitializeEx(None, COINIT_MULTITHREADED)); zero process memory injection or DLL hooks.
- AD-9 Offline-First Bundled AutoEq Database Index: Pre-processed and compressed binary/JSON index (<5 MB, target ~1.8 MB) bundled with app for in-memory prefix lookups with <50ms query latency.
- AD-10 Atomic JSON Configuration Persistence: Presets, window geometry, and settings stored in %APPDATA%\OpenBoom\ as formatted UTF-8 JSON; all writes use temporary file (.tmp) followed by atomic filesystem rename (MoveFileExW / MOVEFILE_REPLACE_EXISTING).

### UX Design Requirements

UX-DR1: Dark Studio Design Token System - Implement CSS/Tailwind design tokens according to DESIGN.md: base surfaces (#121316, #16171b, #1c1e24, #22252c, #282b34), accents (#38bdf8 cyan, #3b82f6 blue, #ec4899 magenta), text hierarchy (#f8fafc, #94a3b8, #64748b), and signal indicators (#22c55e, #ef4444, #f59e0b).
UX-DR2: Master Power Switch Component - Reusable glowing toggle switch (48px x 26px, full pill radius) with cyan LED glow effect (rgba(56, 189, 248, 0.35)), enabling/disabling the audio DSP pipeline.
UX-DR3: Global Volume & Boost Slider Component - Horizontal fader track (6px height) with fill, thumb, and numeric display supporting 0% to 150% volume (0–100% normal, >100% software boost), double-click to snap to 100% (0 dB), and mute icon toggle.
UX-DR4: Interactive 3D Surround Rotary Dial Component - Concentric circular dial (192px outer size, 144px knob) tracking polar angle theta = atan2(y - y0, x - x0) from 0 deg (0%) to 300 deg (100%), with visual detent pips at 25%, 50%, 75%, center numeric readout, mouse wheel step (2% / 5% with Shift), and double-click to reset to 50%.
UX-DR5: 8-Channel Virtual Speaker Configurator Modal - Polar soundstage modal displaying listener head and 8 speaker nodes (FL, C, FR, SL, SR, RL, RR, Sub) allowing individual channel muting (red strike indicator), orbital angular repositioning, and relative gain trim (±6 dB).
UX-DR6: 5-Effect Quick Rack Strip Component - Horizontal strip of 5 modular cards (Ambience, Fidelity, Night Mode, Spatial, Pitch), each with power toggle, mini-fader slider (0–100% / -12 to +12 st), and active state highlight border (#38bdf8).
UX-DR7: Dual-Mode Equalizer Studio Drawer & Spline Curve Visualizer - Expandable full-height EQ drawer with segmented toggle between 10-Band (Basic) and 31-Band (Precision) vertical faders (180px stroke, ±12 dB, 0.1 dB precision with Shift, magnetic snap at 0.0 dB, double-click zero snap) and dynamic Bezier spline curve canvas visualizer.
UX-DR8: AutoEq Headphone Browser & Profile Chip - Popover search dropdown with fuzzy autocomplete over 4,000+ headphone models, displaying manufacturer, model, target curve source, and 1-click apply, linked to active profile chip in EQ dock.
UX-DR9: Application Stream Mixer Drawer - Slide-out drawer displaying live Windows audio sessions (IAudioSessionManager2) with process name, executable icon, live playback meter, 0–100% fader, and per-app mute toggle.
UX-DR10: Persistent Status Bar & Instant A/B Bypass Button - Bottom dock displaying real-time engine telemetry (48 kHz, 32-bit Float, round-trip latency badge, Limiter headroom meter with red/amber pulse on gain reduction) and instant 1-click A/B Bypass comparison button.
UX-DR11: High Contrast & Keyboard Accessibility Floor - Full support for Windows Contrast Themes (Desert, Dusk, Night Sky, Aquatic), solid 2px outline focus borders, logical Tab traversal order across Header -> Dial -> Effects -> EQ, and UIA RangeValuePattern and TogglePattern bindings for screen readers.
UX-DR12: System Tray & Desktop Window Shell - Compact HUD window geometry (840px x 560px, minimum 780px x 520px), frameless title bar with minimize/close, minimize to system tray on close, single/double click restore (<200ms), and tray context menu.

### FR Coverage Map

FR1: Epic 1 - Audio Device Discovery & Hot-Plug Monitoring
FR2: Epic 1 - Driverless User-Mode Loopback Capture
FR3: Epic 1 - Format Negotiation & Resampling
FR4: Epic 1 - Master Volume & Instant A/B Bypass
FR5: Epic 2 - 7.1 Channel Spatialization Matrix
FR6: Epic 2 - Interactive Circular 3D Dial UI
FR7: Epic 2 - Individual Virtual Speaker Toggling & Gain
FR8: Epic 2 - Custom HRTF / SOFA Profile Loading (Generic KEMAR shipped)
FR9: Epic 3 - 31-Band Biquad Filter Engine
FR10: Epic 3 - Interactive Frequency Response Curve Visualizer
FR11: Epic 3 - Genre Presets & User Custom Presets
FR12: Epic 3 - Fast 10-Band / 31-Band Toggle
FR13: Epic 3 - Integrated Offline AutoEq Database
FR14: Epic 3 - Searchable Headphone Profile Selector
FR15: Epic 3 - Auto-Gain Compensation
FR16: Epic 4 - Ambience (Reverberation & Acoustic Space)
FR17: Epic 4 - Fidelity (Harmonic Exciter)
FR18: Epic 4 - Night Mode (Dynamic Range Compressor)
FR19: Epic 4 - Spatial (Stereo Expander & Crossfeed)
FR20: Epic 4 - Pitch (Real-Time Pitch Shifter - Phase 2 placeholder)
FR21: Epic 5 - Process Enumeration & Icon Display
FR22: Epic 5 - Independent Process Volume Faders
FR23: Epic 5 - Persistent Application Settings
FR24: Epic 1 - Lookahead Peak Limiter
FR25: Epic 1 - Real-Time Limiter Gain Reduction Indicator
FR26: Epic 1 - Dark Fluent Visual Theme
FR27: Epic 1 - Master Power Switch
FR28: Epic 1 - Windows System Tray & Background Run
FR29: Epic 1 - Windows Startup & Session Persistence

## Delivery Pillars Overview

### Epic 1: Desktop Shell & Driverless Audio Foundation
The user can launch OpenBoom in a sleek Dark Fluent desktop HUD, capture and stream Windows audio driverlessly with sub-15ms latency, adjust master volume (with clean 150% boost), toggle the master power switch, perform instant click-free A/B bypass comparisons, hear output protected by the master lookahead peak limiter (-0.2 dBFS), and minimize the app to the Windows system tray.
**FRs covered:** FR1, FR2, FR3, FR4, FR24, FR25, FR26, FR27, FR28, FR29.

### Epic 2: 3D Surround Soundstage & Spatial Calibration
The user can expand stereo headphone listening into an immersive 7.1 binaural soundstage using an interactive circular 3D dial knob, and open the 8-channel speaker configurator modal to mute, position, and trim individual virtual speaker levels in real time.
**FRs covered:** FR5, FR6, FR7, FR8.

### Epic 3: Studio Equalization & AutoEq Headphone Calibration
The user can search over 4,000 headphone models offline to apply Harman target parametric compensation with automatic headroom gain protection, sculpt frequency response across 10-band or 31-band faders with an interactive Bezier spline visualizer, and save custom genre presets.
**FRs covered:** FR9, FR10, FR11, FR12, FR13, FR14, FR15.

### Epic 4: Real-Time Sweeteners Rack & Dynamics Control
The user can toggle and dial in four psychoacoustic sound-enhancing DSP modules—Ambience room reverberation, Fidelity harmonic exciter, Night Mode dynamic range compression for movies/speech, and Spatial stereo widening—with zero latency degradation.
**FRs covered:** FR16, FR17, FR18, FR19, FR20.

### Epic 5: Per-Application Audio Session Mixer
The user can open a dedicated per-application mixer drawer to view all active Windows audio sessions with process icons, adjust individual application volumes independently up to 150%, mute specific apps, and persist custom volume preferences across reboots.
**FRs covered:** FR21, FR22, FR23.


## Epic 1: Desktop Shell & Driverless Audio Foundation

The user can launch OpenBoom in a sleek Dark Fluent desktop HUD, capture and stream Windows audio driverlessly with sub-15ms latency, adjust master volume (with clean 150% boost), toggle the master power switch, perform instant click-free A/B bypass comparisons, hear output protected by the master lookahead peak limiter (-0.2 dBFS), and minimize the app to the Windows system tray.

### Story 1.1: Project Workspace Scaffolding & Desktop Shell Window

As a listener,
I want to launch the OpenBoom application window with a dark studio fluent HUD and title controls,
So that I have a responsive, hardware-accelerated workspace matching the design system with instant startup and tray minimization.

**Acceptance Criteria:**

**Given** a clean development environment,
**When** the Cargo workspace is initialized,
**Then** the repository contains `crates/open-boom-core`, `crates/open-boom-wasapi`, `crates/open-boom-storage`, `src-tauri`, and `src-ui` configured according to `ARCHITECTURE-SPINE.md`.
**And** the application compiles cleanly with zero warnings under Rust 1.81.0, Node.js 24, and Tauri v2.

**Given** the application is launched,
**When** the main window appears,
**Then** the HUD renders at `840px × 560px` with dark surfaces (`#121316`, `#16171b`), custom titlebar, and smooth hover/active transitions without white flash glitches (`FR26`, `UX-DR1`, `UX-DR12`).

**Given** the main window is open,
**When** the user clicks the close button (`×`),
**Then** the window minimizes to the Windows notification tray, and clicking the tray icon restores the window in < 200ms (`FR28`).

### Story 1.2: Driverless WASAPI Audio Ingest & Output Engine

As a listener,
I want OpenBoom to capture system audio in user mode without kernel drivers and stream it to my headphones with sub-15ms latency,
So that I enjoy real-time playback without audio delay or system BSOD risk.

**Acceptance Criteria:**

**Given** OpenBoom is running on Windows 10 (19041+) or Windows 11 without administrative elevation,
**When** audio playback is detected from any application,
**Then** `crates/open-boom-wasapi` captures the stream via WASAPI Loopback (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` Mode B or Virtual Bridge Mode A) without installing or invoking `.sys` kernel drivers (`FR2`, `NFR8`, `AD-2`).

**Given** an incoming audio stream at 44.1 kHz, 48 kHz, 96 kHz, or 192 kHz,
**When** samples enter the ingestion adapter,
**Then** the stream is cleanly resampled to the internal 48,000 Hz 32-bit float (`f32`) format without audible clicks, pitch shifting, or buffer underruns (`FR3`, `AD-3`).

**Given** the audio render client is initialized,
**When** the audio thread registers with MMCSS (`AvSetMmThreadCharacteristicsW("Pro Audio")`),
**Then** the end-to-end capture-to-render round-trip latency remains < 15ms and callback duration takes < 60% of the buffer period (`NFR1`, `NFR3`).

### Story 1.3: Lock-Free POD Command Dispatcher & Master Power Control

As a listener,
I want a master power toggle on the HUD that activates or deactivates audio processing without clicks or latency,
So that I can easily engage OpenBoom or drop into zero-overhead passthrough.

**Acceptance Criteria:**

**Given** the UI state store and real-time audio thread,
**When** user actions occur in the UI,
**Then** commands are serialized as flat `#[repr(C)]` POD structs (`AudioCommand` enum) across a single-producer single-consumer (SPSC) lock-free ring buffer without heap allocations or JSON parsing on the audio thread (`AD-1`, `NFR2`).

**Given** audio is streaming through OpenBoom,
**When** the user clicks the glowing Master Power Switch (`48px × 26px`),
**Then** the switch animates with cyan glow (`rgba(56, 189, 248, 0.35)`) and toggles state instantly (`FR27`, `UX-DR2`).

**Given** the Master Power Switch is toggled OFF,
**When** audio continues to play,
**Then** the pipeline drops into bit-perfect zero-latency passthrough and DSP CPU usage drops to 0.0% (`FR27`, `NFR5`).

### Story 1.4: Master Volume, Boost Control & Device Selection

As a listener,
I want to adjust output volume from 0% to 150% and switch playback devices dynamically,
So that I can boost quiet streams cleanly and route audio to my desired headset.

**Acceptance Criteria:**

**Given** the master volume slider on the HUD,
**When** the user drags the slider between 0% and 100%,
**Then** volume adjusts smoothly with logarithmic attenuation, and values between 101% and 150% apply clean digital preamp gain (`FR4`, `UX-DR3`).

**Given** the user double-clicks the volume slider thumb,
**When** clicked,
**Then** the slider snaps to exactly 100% (0.0 dB) (`UX-DR3`).

**Given** multiple audio playback endpoints connected to the PC,
**When** the user clicks the output selector dropdown,
**Then** all active devices enumerated via `MMDeviceAPI` are listed, and selecting a new device transfers streaming within 500ms without crashing (`FR1`).

**Given** the active output device is unplugged during playback,
**When** disconnection occurs,
**Then** OpenBoom recovers within 500ms by falling back to the Windows default endpoint and surfacing an informative notification banner (`FR1`, `EXPERIENCE.md §5.1`).

### Story 1.5: Master Lookahead Peak Limiter & Telemetry Metering

As a listener,
I want an integrated lookahead peak limiter (-0.2 dBFS ceiling) and visual status bar reduction meter,
So that my hearing and equipment are protected against digital clipping distortion even during extreme volume boosts.

**Acceptance Criteria:**

**Given** high volume boosts (> 100%) or loud audio peaks entering the master output stage,
**When** samples pass through the mandatory terminal limiter,
**Then** a 2ms lookahead buffer with transparent release clamps inter-sample peaks strictly at -0.2 dBFS, guaranteeing 0% digital clipping and maximum output <= -0.1 dBFS (`FR24`, `NFR9`, `AD-5`).

**Given** active gain reduction is triggered by the limiter,
**When** attenuation occurs,
**Then** the status bar limiter badge pulses amber/red showing exact real-time dB gain reduction (`FR25`, `UX-DR10`).

**Given** the status bar on the bottom dock,
**When** the engine is streaming,
**Then** sample rate (`48 kHz`), bit depth (`32-bit Float`), measured latency, and limiter headroom are continuously displayed (`FR25`, `UX-DR10`).

### Story 1.6: Phase-Continuous Instant A/B Bypass

As a listener,
I want an instant A/B bypass button on the status bar that toggles between processed and raw audio with a 5ms crossfade,
So that I can objectively evaluate audio enhancements with zero clicks, phase jumps, or timing delays.

**Acceptance Criteria:**

**Given** OpenBoom is actively processing audio,
**When** the user clicks the `[A/B Bypass]` button or presses `Ctrl + Shift + B`,
**Then** audio transitions between processed and unprocessed sound via a 5ms equal-power crossfade retaining the 2ms lookahead delay (`FR4`, `AD-5`, `UX-DR10`).

**Given** the bypass state is toggled rapidly,
**When** listening through sensitive headphones,
**Then** zero audible DC pops, clicks, or volume jumps occur (`FR4`, `AD-5`).

**Given** bypass is engaged,
**When** inspecting the status bar,
**Then** the indicator displays `Hardware Passthrough (Bypassed)` in muted slate text (`EXPERIENCE.md §3`).

### Story 1.7: Application Preferences, Window Geometry & System Startup Persistence

As a listener,
I want OpenBoom to remember my volume, output device, window coordinates, and optional Windows startup preference,
So that my listening setup is automatically restored every time I turn on my PC.

**Acceptance Criteria:**

**Given** user adjustments to volume, output endpoint, and window position,
**When** settings change,
**Then** configurations are atomically written to `%APPDATA%\OpenBoom\config.json` via a temporary file and `MoveFileExW` replace, preventing corrupt state on unexpected shutdown (`FR29`, `AD-10`).

**Given** the Settings modal,
**When** the user toggles "Launch on Windows Startup",
**Then** a clean registry run key or startup shortcut is created without requiring administrative elevation (`FR29`).

**Given** OpenBoom is minimized to the system tray,
**When** the user right-clicks the tray icon,
**Then** a native context menu appears providing quick toggles for Master Power, Preset selection, Mute, and Exit (`FR28`).


## Epic 2: 3D Surround Soundstage & Spatial Calibration

The user can expand stereo headphone listening into an immersive 7.1 binaural soundstage using an interactive circular 3D dial knob, and open the 8-channel speaker configurator modal to mute, position, and trim individual virtual speaker levels in real time.

### Story 2.1: Binaural 7.1 HRTF Convolution Engine & Down-Folding Matrix

As a listener,
I want multichannel and stereo audio streams to be spatialized into binaural stereo using KEMAR HRTF impulse responses,
So that I hear distinct directional cues with realistic depth and elevation over standard headphones.

**Acceptance Criteria:**

**Given** an 8-channel (7.1) or stereo input stream,
**When** samples pass through Stage 2 of `crates/open-boom-core`,
**Then** the engine convolves each of the 8 virtual speaker channels (FL, C, FR, SL, SR, RL, RR, LFE) with calibrated generic KEMAR dummy-head binaural impulse response matrices using SSE2/AVX SIMD acceleration (`FR5`, `FR8`, `NFR6`, `AD-3`).

**Given** rear-channel spatialized audio,
**When** evaluated with binaural analysis,
**Then** discrete interaural time delays (ITD of 0.2–0.7ms) and pinna spectral notches are present, allowing listeners to clearly distinguish rear from frontal sounds (`FR5`).

**Given** the HRTF convolution runs continuously on the audio thread,
**When** processing a full 48 kHz buffer,
**Then** zero heap allocations occur and execution takes < 25% of the buffer period (`NFR1`, `NFR2`).

### Story 2.2: Interactive Circular 3D Surround Dial Control

As a listener,
I want to rotate a circular 3D dial on the main HUD to modulate spatializer width and intensity from 0% to 100%,
So that I can dial in the ideal amount of immersion with immediate visual and acoustic feedback.

**Acceptance Criteria:**

**Given** the central 3D Surround Dial component (`192px` outer size, `144px` knob surface) on the HUD,
**When** the user drags the outer rim or knob,
**Then** the active cyan arc rotates from 0° (0% intensity) to 300° (100% intensity) following polar coordinates theta = atan2(y - y0, x - x0) with detent pips lighting up at 25%, 50%, and 75% (`FR6`, `UX-DR4`).

**Given** the user scrolls the mouse wheel over the dial,
**When** scrolled,
**Then** intensity steps by 2% per notch (or 5% when holding Shift), and double-clicking the knob center snaps intensity back to default 50% (`UX-DR4`).

**Given** dial rotations while audio is playing,
**When** parameter updates are dispatched to the engine,
**Then** linear interpolation smoothes wet/dry spatializer coefficients across samples, preventing zipper noise or audio stutter (`AD-4`).

### Story 2.3: 8-Channel Virtual Speaker Configurator Soundstage Modal

As a gamer or cinema watcher,
I want to open a top-down polar soundstage modal to mute individual virtual speakers and adjust their relative trim gains,
So that I can isolate dialogue, boost rear footsteps, or customize surround channel balance.

**Acceptance Criteria:**

**Given** the 3D Surround Dial on the HUD,
**When** the user clicks the speaker calibration icon,
**Then** a modal opens displaying a top-down polar soundstage with a central listener icon and 8 virtual speaker nodes (`UX-DR5`).

**Given** any virtual speaker node (e.g. Center, Rear Left, LFE Subwoofer),
**When** the user clicks the node,
**Then** its active state toggles; muted speakers render dimmed with a red strikethrough indicator and immediately drop from the spatial convolution mix (`FR7`, `UX-DR5`).

**Given** an active speaker node,
**When** the user adjusts its gain fader,
**Then** relative channel gain adjusts smoothly within +/-6.0 dB at 0.1 dB resolution (`FR7`).

**Given** the configurator modal is open,
**When** the user presses Esc, clicks the backdrop, or clicks Done,
**Then** the modal dismisses cleanly and saves channel trim configurations (`EXPERIENCE.md §2`).

### Story 2.4: Spatial Virtual Speaker Angular Repositioning

As a spatial audio enthusiast,
I want to drag virtual speaker nodes along orbital paths to adjust their azimuth angles in the HRTF matrix,
So that I can tailor the virtual soundstage layout to my personal hearing geometry.

**Acceptance Criteria:**

**Given** the 8-channel polar soundstage modal,
**When** the user drags a speaker node along its radial orbit ring,
**Then** the node slides smoothly and updates its real-time azimuth coordinate angle in the HUD display (`FR7`, `UX-DR5`).

**Given** an updated azimuth angle for any speaker,
**When** the command reaches the audio engine via the SPSC queue,
**Then** the HRTF interpolation tables dynamically adjust the binaural convolution filter coefficients for that channel without dropouts (`AD-1`, `AD-4`).

**Given** custom speaker positions have been adjusted,
**When** the user clicks "Reset Speaker Positions",
**Then** all 8 speaker nodes return to standard ITU-R BS.775 7.1 reference azimuth angles (`EXPERIENCE.md §4.7`).


## Epic 3: Studio Equalization & AutoEq Headphone Calibration

The user can search over 4,000 headphone models offline to apply Harman target parametric compensation with automatic headroom gain protection, sculpt frequency response across 10-band or 31-band faders with an interactive Bezier spline visualizer, and save custom genre presets.

### Story 3.1: 31-Band Biquad Filter Engine & Direct Form II Transposed Topology

As a listener,
I want a high-precision 31-band equalizer engine operating at ISO 1/3-octave frequencies from 20 Hz to 20 kHz,
So that I can adjust frequency response with ultra-low distortion (THD+N < 0.005%) and zero filter explosions during parameter changes.

**Acceptance Criteria:**

**Given** Stage 3 of `crates/open-boom-core`,
**When** audio samples pass through the graphic equalizer bank,
**Then** 31 second-order IIR peaking filters calculated via Robert Bristow-Johnson biquad formulas (constant Q = 4.31) process stereo samples with THD+N < 0.005% using SSE2/AVX vector intrinsics (`FR9`, `NFR6`, `AD-3`).

**Given** user slider movements in the UI,
**When** filter coefficients are recalculated,
**Then** updates execute at sub-buffer block boundaries (every 32 or 64 samples) using Direct Form II Transposed topology with smoothed target gain, guaranteeing filter poles strictly remain within the unit circle (`AD-4`).

**Given** any individual band slider,
**When** gain is adjusted,
**Then** the band operates over a range of -12.0 dB to +12.0 dB at 0.1 dB resolution (`FR9`).

### Story 3.2: Dual-Mode Equalizer Studio Drawer (10-Band vs 31-Band) & Spline Curve Visualizer

As a listener,
I want an expandable equalizer drawer with a toggle between a 10-band casual view and a 31-band studio fader bank, plus an interactive Bezier spline curve visualizer,
So that I can sculpt sound visually with 0.1 dB precision, magnetic 0 dB snaps, and double-click resets.

**Acceptance Criteria:**

**Given** the Equalizer preview dock on the main HUD,
**When** the user clicks "Expand EQ" or presses `Ctrl + Shift + E`,
**Then** the full-height Equalizer Studio Drawer slides open with smooth hardware acceleration (`UX-DR7`, `EXPERIENCE.md §4.5`).

**Given** the mode toggle in the drawer header,
**When** the user switches between "10-Band" and "31-Band",
**Then** the UI transitions smoothly while preserving underlying frequency curve values without resetting faders (`FR12`, `UX-DR7`).

**Given** vertical fader sliders (`180px` stroke),
**When** dragging,
**Then** a floating Cascadia Code tooltip displays current dB value; holding `Shift` enables 5x precision mode; passing within +/-0.3 dB applies a magnetic detent snap to `0.0 dB`; and double-clicking snaps directly to `0.0 dB` (`UX-DR7`, `EXPERIENCE.md §6.2`).

**Given** fader adjustments,
**When** sliders move,
**Then** the canvas above the faders immediately renders a smooth Bezier spline curve of the cumulative frequency response (`FR10`, `UX-DR7`).

### Story 3.3: Offline AutoEq Headphone Database Indexing & Instant Fuzzy Search

As a headphone user,
I want to search over 4,000 headphone models in an offline bundled database with instant fuzzy search results (<50ms),
So that I can find and inspect my exact headphone model without an internet connection or telemetry.

**Acceptance Criteria:**

**Given** `crates/open-boom-storage`,
**When** the crate is packaged,
**Then** it bundles a pre-compiled, compressed binary/JSON index of 4,000+ AutoEq profiles measuring < 5 MB total on disk (`FR13`, `AD-9`).

**Given** the Headphone Profile picker in the EQ dock,
**When** the user clicks it,
**Then** a search popover appears with an autocomplete input field (`UX-DR8`).

**Given** user typing into the search field (e.g. "HyperX", "HD 600", "Sony XM5"),
**When** queries are executed against the in-memory index,
**Then** fuzzy-matched results return in < 50ms completely offline, displaying brand, model, measurement target (e.g. Oratory1990 - Harman Target), and filter count (`FR14`, `NFR7`, `NFR10`, `UX-DR8`).

### Story 3.4: Decoupled Parametric AutoEq Stage & Auto-Gain Preamp Compensation

As an audiophile,
I want OpenBoom to apply up to 12 parametric biquad correction filters for my headphone model while automatically attenuating preamp gain to compensate for positive boosts,
So that my headphones achieve a flat Harman target response without clipping or clashing with my custom 31-band EQ settings.

**Acceptance Criteria:**

**Given** a selected AutoEq profile,
**When** the user clicks "Apply Profile",
**Then** up to 12 second-order parametric IIR biquad filters are loaded into the Parametric Correction Stage sequentially *before* the 31-band graphic equalizer (`AD-6`).

**Given** an AutoEq profile with positive gain peaks (e.g. +6.5 dB peak at 3.2 kHz),
**When** the snapshot is committed,
**Then** the UI Control Store automatically calculates an equal negative preamp attenuation offset (`preamp_offset_db = -max_positive_gain_db`) and transmits it atomically in the snapshot (`FR15`, `AD-6`).

**Given** both an AutoEq profile and custom 31-band fader boosts are active simultaneously,
**When** audio processes through the pipeline,
**Then** headroom attenuation prevents intermediate digital clipping before the master limiter stage (`FR15`, `AD-6`).

### Story 3.5: Genre Factory Presets & Custom Preset Persistence

As a listener,
I want to select from 15 factory genre presets and save, rename, or delete my own custom EQ presets stored safely in JSON,
So that I can switch profiles instantly for gaming, EDM, classical music, or movies.

**Acceptance Criteria:**

**Given** the preset selector dropdown on the main HUD,
**When** opened,
**Then** at least 15 factory presets are available: Flat, Bass Boost, Treble Boost, Acoustic, Classical, Electronic, Gaming, Hip-Hop, Jazz, Movie, Pop, Rock, Vocal, Lounge, Deep (`FR11`).

**Given** the user alters faders while on a preset,
**When** changes occur,
**Then** the preset label displays an asterisk (e.g. `Rock *`) and enables "Save As Custom Preset" (`EXPERIENCE.md §5.2`).

**Given** a custom preset is saved or deleted,
**When** saved,
**Then** `crates/open-boom-storage` writes to `%APPDATA%\OpenBoom\presets\` using atomic temporary-file writes and `MoveFileExW` replacement (`FR11`, `AD-10`).


## Epic 4: Real-Time Sweeteners Rack & Dynamics Control

The user can toggle and dial in four psychoacoustic sound-enhancing DSP modules—Ambience room reverberation, Fidelity harmonic exciter, Night Mode dynamic range compression for movies/speech, and Spatial stereo widening—with zero latency degradation.

### Story 4.1: 5-Effect Quick Rack Strip UI Component

As a listener,
I want a horizontal strip of modular effect cards on the HUD (`Ambience`, `Fidelity`, `Night Mode`, `Spatial`, `Pitch`),
So that I can toggle effects on/off and adjust intensities with dedicated mini-sliders and neon active borders.

**Acceptance Criteria:**

**Given** the main HUD below the 3D Surround Dial,
**When** rendered,
**Then** a horizontal rack displays 5 standard modular cards (`{components.effect-card}`) styled according to `DESIGN.md`: Ambience, Fidelity, Night Mode, Spatial, and Pitch (`UX-DR6`).

**Given** any effect card,
**When** clicking the card header or power icon,
**Then** its active state toggles immediately with a cyan active border (`#38bdf8`) and illuminated indicator icon (`UX-DR6`).

**Given** the Pitch card on the rack,
**When** viewed,
**Then** it displays semitone transposition (`-12 st` to `+12 st`) with a Phase 2 "Coming Soon" badge, gracefully preserving layout without invoking unimplemented DSP routines (`FR20`, `PRD §6.2`).

**Given** intensity mini-sliders,
**When** dragging or scrolling,
**Then** values update from 0% to 100% and dispatch atomic POD command messages to the engine (`AD-1`, `UX-DR6`).

### Story 4.2: Ambience Reverb Module (Schroeder-Moorer / FDN DSP)

As a music and movie enthusiast,
I want an Ambience effect simulating natural room reflections and acoustic stage depth,
So that dry recordings sound spacious and airy without metallic ringing or phase issues.

**Acceptance Criteria:**

**Given** audio passing through Stage 4 of `crates/open-boom-core`,
**When** Ambience is enabled,
**Then** a feedback delay network (FDN) / Schroeder-Moorer reverb matrix generates early reflections and smooth stereo diffusion using SSE2/AVX vector intrinsics (`FR16`, `NFR6`).

**Given** the Ambience intensity slider (0% to 100%),
**When** adjusted,
**Then** the wet/dry mix ratio modulates smoothly with linear parameter interpolation without audio dropouts or metallic comb-filtering (`FR16`, `AD-4`).

**Given** monaural or center-panned audio signals,
**When** processed through Ambience,
**Then** reverberation tails remain phase-coherent in both channels (`FR16`).

### Story 4.3: Fidelity Psychoacoustic Harmonic Exciter Module

As a streaming listener,
I want a Fidelity effect that synthesizes subtle even and odd harmonics and high-shelf air,
So that compressed audio streams (YouTube, Spotify) regain upper-frequency brilliance and vocal presence.

**Acceptance Criteria:**

**Given** compressed audio streams with roll-off above 10 kHz,
**When** Fidelity is engaged,
**Then** a psychoacoustic waveshaper synthesizes subtle 2nd and 3rd harmonic overtones and boosts upper-treble presence (`FR17`).

**Given** the Fidelity intensity slider (0% to 100%),
**When** modulated,
**Then** excitation depth increases smoothly without introducing harsh sibilance, aliasing distortion, or clipping (`FR17`, `NFR9`).

**Given** continuous streaming on the real-time audio thread,
**When** Fidelity processes buffers,
**Then** zero allocations occur and latency impact is 0.0ms (`NFR1`, `NFR2`).

### Story 4.4: Night Mode Dynamic Range Compressor (DRC)

As a late-night movie watcher or gamer,
I want a Night Mode soft-knee dynamic compressor with fast attack and dialogue leveling,
So that sudden explosions and gunshots are automatically tamed while quiet whispers remain clearly audible.

**Acceptance Criteria:**

**Given** high-dynamic-range cinema audio (e.g. explosive action scene),
**When** Night Mode is active,
**Then** an RMS/peak dynamic compressor engages with fast attack (< 5ms), program-dependent release, and soft-knee curvature (`FR18`, `PRD §4.5`).

**Given** loud transient signals exceeding -12 dBFS,
**When** processed,
**Then** peaks are smoothly attenuated by up to 18 dB, while low-level signals below -30 dBFS receive up to +6 dB clean dialogue boost (`FR18`).

**Given** Night Mode intensity adjusted from 0% to 100%,
**When** changed,
**Then** the dynamic compression threshold scales smoothly from 0 dBFS down to -30 dBFS without audible pumping artifacts (`FR18`, `AD-4`).

### Story 4.5: Spatial Stereo Expander & Crossfeed Matrix

As a headphone listener,
I want a Spatial stereo widening effect based on Mid/Side processing with frequency-selective crossfeed,
So that the soundstage expands beyond my ears while center vocal integrity is strictly preserved.

**Acceptance Criteria:**

**Given** a stereo audio stream,
**When** Spatial is enabled,
**Then** the engine decomposes the signal into Mid ($L+R$) and Side ($L-R$) components and boosts the Side component with frequency-selective crossfeed filtering (`FR19`, `PRD §4.5`).

**Given** pure monaural audio ($L = R$),
**When** Spatial intensity is set to 100%,
**Then** the mono signal passes through with zero volume drop and zero phase cancellation artifacts (`FR19`).

**Given** stereo music playback,
**When** Spatial intensity is adjusted from 0% to 100%,
**Then** soundstage width widens smoothly beyond headphone earcups while vocal center anchoring remains solid (`FR19`, `AD-4`).


## Epic 5: Per-Application Audio Session Mixer

The user can open a dedicated per-application mixer drawer to view all active Windows audio sessions with process icons, adjust individual application volumes independently up to 150%, mute specific apps, and persist custom volume preferences across reboots.

### Story 5.1: Windows Core Audio Session Manager & Dedicated COM MTA Thread

As a listener,
I want OpenBoom to discover and track all active Windows audio sessions via a dedicated COM MTA background thread,
So that running processes producing sound are monitored in real-time without COM apartment crashes or RPC thread locks.

**Acceptance Criteria:**

**Given** `crates/open-boom-wasapi`,
**When** initializing the session manager,
**Then** a dedicated worker thread initializes COM with `CoInitializeEx(None, COINIT_MULTITHREADED)` to interact safely with `IAudioSessionManager2` (`AD-8`).

**Given** applications playing audio (e.g., `chrome.exe`, `discord.exe`, `spotify.exe`),
**When** audio streams initiate,
**Then** the session manager discovers them within 1 second and streams updates to the UI containing process ID, executable name, and active audio state (`FR21`, `PRD §4.6`).

**Given** an audio-producing process exits,
**When** the audio session terminates,
**Then** the process is cleanly removed from the active session cache without resource leaks or thread hangs (`FR21`).

### Story 5.2: Per-Application Mixer Drawer UI & Individual Process Faders

As a multitasking user,
I want to open a side drawer in OpenBoom displaying each audio application with its name, icon, activity indicator, and a 0% to 150% volume slider plus mute button,
So that I can adjust individual app volumes (e.g. lower browser, boost Discord) without digging into Windows settings.

**Acceptance Criteria:**

**Given** the Header on the main HUD,
**When** the user clicks the "App Volumes" icon or presses the mixer shortcut,
**Then** the Application Stream Mixer Drawer slides open (`UX-DR9`, `EXPERIENCE.md §2`).

**Given** the list of active applications,
**When** rendered,
**Then** each row (`{components.app-mixer-row}`) displays the application executable icon, process name, an animated green playback indicator, an independent 0% to 150% volume slider, and a mute button (`FR21`, `FR22`, `UX-DR9`).

**Given** the user adjusts an application slider (e.g. lowers Discord to 40%),
**When** the fader moves,
**Then** `ISimpleAudioVolume::SetMasterVolume` executes on the dedicated MTA thread, immediately updating the target application's volume without affecting global master volume (`FR22`, `AD-8`).

**Given** the user clicks the mute button on an app row,
**When** clicked,
**Then** `ISimpleAudioVolume::SetMute` toggles the mute state for that application session (`FR22`).

### Story 5.3: Persistent Application Volume Preferences & Automatic Session Recovery

As a user,
I want OpenBoom to remember custom volume offsets and mute states for specific applications across system reboots and app restarts,
So that my tailored balance between games, music, and voice chat is automatically maintained.

**Acceptance Criteria:**

**Given** customized volume sliders or mute states for specific applications,
**When** settings change,
**Then** `crates/open-boom-storage` saves application volume profiles to `%APPDATA%\OpenBoom\app_volumes.json` using atomic temporary-file writes (`FR23`, `AD-10`).

**Given** an application is closed and reopened later (or the PC is rebooted),
**When** the application initiates audio playback again,
**Then** OpenBoom matches the process executable name and automatically restores its previously configured volume level and mute state (`FR23`, `UJ-4`).
