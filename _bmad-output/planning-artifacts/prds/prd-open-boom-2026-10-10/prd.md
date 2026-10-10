---
title: OpenBoom
status: final
created: 2026-10-10
updated: 2026-10-10
---

# PRD: OpenBoom
*Studio-Grade 3D Audio Virtualization, Multiband Equalization, and Real-Time Audio Sweetening for Windows.*

---

## 0. Document Purpose

This Product Requirements Document (PRD) defines the functional requirements, performance budgets, user experience flows, and system boundaries for **OpenBoom** — an open-source, lightweight desktop audio enhancer for Windows. OpenBoom delivers complete 1:1 functional parity with the core capabilities of **Boom 3D** (Global Delight), replacing proprietary kernel drivers and commercial bloatware with transparent user-mode audio pipelines, sub-15ms processing latency, zero telemetry, and native AutoEq integration.

This document builds directly upon foundational technical blueprints in [.bmad/00-PROJECT-BRIEF.md](file:///c:/Users/joaov/Documents/GitHub/open-boom/.bmad/00-PROJECT-BRIEF.md), [.bmad/01-ARCHITECTURE.md](file:///c:/Users/joaov/Documents/GitHub/open-boom/.bmad/01-ARCHITECTURE.md), and [.bmad/04-DECISION-LOG.md](file:///c:/Users/joaov/Documents/GitHub/open-boom/.bmad/04-DECISION-LOG.md). Architectural trade-offs, reverse-engineering evidence from Boom 3D binaries, and DSP implementation notes are cataloged in [addendum.md](file:///c:/Users/joaov/Documents/GitHub/open-boom/_bmad-output/planning-artifacts/prds/prd-open-boom-2026-10-10/addendum.md).

---

## 1. Vision

Modern PC audio is compromised by low-cost laptop DACs, suboptimal default headphone tunings, and flat, uninspired stereo soundstages. Commercial audio enhancement software like Boom 3D and SteelSeries Sonar introduced compelling spatial simulation and frequency shaping, but at a punishing price: proprietary kernel drivers prone to Blue Screen of Death (BSOD) crashes, high buffer latency (40–80ms) unsuitable for competitive gaming, aggressive monetization nag screens, mandatory account creation, and intrusive background telemetry. Conversely, existing open-source solutions (Equalizer APO with Peace) provide low latency but suffer from intimidating setup friction, fragile installation steps, and an outdated visual aesthetic.

**OpenBoom** bridges this gap. It delivers the beloved, tactile visual experience of Boom 3D — centered around an interactive 3D surround dial, 31-band EQ, AutoEq calibration, and a 5-effect real-time DSP rack — engineered as an ultra-fast, driverless Windows application. By routing audio via user-mode WASAPI loopback streams with SIMD-accelerated 32-bit floating-point buffers, OpenBoom achieves sub-15ms round-trip latency, consumes under 40 MB of RAM, and respects complete user privacy.

---

## 2. Target User & Journeys

### 2.1 Jobs To Be Done (JTBD)
- **Immersive Gamer:** *"When I play competitive or immersive games, I want accurate 360-degree positional audio cues (footsteps, reloads, distance) without perceptible audio delay, so that I can react instantly without gaming performance penalties."*
- **Headphone Audiophile & Music Enthusiast:** *"When listening to music on my specific headphones, I want automatic parametric frequency correction to the Harman target curve and rich harmonic excitation, so that my headphones sound like high-end planar magnetic studio monitors."*
- **Media Consumer & Binge-Watcher:** *"When watching movies or late-night streams with loud explosions and quiet dialogue, I want dynamic range leveling and acoustic stage expansion, so that dialogue is crystal clear without blasting the neighborhood."*
- **Multitasking Power User:** *"When running Discord, Spotify, and a game simultaneously, I want quick, independent volume sliders per application in one clean dashboard, so that I don't have to fiddle with Windows Sound settings."*

### 2.2 Non-Users (v1)
- **Professional Music Producers / DAW Mastering Engineers:** OpenBoom is designed for consumer listening enhancement and real-time playback, not for recording studios requiring ASIO bit-perfect output with zero coloration.
- **Kernel-Level Audiophile Purists:** Users who refuse any software processing and insist strictly on hardware-only external DAC/DSP units.

### 2.3 Key User Journeys

#### UJ-1: Alex Calibrates New Headphones in 10 Seconds
- **Persona + Context:** Alex bought a pair of HyperX Cloud II headphones. The sound is slightly muffled in the mids and harsh in the highs.
- **Entry state:** OpenBoom is running in the Windows System Tray. Audio is playing via Spotify.
- **Path:** Alex clicks the OpenBoom tray icon. The sleek dark interface appears instantly. Alex types "HyperX Cloud II" into the Headphone Profile search bar. The auto-complete dropdown suggests the matching AutoEq entry. Alex clicks it.
- **Climax:** The DSP engine immediately activates the multi-band biquad compensation curve. The muddiness vanishes; vocals snap into crisp focus.
- **Resolution:** Alex toggles the A/B comparison switch once to verify the improvement, smiles, and minimizes the window back to the tray.
- **Edge case:** If the exact headphone revision is missing, OpenBoom offers a fallback to general Over-Ear, On-Ear, or In-Ear compensation curves.

#### UJ-2: Marcus Configures 3D Positional Surround for Gaming
- **Persona + Context:** Marcus is playing a tactical first-person shooter. Standard stereo makes it hard to distinguish rear sounds from front sounds.
- **Entry state:** OpenBoom is active on the default gaming headset endpoint.
- **Path:** Marcus opens OpenBoom and clicks the central **3D Surround Dial**. An interactive circular soundstage appears displaying 8 virtual speakers (FL, FR, Center, LFE, Surround L/R, Rear L/R). Marcus rotates the intensity knob to 85% and clicks the Rear Left and Rear Right speakers to boost their relative gain by +3 dB.
- **Climax:** In-game enemy footsteps approaching from behind are instantly distinguishable in distance and elevation with zero perceptible delay (<15ms).
- **Resolution:** Marcus saves this setup as a preset named "Tactical Shooter" and assigns a global hotkey to toggle it.

#### UJ-3: Elena Watches Late-Night Cinema Without Volume Jumps
- **Persona + Context:** Elena is watching an action blockbuster at 1:00 AM while family members are asleep.
- **Entry state:** Movie playing with 5.1 multichannel audio stream.
- **Path:** Elena opens OpenBoom, toggles **Night Mode** to ON, and adjusts the leveling intensity slider to 70%. She also enables **Ambience** at 20% for subtle room warmth.
- **Climax:** Whispered dialogue is automatically amplified while gunfire and explosion peaks are gently compressed with a soft knee. She never has to ride the volume slider.
- **Resolution:** Seamless listening without digital distortion or ear fatigue, guarded by the built-in Master Peak Limiter.

#### UJ-4: David Balances Voice Chat and Background Music
- **Persona + Context:** David is in a Discord voice channel while playing an RPG with background lo-fi music on browser.
- **Entry state:** Multiple applications producing audio simultaneously.
- **Path:** David clicks the **Applications** tab in OpenBoom's top bar. A live list of audio-producing processes appears with per-app faders. David lowers the browser volume to 40% and boosts Discord to 100%.
- **Climax:** Voice chat is clear without lowering master system volume or navigating deep Windows settings.
- **Resolution:** Application volume preferences persist across reboots.

---

## 3. Glossary

- **OpenBoom Core:** The main application runtime encapsulating the UI frontend and the DSP processing engine.
- **WASAPI Loopback:** Windows Audio Session API capture mode operating in user-space, intercepting audio samples rendered to an audio device without requiring kernel drivers.
- **WASAPI Process Loopback:** Process-specific audio capture (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS`) available on Windows 10 build 20348+ and Windows 11, capturing target application streams without re-capturing OpenBoom's output.
- **HRTF (Head-Related Transfer Function):** Mathematical response matrix characterizing how an ear receives sound from a specific point in 3D space, incorporating interaural time delay (ITD) and interaural level difference (ILD).
- **3D Surround Dial:** The interactive circular UI component representing the spatial soundfield and virtual speaker positions.
- **Virtual Speaker:** A simulated acoustic source point (FL, FR, C, LFE, SL, SR, RL, RR) in the 7.1 virtualization matrix.
- **AutoEq:** Open-source project and database containing frequency response compensation measurements and biquad filter targets for over 4,000 headphone models.
- **Biquad Filter:** A second-order IIR (Infinite Impulse Response) digital filter providing peaking, low-shelf, high-shelf, or notch frequency responses.
- **Fidelity Exciter:** A psychoacoustic DSP module that synthesizes subtle even and odd harmonics and boosts upper-treble presence.
- **Ambience Module:** A reverb/delay convolution network simulating acoustic room reflections and stage widening.
- **Night Mode (DRC):** Dynamic Range Compression unit featuring fast attack, program-dependent release, and soft-knee leveling.
- **Spatial Expander:** A Mid/Side stereo processing matrix with crossfeed filtering to widen the soundstage beyond headphone earcups.
- **Pitch Shifter:** A phase vocoder / delay-modulation DSP block altering musical key/pitch by ± semitones without tempo modification.
- **Master Peak Limiter:** A lookahead digital brickwall peak limiter ensuring the summed audio signal does not exceed 0 dBFS, preventing digital clipping.
- **Per-App Mixer:** Process-level volume and mute management interfaced via Windows `IAudioSessionManager2`.
- **A/B Bypass:** Instant toggle allowing the listener to compare processed audio against raw un-equalized passthrough.

---

## 4. Features & Functional Requirements

```
+---------------------------------------------------------------------------------------+
|                                  OPENBOOM MAIN WINDOW                                 |
|                                                                                       |
|  [Master Power] [Vol Slider: 78%]   [Output: HyperX Cloud II v]   [App Mixer Tab]   [-] [x]
+---------------------------------------------------------------------------------------+
|                                                                                       |
|                              ( ( (   3D SURROUND DIAL   ) ) )                         |
|                                   [ FL ]   [ C ]   [ FR ]                             |
|                              [ SL ]       ( 85% )       [ SR ]                        |
|                                   [ RL ]   [LFE]   [ RR ]                             |
|                                                                                       |
|                     [ 3D Toggle: ON ]       [ Speaker Config Modal ]                  |
|                                                                                       |
+---------------------------------------------------------------------------------------+
|  EFFECTS RACK                                                                         |
|  [ Ambience: 20% ] [ Fidelity: 65% ] [ Night Mode: 70% ] [ Spatial: 40% ] [ Pitch: 0 ]|
+---------------------------------------------------------------------------------------+
|  EQUALIZER & AUTOEQ                                                                   |
|  Headphone Profile: [ HyperX Cloud II (AutoEq) v ]            Preset: [ Rock / EDM v ] |
|  [ 31-Band Graph Visualization / Sliders: -12dB ... +12dB ]             [ A/B Bypass ] |
+---------------------------------------------------------------------------------------+
|  STATUS: 48kHz | 32-bit Float | Latency: 9.2ms | Limiter Headroom: -0.2 dB            |
+---------------------------------------------------------------------------------------+
```

### 4.1 Master Audio Engine & Device I/O
**Description:** Manages audio endpoint enumeration, stream capture, format negotiation, and output rendering. Intercepts system audio in real-time via user-mode WASAPI Loopback, processes samples through the DSP chain, and outputs to the user-selected playback device with sub-15ms buffer latency. Realizes UJ-1, UJ-2, UJ-3.

#### FR-1: Audio Device Discovery & Hot-Plug Monitoring
The system shall enumerate all active Windows audio render endpoints via `MMDeviceAPI` and detect device connection/disconnection events dynamically.
**Consequences (testable):**
- System updates the device dropdown within 500ms of plugging in a USB headset or unplugging Bluetooth headphones.
- If the active endpoint is disconnected, the engine gracefully falls back to the system default device without crashing or hanging the audio thread.

#### FR-2: Driverless User-Mode Loopback Capture
The system shall capture audio in user-mode using WASAPI Process Loopback (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS`) on Windows 11 and Windows 10 (build 20348+), eliminating routing feedback loops without kernel drivers. For legacy Windows 10 releases (builds 1903–2004), an optional open-source virtual audio cable bridge fallback is provided. Realizes UJ-1, UJ-3. `[ASSUMPTION: Process Loopback Primary]`
**Consequences (testable):**
- Application launches and captures audio on standard, non-elevated user accounts.
- Zero kernel-mode drivers (`.sys`) are installed or executed.

#### FR-3: Format Negotiation & Resampling
The system shall accept input streams at arbitrary sample rates (44.1 kHz, 48 kHz, 96 kHz, 192 kHz) and channel configurations (Stereo 2.0, Quadraphonic, 5.1, 7.1) and resample them cleanly to the internal processing format (48 kHz, 32-bit float).
**Consequences (testable):**
- Playing a 96 kHz FLAC file alongside a 48 kHz video stream produces artifact-free audio without pitch shifting or buffer underruns.

#### FR-4: Master Volume & Instant A/B Bypass
The user can adjust global output volume (0% to 100%) and trigger an instant A/B Bypass to compare processed audio directly with raw unprocessed audio.
**Consequences (testable):**
- Clicking A/B Bypass toggles DSP coloration within 1 audio buffer (<10ms) without audible clicks, pops, or volume surges.

---

### 4.2 3D Surround Virtualization & Channel Controller
**Description:** Simulates an immersive 7.1 multichannel soundstage over stereo headphones using HRTF convolution, interaural delay (ITD), and head shadowing (ILD). Features an interactive circular 3D Surround Dial identical in ergonomics to Boom 3D. Realizes UJ-2.

#### FR-5: 7.1 Channel Spatialization Matrix
The system shall fold 8 virtual speaker positions (Front Left, Center, Front Right, Surround Left, Surround Right, Rear Left, Rear Right, Subwoofer/LFE) into binaural stereo using HRTF filter matrices.
**Consequences (testable):**
- Discrete rear-channel test audio generates measurable interaural time delays (0.2–0.7ms) and high-frequency spectral notches characteristic of rear elevation.

#### FR-6: Interactive Circular 3D Dial UI
The user can drag a circular dial knob to modulate 3D spatializer intensity from 0% (standard stereo) to 100% (expanded binaural soundfield). Realizes UJ-2.
**Consequences (testable):**
- Dial responds to mouse drags, touch, and scroll wheel with immediate visual feedback and continuous audio parameter updates without audio stutter.

#### FR-7: Individual Virtual Speaker Toggling & Gain
The user can open a speaker configuration panel and click on individual virtual speakers to mute them or adjust their relative gain (±6 dB). Realizes UJ-2.
**Consequences (testable):**
- Clicking the Center speaker toggles dialogue track isolation/attenuation in multichannel streams.
- Clicking the Subwoofer (LFE) toggles low-end bass augmentation.

#### FR-8: Custom HRTF / SOFA Profile Loading
Advanced users can import standard Spatially Oriented Format for Acoustics (`.sofa`) measurement files to tailor the HRTF response to their personal anatomical geometry. Realizes UJ-2. `[ASSUMPTION: Generic KEMAR HRTF Default]`
**Out of Scope for MVP:** Shipped default HRTF profile is optimized for standard generic KEMAR dummy-head measurements; custom `.sofa` file import is deferred to Phase 2.

---

### 4.3 Multiband Graphic & Parametric Equalizer
**Description:** High-precision frequency equalization engine providing 31-band graphic control, interactive curve display, and curated genre presets. Realizes UJ-1, UJ-3.

#### FR-9: 31-Band Biquad Filter Engine
The system shall provide 31 standard 1/3-octave frequency bands ranging from 20 Hz to 20,000 Hz, each adjustable from -12.0 dB to +12.0 dB with 0.1 dB resolution.
**Consequences (testable):**
- Frequency bands implement second-order IIR peaking filters calculated via Robert Bristow-Johnson Audio EQ Cookbook biquad formulas.
- Total harmonic distortion (THD+N) introduced by the filters remains < 0.005%.

#### FR-10: Interactive Frequency Response Curve Visualizer
The user can view a real-time spline curve representing the combined frequency response across all bands, with draggable control points. Realizes UJ-1.
**Consequences (testable):**
- Moving any band slider immediately renders the updated curve with smooth Bezier interpolation.

#### FR-11: Genre Presets & User Custom Presets
The system shall provide at least 15 factory presets (Flat, Bass Boost, Treble Boost, Acoustic, Classical, Electronic, Gaming, Hip-Hop, Jazz, Movie, Pop, Rock, Vocal, Lounge, Deep) and allow users to save, rename, and delete custom presets.
**Consequences (testable):**
- Presets persist across application sessions in human-readable JSON files in `%APPDATA%\OpenBoom\presets\`.

#### FR-12: Fast 10-Band / 31-Band Toggle
The user can switch the equalizer view between a streamlined 10-band octave view and the advanced 31-band 1/3-octave view.
**Consequences (testable):**
- Switching views preserves the underlying frequency profile without resetting slider values.

---

### 4.4 AutoEq Headphone Profile Calibration
**Description:** Integrates the open-source AutoEq database containing pre-calculated equalization profiles for thousands of headphone models, correcting frequency nonlinearities to target curves (Harman Target). Realizes UJ-1.

#### FR-13: Integrated Offline AutoEq Database
The system shall package an offline index of over 4,000 measured headphone models (Over-Ear, In-Ear, On-Ear) from the AutoEq repository. Realizes UJ-1. `[ASSUMPTION: Offline AutoEq Bundling]`
**Consequences (testable):**
- Full search and curve loading operates 100% offline without requiring internet access or external API calls.
- Total binary size of the bundled compressed database is < 5 MB.

#### FR-14: Searchable Headphone Profile Selector
The user can type manufacturer or model names into a search field to filter and select their exact headphone model. Realizes UJ-1.
**Consequences (testable):**
- Search queries return instant fuzzy-matched results within < 50ms for a library of 4,000+ entries.

#### FR-15: Auto-Gain Compensation
When applying an AutoEq profile with positive gain peaks, the system shall automatically calculate and apply preamp attenuation to prevent digital clipping before the limiter stage.
**Consequences (testable):**
- If an AutoEq profile has a +6.5 dB peak at 3.2 kHz, the preamp gain automatically shifts to -6.5 dBFS.

---

### 4.5 Real-Time DSP Audio Effects Suite
**Description:** Five modular psychoacoustic effects reproducing Boom 3D's sweetening rack. Each effect can be independently toggled and adjusted via a dedicated slider. Realizes UJ-1, UJ-2, UJ-3.

#### FR-16: Ambience (Reverberation & Acoustic Space)
The system shall provide an Ambience effect simulating room reflections and stage depth using a feedback delay network (FDN) / Schroeder-Moorer reverb model with variable wet/dry mix adjustable from 0% to 100%. Realizes UJ-3.
**Consequences (testable):**
- Increasing Ambience adds a smooth, diffuse tail without metallic ringing or phase cancellation in stereo signals.

#### FR-17: Fidelity (Harmonic Exciter)
The system shall provide a Fidelity effect that synthesizes subtle even/odd harmonics and high-shelf air, revitalizing lossy compressed audio streams, adjustable from 0% to 100% excitation depth. Realizes UJ-1.
**Consequences (testable):**
- Applying Fidelity to low-bitrate audio introduces measurable harmonic content above 10 kHz without causing harsh sibilance or aliasing.

#### FR-18: Night Mode (Dynamic Range Compressor)
The system shall provide a Night Mode compressor with fast attack (<5ms), soft knee, and program-dependent release, taming explosive spikes and boosting quiet dialogue, with leveling intensity adjustable from 0% to 100% (compression threshold ranging dynamically from 0 dBFS down to -30 dBFS). Realizes UJ-3.
**Consequences (testable):**
- Loud transient peaks exceeding -12 dBFS are smoothly attenuated by up to 18 dB, while signals below -30 dBFS receive up to +6 dB clean makeup gain.

#### FR-19: Spatial (Stereo Expander & Crossfeed)
The system shall provide a Spatial stereo widening effect based on Mid/Side matrixing with frequency-selective crossfeed to widen the soundfield beyond earcups while preserving mono center integrity, adjustable from 0% to 100% width. Realizes UJ-1, UJ-2.
**Consequences (testable):**
- Mono signals (sum L+R) pass through without comb-filtering or volume drops when Spatial is set to 100%.

#### FR-20: Pitch (Real-Time Pitch Shifter)
The system shall provide a Pitch adjustment slider allowing the user to shift overall audio pitch by ±12 semitones in real-time. `[ASSUMPTION: Pitch Shifter Phase 2]`
**Out of Scope for MVP:** Real-time pitch shifting DSP is slated for immediate Phase 2 delivery after MVP release.

---

### 4.6 Per-Application Audio Session Mixer
**Description:** Dedicated mixer panel providing volume control and mute toggles for every individual running process playing audio on Windows. Realizes UJ-4.

#### FR-21: Process Enumeration & Icon Display
The system shall query Windows `IAudioSessionManager2` to enumerate all active audio sessions, displaying each process's application name, executable icon, and current playback level. Realizes UJ-4.
**Consequences (testable):**
- Active audio sessions (e.g., `chrome.exe`, `discord.exe`, `spotify.exe`) appear in the mixer within 1 second of initiating playback.
- Terminated processes disappear automatically.

#### FR-22: Independent Process Volume Faders
The user can adjust individual process volumes from 0% to 100% (or boost up to 150% via software gain) and click a mute button per process. Realizes UJ-4.
**Consequences (testable):**
- Lowering the Discord slider in OpenBoom directly updates the process volume via Windows Audio Session API without affecting master volume.

#### FR-23: Persistent Application Settings
The system shall remember custom volume offsets and mute states for specific application executables across reboots. Realizes UJ-4.
**Consequences (testable):**
- Relaunching an application restores its previously configured volume slider position.

---

### 4.7 Master Peak Limiter & Clipping Prevention
**Description:** A dedicated lookahead brickwall peak limiter at the end of the DSP processing chain to prevent digital distortion and protect listener hearing. Realizes UJ-1, UJ-2, UJ-3.

#### FR-24: Lookahead Peak Limiter
The system shall implement a peak limiter with 2ms lookahead and transparent release envelope to clamp inter-sample peaks at -0.2 dBFS. Realizes UJ-1, UJ-3.
**Consequences (testable):**
- Even with +12 dB EQ boost, 100% Fidelity, and high master volume, output audio signal strictly never exceeds 0 dBFS (0% digital clipping distortion).

#### FR-25: Real-Time Limiter Gain Reduction Indicator
The UI status bar shall display a dynamic visual indicator showing when and how much gain reduction (in dB) is being applied by the limiter.
**Consequences (testable):**
- When audio peaks trigger limiting, the indicator pulses red/amber showing exact dB attenuation.

---

### 4.8 Desktop UI Shell & System Tray Integration
**Description:** Modern dark Fluent aesthetic matching Boom 3D's visual fidelity, offering fast launch, hardware-accelerated rendering, and low resource idle in the system tray. Realizes UJ-1, UJ-2, UJ-3, UJ-4.

#### FR-26: Dark Fluent Visual Theme
The UI shall implement a cohesive dark palette (`#16171b` surface, `#3b82f6` electric blue accents, `#ec4899` pink/magenta accents, soft slate text) with smooth hover and active state transitions.
**Consequences (testable):**
- All UI elements, sliders, dials, and cards adhere to the high-contrast dark theme without white flashbang glitches on launch or resize.

#### FR-27: Master Power Switch
A prominent glowing master power toggle switch at the top-left shall enable or disable the entire DSP chain in one click.
**Consequences (testable):**
- When switched OFF, the audio pipeline switches to zero-latency passthrough or enters idle standby, dropping DSP CPU consumption to 0.0%.

#### FR-28: Windows System Tray & Background Run
Closing the main window minimizes OpenBoom to the Windows notification tray; double-clicking or single-clicking the tray icon restores the window within <200ms.
**Consequences (testable):**
- Right-clicking the tray icon surfaces a quick-action context menu (Master Power ON/OFF, Preset selector, Mute, Exit).

#### FR-29: Windows Startup & Session Persistence
The user can toggle "Launch on Windows Startup" in settings. Application window coordinates, active presets, volume states, and headphone models persist automatically.
**Consequences (testable):**
- Registry run key or startup shortcut is created cleanly without requiring admin escalation.

---

## 5. Non-Goals (Explicit)

- **Kernel Driver Development (`.sys` driver):** We will NOT develop, sign, or install proprietary kernel-mode audio drivers. All interception is handled via user-mode WASAPI or established virtual audio loopback interfaces to guarantee 100% OS stability and zero BSOD risks.
- **Cloud Accounts, Telemetry, and Licensing Servers:** OpenBoom will NOT require user accounts, email registration, internet connectivity, or background telemetry analytics.
- **DAW / VST Host Architecture (v1):** OpenBoom is not a general-purpose VST plugin host or digital audio workstation in v1. It is a dedicated consumer audio sweetening suite.
- **Active Noise Cancellation (ANC) DSP:** We are not building microphone AI noise suppression in v1 (SteelSeries ClearCast / Krisp scope). OpenBoom focuses exclusively on render/playback audio enhancement.
- **Mobile Platforms (iOS / Android):** OpenBoom v1 is exclusively targeted at Windows 10 (1903+) and Windows 11 (64-bit).

---

## 6. MVP Scope

### 6.1 In Scope (Phase 1 / MVP)
- Modern Dark Fluent UI with master toggle, volume slider, and interactive circular 3D Surround Dial.
- User-mode WASAPI Process Loopback capture and low-latency rendering (<15ms).
- 31-Band Biquad Graphic/Parametric Equalizer with 15 standard presets + custom save/load.
- Integrated offline AutoEq database (4,000+ headphone models) with auto-gain preamp compensation.
- Core DSP Rack: Spatial Stereo Widening, Fidelity Harmonic Exciter, Ambience Reverb, and Night Mode (DRC).
- Master Lookahead Peak Limiter with gain-reduction metering.
- Per-Application Volume Mixer tab (`IAudioSessionManager2`).
- System tray minimization and startup launch.

### 6.2 Out of Scope for MVP (Phase 2 & Beyond)
- Pitch Shifter module (slated for immediate Phase 2 follow-up).
- Custom HRTF `.sofa` measurement file import.
- Windows Audio Processing Object (APO) native integration mode.
- Global configurable hotkeys (e.g., Ctrl+Shift+B for 3D toggle).
- Audio visualizer spectrum analyzer (real-time FFT dancing bars).

---

## 7. Success Metrics

### Primary Metrics
- **SM-1 (Buffer Round-Trip Latency):** Total audio latency between system audio generation and headphone output is **< 15ms** at 48 kHz. Validates FR-2, FR-3, FR-4.
- **SM-2 (Zero Driver Crashes):** 100% user-mode execution with **0 operating system BSODs or kernel panics**. Validates FR-2.
- **SM-3 (Resource Footprint):** Idle RAM consumption remains **< 40 MB**, and active DSP processing CPU usage remains **< 2.0%** on an Intel Core i5 / AMD Ryzen 5 or newer. Validates FR-1, FR-9, FR-16, FR-18.
- **SM-4 (AutoEq Calibration Speed):** Headphone model search and biquad profile application completes in **< 100ms**. Validates FR-13, FR-14, FR-15.

### Secondary Metrics
- **SM-5 (Startup Time):** Application cold start to audio readiness in **< 800ms**. Validates FR-28, FR-29.
- **SM-6 (Zero Digital Clipping):** With limiter active, maximum measured peak output never exceeds **-0.1 dBFS** across all extreme test tracks. Validates FR-24.

### Counter-Metrics (Do Not Optimize)
- **SM-C1 (Do Not Sacrificed Stability for Microsecond Latency):** Do not reduce the internal circular buffer below 128 samples if it causes audio stutter, crackling, or underruns on lower-tier hardware. Audio continuity and glitch-free playback strictly takes precedence over sub-5ms extreme targets.
- **SM-C2 (Do Not Sacrificed Phase Coherence for Width):** Do not artificially expand the 3D surround or Spatial stereo field if it causes severe mono-sum phase cancellation or hollow dialogue.

---

## 8. Cross-Cutting Non-Functional Requirements (NFRs)

- **Performance & Latency Budget:** 
  - Audio processing must execute within real-time priority threads (`AVRT` / `Pro Audio` thread characteristics).
  - Memory allocations (`malloc` / `new`) are strictly prohibited inside the real-time audio callback loop to prevent GC pauses or heap lock contention.
- **Architecture & OS Compatibility:**
  - Native 64-bit Windows support (x64 and ARM64).
  - Primary driverless mode utilizes native Windows 11 and Windows 10 (build 20348+) process loopback APIs; legacy Windows 10 (1903–2004) is supported via optional virtual cable bridge fallback.
- **SIMD Acceleration:**
  - DSP math (biquad filtering, convolution, FFT) must utilize SSE2/AVX vector intrinsics on x64 to minimize battery and CPU impact on laptops.
- **Privacy & Security:**
  - Zero outbound network traffic during audio processing. No telemetry beacons, crash analytics, or tracking cookies.
- **Audio Thread Resilience:**
  - Audio callback processing time must remain < 60% of the buffer duration to prevent audio glitching during heavy desktop loads.

---

## 9. Constraints and Guardrails

- **Zero Kernel Driver Signing Expenses:** OpenBoom must never require expensive EV code signing certificates or Microsoft WHQL submissions. The application must run cleanly out of the box.
- **Open-Source License Compliance:** All integrated third-party algorithms (AutoEq database, biquad calculations, resamplers) must adhere to permissive open-source licenses (MIT, Apache 2.0, BSD-3, or GPLv3).
- **Graceful Failure:** If loopback capture fails due to exclusive-mode audio locks from another application, OpenBoom must display a clear, helpful toast notification rather than silently failing or playing static.

---

## 10. Open Questions

1. **OQ-1 (Audio Interception & Feedback Prevention):** *Resolved.* Primary capture utilizes Windows 10 build 20348+ / Windows 11 `AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` for feedback-free driverless processing. Legacy Windows 10 releases (1903–2004) route via an optional virtual audio cable bridge.
2. **OQ-2 (UI Implementation Framework):** Handed off to architecture planning (`bmad-architecture`) to evaluate C# / Avalonia (.NET 8/9 AOT) vs. Rust / Tauri + React frontend.

---

## 11. Assumptions Index

- `[ASSUMPTION: Process Loopback Primary]` (§4.1 FR-2) — Windows 10 build 20348+ and Windows 11 process loopback is the primary driverless audio capture pipeline.
- `[ASSUMPTION: Generic KEMAR HRTF Default]` (§4.2 FR-8) — Shipped generic KEMAR dummy-head binaural impulse response matrix provides immediate high-fidelity surround simulation for 95% of users; custom `.sofa` profile loading is deferred to Phase 2.
- `[ASSUMPTION: Offline AutoEq Bundling]` (§4.4 FR-13) — Bundling a compressed pre-computed offline database of AutoEq profiles (<5MB) is preferred over dynamic GitHub API fetching at runtime.
- `[ASSUMPTION: Pitch Shifter Phase 2]` (§4.5 FR-20) — Real-time pitch shifting DSP is deferred to Phase 2 while Ambience, Fidelity, Night Mode, and Spatial are delivered in the Phase 1 MVP.
