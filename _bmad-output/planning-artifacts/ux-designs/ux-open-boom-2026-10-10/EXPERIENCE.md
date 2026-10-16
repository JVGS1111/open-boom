---
name: OpenBoom
status: final
sources:
  - _bmad-output/planning-artifacts/prds/prd-open-boom-2026-10-10/prd.md
  - _bmad-output/planning-artifacts/prds/prd-open-boom-2026-10-10/addendum.md
  - .bmad/00-PROJECT-BRIEF.md
  - .bmad/01-ARCHITECTURE.md
  - .bmad/02-MILESTONES.md
created: 2026-10-10
updated: 2026-10-10
---

# OpenBoom — Experience & Interaction Spine (EXPERIENCE.md)

This specification defines the information architecture, behavioral component contracts, interaction primitives, state models, accessibility baselines, and key user flows for **OpenBoom**. It governs how the application behaves, complementing [DESIGN.md](file:///c:/Users/joaov/Documents/GitHub/open-boom/_bmad-output/planning-artifacts/ux-designs/ux-open-boom-2026-10-10/DESIGN.md) (*how it looks*).

* **Interactive Key-Screen Reference:** [open-boom-shell.html](mockups/open-boom-shell.html) (promoted 1:1 tactile reference mockup).
* **Precedence:** Both `DESIGN.md` and `EXPERIENCE.md` win on conflict with any mock, wireframe, or imported asset.

---

## 1. Foundation

* **Form-Factor:** Windows Native Desktop Utility (Windows 11 and Windows 10 build 20348+). Single-window shell with expandable drawers, popovers, and dedicated modal sheets.
* **UI Platform / Implementation System:** Windows Native XAML (WinUI 3 / WPF) with Mica/Acrylic backdrop material compatibility. Follows modern Windows desktop ergonomics and native DPI scaling (100% to 250%).
* **Design System Reference:** All visual tokens (colors, typography, radii, spacing, elevation) are formally declared in [DESIGN.md](file:///c:/Users/joaov/Documents/GitHub/open-boom/_bmad-output/planning-artifacts/ux-designs/ux-open-boom-2026-10-10/DESIGN.md) and referenced throughout this document using `{path.to.token}` syntax.
* **Execution Boundary:** 100% local, zero-telemetry, offline-first application. No login gates, account creation, or internet dependency.

---

## 2. Information Architecture

### Surfaces Map

| Surface | Reached From | Purpose | Dismissal / Transition |
|---|---|---|---|
| **Main Rack (Shell)** | Application launch / Taskbar / Tray click | The central command console: Master Switch, Output Endpoint selector, Global Volume/Boost, 3D Surround Dial, 5-Effect Strip, Equalizer preview dock, and Status Bar. | Minimize to system tray (`close` button minimizes by default; customizable). |
| **3D Speaker Configurator** | Click speaker calibration icon on the 3D Surround Dial | Full 8-channel virtual speaker soundstage editor (FL, C, FR, SL, SR, RL, RR, Sub). Allows individual speaker muting, angular re-positioning, and relative gain trim (±6 dB). | Modal overlay; dismissed by `Esc`, clicking outside, or clicking `Done`. |
| **Equalizer Studio Drawer** | Click `Expand EQ` or the EQ Curve graph in the dock | Full-height frequency workstation. Toggles between 10-Band (Basic) and 31-Band (Precision) faders, real-time FFT spectrum overlay, preset management, and custom curve saving. | Inline accordion drawer or detached side panel; collapses via toggle. |
| **AutoEq Headphone Browser** | Click Headphone Profile chip in EQ dock | Searchable picker covering 4,000+ headphone models (HyperX, Sennheiser, Sony, etc.). Displays target curve vs compensated response with 1-click apply. | Dropdown popover with search input; auto-dismisses on selection or `Esc`. |
| **Application Stream Mixer** | Click `App Volumes` icon in Window Header | Per-application audio mixer enumerating all active Windows audio sessions (`IAudioSessionManager2`). Controls volume and mute per process (e.g. Spotify, Discord, Game). | Slide-out right drawer or popover; dismisses via toggle or `Esc`. |
| **Device Routing Popover** | Click Output Device name in Header | Lists all active WASAPI playback endpoints (`MMDeviceAPI`) with latency and sample rate badges. Instant 1-click endpoint switching. | Popover menu; auto-closes upon device selection or clicking outside. |
| **Settings & Hotkeys Modal** | Click Gear icon in Window Header | Configure startup behavior (launch with Windows, start minimized), global keyboard shortcuts, buffer size/latency preferences, and diagnostics export. | Modal dialog; dismissed via `Close` button or `Esc`. |

### Screen Hierarchy & Anatomy

```
+----------------------------------------------------------------------------------------------------+
| OPENBOOM  [Master Switch: ON]  [Output: Realtek Audio v]  [Volume Fader: 78%]  [Apps] [Settings] [-][x]|
+----------------------------------------------------------------------------------------------------+
|                                                                                                    |
|                                       [ 3D SURROUND DIAL ]                                         |
|                                     Concentric Knob (192px)                                        |
|                                    Intensity Readout: "65%"                                        |
|                                [Calibrate 7.1 Virtual Speakers]                                    |
|                                                                                                    |
+----------------------------------------------------------------------------------------------------+
|  EFFECTS RACK STRIP (5 Cards):                                                                     |
|  [ Ambience: 20% ]  [ Fidelity: 65% ]  [ Night Mode: ON ]  [ Spatial: 40% ]  [ Pitch: 0 st ]      |
+----------------------------------------------------------------------------------------------------+
|  EQUALIZER & AUTOEQ DOCK:                                                                          |
|  Profile: [ HyperX Cloud II (AutoEq) v ]    Preset: [ Rock / Electronic v ]     Mode: [ 10 | 31 ]   |
|  [ Real-Time FFT Spectrum Visualizer & 31-Band Interactive Curve Graph ]              [Expand EQ]  |
+----------------------------------------------------------------------------------------------------+
|  STATUS: 48 kHz | 32-bit Float | Latency: 9.2 ms | Limiter: -0.2 dB FS | DSP: Active   [A/B Bypass]|
+----------------------------------------------------------------------------------------------------+
```

---

## 3. Voice and Tone

OpenBoom speaks like professional, high-end audio hardware: concise, unembellished, unambiguous, and immediate.

* **Purity and Precision:** Never use exaggerated hype or promotional slogans like *"Magical sound"*, *"Revolutionary spatial audio"*, or *"Supercharge your PC"*. Use technical audio facts: *"Binaural 7.1 HRTF Convolution"*, *"Dynamic Range Compression"*, *"Harmonic Exciter"*.
* **Status Readouts:**
  * Engine Active: `DSP Streaming (48 kHz / 32-bit Float)`
  * Bypass Active: `Hardware Passthrough (Bypassed)`
  * Limiter Protection: `Limiter Active (-1.4 dB Gain Reduction)`
  * Device Swapped: `Switched to 'SteelSeries Arctis Nova 7' (10.4 ms)`
* **Destructive Confirmations:**
  * Resetting an EQ curve: *"Reset 'Custom Rock' to flat (0.0 dB across all bands)? This action cannot be undone."*
  * Discarding unsaved changes: *"You have unsaved changes in preset 'Acoustic Warmth'. Discard changes or save as new preset?"*
* **Microcopy Brevity:** Tooltips must not exceed one short sentence:
  * 3D Surround: *"Modulates binaural HRTF expansion and interaural time delay."*
  * Night Mode: *"Compresses dynamic peaks and enhances speech clarity for quiet listening."*
  * Fidelity: *"Generates subtle harmonic overtones to restore compressed high frequencies."*

---

## 4. Component Patterns

### 4.1 Master Power Switch & A/B Bypass
* **Master Switch (`{components.master-switch}`):** Located at the top-left of the header. Master on/off toggle for the entire DSP processing pipeline. Turning it off stops the loopback processing loop and returns audio to direct bit-perfect passthrough.
* **Instant A/B Bypass Button:** Located in the bottom-right status bar. When toggled, audio flips between processed DSP and unprocessed dry audio within 1 audio buffer (<10ms). Crucial for audio verification without losing current knob states.

### 4.2 Global Volume & Boost Slider (`{components.volume-slider}`)
* **Range:** `0%` to `150%` (0%–100% standard Windows master volume; >100% applies clean software preamp gain with lookahead limiter safety).
* **Feedback:** Hover displays exact dB gain (`-∞ dB` to `+6.0 dB`). Double-clicking the slider thumb immediately snaps volume to `100%` (`0.0 dB`).
* **Mute Toggle:** Clicking the volume speaker icon mutes/unmutes without moving the slider thumb.

### 4.3 3D Surround Rotary Dial (`{components.surround-dial-3d}`)
* **Behavior:** The visual focal point of OpenBoom. Dragging the outer rim or knob rotates the active angular arc from 0° (0% intensity) to 300° (100% intensity).
* **Sensory Feedback:** As the knob passes 25%, 50%, and 75%, subtle visual detent pips illuminate along the arc.
* **Center Display:** Displays real-time numeric percentage (`0%` to `100%`). Double-clicking the center resets the dial to default `50%`.

### 4.4 5-Effect Quick Rack Strip (`{components.effect-card}`)
* **Rack Structure:** Horizontal sequence of 5 standard cards:
  1. **Ambience:** Early reflections and stereo reverb depth (`0%` to `100%`).
  2. **Fidelity:** Odd/even harmonic exciter intensity (`0%` to `100%`).
  3. **Night Mode:** Fast-attack dynamic range compression and dialogue boost (`0%` to `100%`).
  4. **Spatial:** Mid/side stereo widening and crossfeed (`0%` to `100%`).
  5. **Pitch:** Real-time pitch transposition in semitones (`-12 st` to `+12 st`, default `0`).
* **Interaction:** Clicking the card header or icon toggles power on/off. Dragging the mini-slider or scrolling adjusts intensity. Clicking the card gear icon reveals advanced DSP tweak parameters (e.g. Reverb room size, Compressor threshold).

### 4.5 Equalizer Canvas & Fader Bank (`{components.eq-slider}`)
* **Modes:** Segmented switch toggles between **10-Band** (standard casual tuning) and **31-Band** (1/3-octave studio precision).
* **Fader Controls:** Each frequency band features a vertical fader slider spanning `-12.0 dB` to `+12.0 dB` with a mechanical detent at `0.0 dB`.
* **Zero Snap:** Double-clicking any fader cap instantly resets that band to `0.0 dB`.
* **Fine Adjustment:** Holding `Shift` while dragging reduces fader speed by 5×, allowing exact `0.1 dB` precision.
* **Curve Canvas:** Above the faders, a spline curve dynamically draws the cumulative frequency response curve.

### 4.6 AutoEq Headphone Picker
* **Search Field:** Autocomplete input with instant fuzzy matching against 4,000+ headphone models.
* **Profile Card:** Shows headphone manufacturer, model name, measurement source (e.g., Oratory1990, Crinacle), and compensatory parametric filter count.
* **Stacking Behavior:** AutoEq correction curve is applied *prior* to user equalizer adjustments, ensuring user EQ boosts or cuts modify a neutral target rather than a flawed headphone response.

### 4.7 8-Channel Virtual Speaker Configurator
* **Top-Down Polar Soundstage:** Displays a central listener head surrounded by 8 virtual speaker nodes: Front Left, Center, Front Right, Surround Left, Surround Right, Rear Left, Rear Right, and Subwoofer.
* **Channel Solo/Mute:** Clicking a speaker node toggles it active/muted. Muting a speaker renders it grayed out with a red strike indicator.
* **Angular Repositioning:** Dragging a speaker node slides it along its orbital radial track, updating the HRTF azimuth coordinate in real time.

### 4.8 Application Volume Mixer (`{components.app-mixer-row}`)
* **Process Discovery:** Automatically monitors Windows audio sessions. Apps actively producing sound display an animated green audio indicator.
* **Volume Slider:** Independent `0%` to `100%` fader per application.
* **Mute Button:** Mutes the target application without altering the Windows global mixer.

---

## 5. State Patterns

### 5.1 Master Audio Engine States

```
[ Uninitialized / Starting ]
         |
         v (WASAPI Loopback Initialized)
[ DSP Streaming (Active) ] <====================> [ Bypassed (Dry Passthrough) ]
         |                                                 ^
         | (Audio Buffer > 0 dB FS)                        | (Click A/B Bypass)
         v                                                 |
[ Limiting (Soft-Clipped) ] -------------------------------+
         |
         | (Active Output Device Unplugged)
         v
[ Device Lost / Fallback ] ---> [ Reconnected / Auto-Recovered ]
```

* **Starting:** Engine allocates WASAPI loopback capture buffer and circular FIFO queue. UI displays connecting spinner on status strip.
* **Active:** Normal operating state. Audio is captured, processed through HRTF, EQ, 5 effects, and limiter, and output to playback endpoint. Latency reported in status strip.
* **Bypassed:** DSP processing bypassed. Audio flows directly from source to output bit-perfect without tonal coloration.
* **Limiting:** Input or accumulated gain exceeds `0.0 dB FS`. Lookahead peak limiter attenuates output to prevent digital clipping. Limiter meter lights up amber/red.
* **Device Lost:** Headphone disconnected. Engine catches `AUDCLNT_E_DEVICE_INVALIDATED`, falls back to system default endpoint within 500ms, and posts a discreet notification banner.

### 5.2 Preset Lifecycle States
* **Clean Factory Preset:** User has selected a stock preset (e.g., `Rock`). Preset name displayed in `{colors.text-primary}`.
* **Modified Preset (`*`):** User adjusts any EQ band or effect while on a preset. Name updates to `Rock *` with an adjacent `Save As` button.
* **Custom User Preset:** Custom curve saved locally to `%APPDATA%/OpenBoom/presets.json`. Can be renamed, exported, or deleted.

---

## 6. Interaction Primitives

### 6.1 Rotary Dial Dragging Physics
* **Mouse Drag:** Clicking anywhere on the circular arc and dragging tracks the cursor's polar angle relative to the knob center:
  $$\theta = \text{atan2}(y - y_0, x - x_0)$$
* **Mouse Wheel:** Hovering over the dial and scrolling steps intensity up or down by `2%` per notch (`5%` when holding `Shift`).
* **Direct Jump:** Clicking directly on a point along the outer arc snaps the knob instantly to that angular value.

### 6.2 EQ Fader Dynamics
* **Vertical Travel:** 180px vertical stroke corresponding to 24.0 dB total range (7.5 px/dB).
* **Tactile Center Detent:** Approaching within ±0.3 dB of `0.0 dB` applies a soft magnetic snap to `0.0 dB`.
* **Floating Value Badge:** Dragging displays a high-contrast Cascadia Code tooltip showing current value (`+2.4 dB`). Dismisses 300ms after release.

### 6.3 Global Keyboard Shortcuts `[ASSUMPTION: Default Hotkeys]`
* **`Ctrl + Shift + B`:** Toggle Master DSP Bypass (A/B Test).
* **`Ctrl + Shift + M`:** Master Mute / Unmute.
* **`Ctrl + Shift + Up`:** Increase Master Volume by 5%.
* **`Ctrl + Shift + Down`:** Decrease Master Volume by 5%.
* **`Ctrl + Shift + E`:** Open / Close Equalizer Studio Drawer.
* **`Ctrl + Shift + S`:** Open / Close 3D Speaker Configurator.

---

## 7. Accessibility Floor

* **High Contrast & Windows Contrast Themes:** All borders, active indicator rings, and slider tracks respect Windows Contrast Themes (`Desert`, `Dusk`, `Night Sky`, `Aquatic`). When high-contrast mode is detected, neon glows are replaced with solid 2px outline focus borders.
* **Keyboard Focus Traversal:** Strict logical tab order:
  1. Window Header controls (Master Switch → Output Selector → Volume Fader → App Mixer → Settings).
  2. 3D Surround Dial (Arrow Up/Right increases, Arrow Down/Left decreases, `Home` = 0%, `End` = 100%).
  3. Effect Rack Cards (Tab to card, `Space` toggles power, Arrow keys adjust slider).
  4. Equalizer Dock (Tab through frequency bands, Up/Down adjusts by 0.5 dB, `PageUp`/`PageDown` by 3 dB).
* **Windows UI Automation (UIA) Integration:**
  * All sliders expose `RangeValuePattern` with accurate current values, minimums, maximums, and unit strings (`"decibels"`, `"percent"`).
  * Switches expose `TogglePattern` with clear `"On"` and `"Off"` states.
  * Live status changes (Bypass active, device switch) fire `UIA_LiveRegionChangedEventId` for screen readers (NVDA, JAWS, Narrator).

---

## 8. Key Flows

### Flow 1: Marcus — Spatial Audio Listener Calibrating Positional Soundstage
1. **Context:** Marcus starts a multi-channel spatial audio simulation. Spatial orientation and directional cues are critical to his listening experience.
2. **Setup:** Marcus opens OpenBoom from the system tray. The main HUD displays on his secondary monitor.
3. **Spatial Tuning:** Marcus grabs the 3D Surround Dial and dials it to `75%`. The soundstage widens instantly over his stereo headset.
4. **Speaker Fine-Tuning:** Marcus clicks the calibration icon to open the **8-Channel Virtual Speaker Configurator**. He notices ambient cues in the rear left sound slightly subdued; he selects the Rear Left speaker node and bumps its gain to `+3.0 dB`.
5. **Fidelity Boost:** On the Effects Rack, Marcus clicks the **Fidelity** tile and sets it to `50%`, enhancing high-frequency transient detail for acoustic cues.
6. **Climax Beat:** Marcus listens to an immersive multichannel sequence. With the customized rear azimuth HRTF and high-frequency exciter active, Marcus identifies the exact angle and distance of surround effects instantly. He presses `Ctrl+Shift+B` to test with and without OpenBoom, marveling at how flat and directional-less raw stereo sounds in comparison.

### Flow 2: Elena — Audiophile Calibrating Headphone Curves with AutoEq
1. **Context:** Elena connects her Sennheiser HD 600 headphones to her PC to listen to a new lossless orchestral release.
2. **AutoEq Search:** In the Equalizer dock, Elena clicks the Headphone Profile picker. She types `"HD 600"` in the search box.
3. **Profile Application:** OpenBoom presents `Sennheiser HD 600 (Oratory1990 - Harman Target)`. Elena clicks **Apply Profile**. The dock updates, applying a 10-filter parametric compensation curve that flattens sub-bass roll-off.
4. **Equalizer Customization:** Wanting slightly warmer cello timbre, Elena clicks **Expand EQ** to reveal the 31-Band studio fader bank.
5. **Fader Adjustment:** She holds `Shift` and drags the `125 Hz` fader up by `+1.8 dB` and the `250 Hz` fader by `+1.2 dB`. The interactive spline curve animates smoothly.
6. **Climax Beat:** As the symphony reaches its crescendo, Elena saves the preset as `"HD600 Classical Warmth"`. She toggles **A/B Bypass** in the footer; the sound transforms from clinical neutrality to breathtaking concert hall richness with zero distortion and 0.0 dB limiter clipping.

### Flow 3: David — Late-Night Movie Watcher with Dynamic Compression
1. **Context:** It is 11:30 PM. David's family is asleep, and he is watching an action blockbuster with explosive sound effects and whisper-quiet dialogue.
2. **Problem:** Constantly having to raise the volume to hear whispers and quickly lower it during explosions is frustrating.
3. **Activation:** David brings up OpenBoom and clicks the **Night Mode** card on the Effects Rack, setting intensity to `80%`.
4. **Processing:** The dynamic range compressor engages with a fast attack time and soft knee curve, taming high-energy explosions down by -8 dB while boosting low-level dialogue tracks by +6 dB.
5. **Spatial Expansion:** David enables **Ambience** at `25%` and **Spatial** at `45%`, creating an expansive theatrical atmosphere inside his headphones.
6. **Climax Beat:** A sudden stealth explosion rocks the movie screen. The limiter and Night Mode compressor catch the transient instantly, preventing room-shaking loudness or headphone distortion, while the whispered dialogue immediately following remains crystal clear. David leans back and enjoys the film without touching the volume slider once.

---

## 9. Assumptions & Boundaries `[ASSUMPTION]`

1. `[ASSUMPTION: Default HUD Dimensions]`: The base HUD window defaults to `840px × 560px` with a minimum clamp of `780px × 520px` to ensure compatibility with 1080p and 720p displays under 125% OS scaling.
2. `[ASSUMPTION: WinUI 3 Native Styling]`: Visual styling will be implemented using WinUI 3 XAML resources and Acrylic/Mica brushes, fallback to standard dark solids on older Windows 10 builds.
3. `[ASSUMPTION: AutoEq Database Cache]`: The AutoEq index is pre-compiled into a local JSON bundle (~1.8 MB compressed) stored locally, enabling instant offline searching with <5ms search query latency.
4. `[ASSUMPTION: Hotkey Scheme]`: Global hotkeys use `Ctrl + Shift` modifiers to prevent collisions with in-game controls and Windows system shortcuts.
