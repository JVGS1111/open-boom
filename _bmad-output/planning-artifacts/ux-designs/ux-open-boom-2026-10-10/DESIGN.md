---
name: OpenBoom
description: Lightweight, driverless 3D spatial audio enhancer, multiband equalizer, and DSP rack for Windows.
colors:
  # Base surfaces (Pro Audio hardware dark theme)
  background: '#121316'
  surface-base: '#16171b'
  surface-card: '#1c1e24'
  surface-card-hover: '#22252c'
  surface-elevated: '#282b34'
  surface-overlay: '#0d0e11'
  border-subtle: '#262933'
  border-default: '#2e323e'
  border-focus: '#38bdf8'
  # Accents & Signal Indicators
  accent-cyan: '#38bdf8'
  accent-cyan-hover: '#0ea5e9'
  accent-cyan-glow: 'rgba(56, 189, 248, 0.35)'
  accent-blue: '#3b82f6'
  accent-magenta: '#ec4899'
  accent-magenta-glow: 'rgba(236, 72, 153, 0.30)'
  # Text & Glyphs
  text-primary: '#f8fafc'
  text-secondary: '#94a3b8'
  text-muted: '#64748b'
  text-inverted: '#0f172a'
  # Status & DSP Meters
  status-online: '#22c55e'
  status-bypassed: '#64748b'
  status-clipping: '#ef4444'
  status-warning: '#f59e0b'
  meter-green: '#10b981'
  meter-yellow: '#facc15'
  meter-red: '#f43f5e'
typography:
  display:
    fontFamily: 'Segoe UI Variable Display, -apple-system, sans-serif'
    fontSize: '28px'
    fontWeight: '600'
    lineHeight: '1.2'
    letterSpacing: '-0.02em'
  title:
    fontFamily: 'Segoe UI Variable Display, -apple-system, sans-serif'
    fontSize: '18px'
    fontWeight: '600'
    lineHeight: '1.25'
    letterSpacing: '-0.01em'
  body:
    fontFamily: 'Segoe UI Variable Text, -apple-system, sans-serif'
    fontSize: '13px'
    fontWeight: '400'
    lineHeight: '1.4'
  body-bold:
    fontFamily: 'Segoe UI Variable Text, -apple-system, sans-serif'
    fontSize: '13px'
    fontWeight: '600'
    lineHeight: '1.4'
  label:
    fontFamily: 'Segoe UI Variable Text, -apple-system, sans-serif'
    fontSize: '11px'
    fontWeight: '500'
    lineHeight: '1.3'
    letterSpacing: '0.04em'
  metric-mono:
    fontFamily: 'Cascadia Code, Consolas, monospace'
    fontSize: '12px'
    fontWeight: '500'
    lineHeight: '1.0'
    letterSpacing: '0.01em'
rounded:
  none: '0px'
  sm: '4px'
  md: '8px'
  lg: '12px'
  xl: '16px'
  full: '9999px'
spacing:
  1: '4px'
  2: '8px'
  3: '12px'
  4: '16px'
  5: '20px'
  6: '24px'
  8: '32px'
  10: '40px'
  gutter: '16px'
  window-padding: '20px'
components:
  master-switch:
    width: '48px'
    height: '26px'
    borderRadius: '{rounded.full}'
    background-active: '{colors.accent-cyan}'
    background-inactive: '{colors.border-default}'
    glow: '{colors.accent-cyan-glow}'
  volume-slider:
    track-height: '6px'
    track-background: '{colors.border-subtle}'
    track-fill: '{colors.accent-cyan}'
    thumb-size: '16px'
    thumb-border: '2px solid {colors.background}'
    thumb-background: '{colors.text-primary}'
  surround-dial-3d:
    size: '192px'
    track-width: '10px'
    arc-background: '{colors.surface-card}'
    arc-active: '{colors.accent-cyan}'
    knob-size: '144px'
    knob-surface: '{colors.surface-elevated}'
    speaker-icon-size: '18px'
  effect-card:
    padding: '{spacing.3}'
    background: '{colors.surface-card}'
    background-active: '{colors.surface-card-hover}'
    border: '1px solid {colors.border-subtle}'
    border-active: '1px solid {colors.accent-cyan}'
    borderRadius: '{rounded.md}'
  eq-slider:
    width: '28px'
    track-width: '4px'
    track-background: '{colors.border-subtle}'
    track-fill: '{colors.accent-blue}'
    thumb-width: '16px'
    thumb-height: '10px'
    thumb-radius: '{rounded.sm}'
    thumb-color: '{colors.text-primary}'
  app-mixer-row:
    height: '44px'
    background: '{colors.surface-card}'
    borderRadius: '{rounded.md}'
    border: '1px solid {colors.border-subtle}'
status: final
created: 2026-10-10
updated: 2026-10-10
---

# OpenBoom — Visual Design System (DESIGN.md)

This specification defines the visual identity, token taxonomy, and aesthetic rules for **OpenBoom** in accordance with the [Google Labs DESIGN.md specification](https://github.com/google-labs-code/design.md). It serves as the single visual source of truth for the desktop shell across Windows native XAML implementations (WinUI 3 / WPF).

* **Interactive Key-Screen Reference:** [open-boom-shell.html](mockups/open-boom-shell.html) (promoted 1:1 tactile reference mockup).
* **Precedence:** Both `DESIGN.md` and `EXPERIENCE.md` win on conflict with any mock, wireframe, or imported asset.

---

## 1. Brand & Style

OpenBoom projects the precision, confidence, and tactile gratification of high-end studio audio rack equipment, distilled into an ultra-responsive modern Windows application.

* **Aesthetic Stance:** Precision Studio Utility meets Modern Fluent Dark. The interface rejects generic web SaaS flatness in favor of subtle tactile depth: machined rotary indicators, physical fader wells, illuminated LED-style states, and purposeful neon telemetry glows.
* **Surface Philosophy:** Deep matte obsidian (`{colors.background}`) acts as the acoustic canvas. Interactive components float slightly above the canvas with dark graphite borders (`{colors.border-subtle}`) and dark card fills (`{colors.surface-card}`), ensuring zero eye fatigue during extended gaming sessions or late-night music listening.
* **Tone:** Functional, authoritative, and clean. No gimmicky skeuomorphic wood panels; every shadow, gradient, and glow communicates an active audio state or physical degree of processing.

---

## 2. Colors

The color palette is deliberately calibrated for dark environments, high legibility, and unmistakable state signaling.

### Base Surfaces
* **`{colors.background}` (`#121316`):** The master canvas and Windows window frame. Used for the root background and negative space between major panels.
* **`{colors.surface-base}` (`#16171b`):** Base structural container for toolbars, dock panels, and drawers.
* **`{colors.surface-card}` (`#1c1e24`):** Primary interactive container. Houses the 5 DSP effect tiles, the Equalizer fader bank, and application mixer rows.
* **`{colors.surface-card-hover}` (`#22252c`):** Hover state for cards and list items, providing immediate visual affordance.
* **`{colors.surface-elevated}` (`#282b34`):** The center core of the 3D Surround rotary knob and modal dialog surfaces.
* **`{colors.surface-overlay}` (`#0d0e11`):** Modal backdrops and dimming layers for focused configuration sheets.

### Structural Borders
* **`{colors.border-subtle}` (`#262933`):** Subdued boundaries separating cards, slider gutters, and toolbar segments.
* **`{colors.border-default}` (`#2e323e`):** Inactive controls, switches in the off-position, and unfocused text input frames.
* **`{colors.border-focus}` (`#38bdf8`):** Keyboard focus rings and active drag highlights.

### Signal Accents
* **`{colors.accent-cyan}` (`#38bdf8`):** The primary soul of OpenBoom. Used for active DSP power, master volume fills, the 3D Surround circular arc, and toggled effects.
* **`{colors.accent-cyan-glow}` (`rgba(56, 189, 248, 0.35)`): Radial soft glow cast around active switches and high-intensity knob indicators.
* **`{colors.accent-blue}` (`#3b82f6`):** Secondary modulation accent. Used for Equalizer frequency curve fills, band faders, and AutoEq profile badges.
* **`{colors.accent-magenta}` (`#ec4899`):** Special feature highlights, stereo spatialization expander indicator, and dialogue boost badges.

### Text & Metrics
* **`{colors.text-primary}` (`#f8fafc`):** Maximum contrast foreground for titles, active values, and primary labels.
* **`{colors.text-secondary}` (`#94a3b8`):** Subtitles, inactive band labels, and device dropdown selections.
* **`{colors.text-muted}` (`#64748b`):** Units (Hz, dB, ms), disabled items, and footer telemetry keys.

### Telemetry & Meters
* **`{colors.status-online}` (`#22c55e`):** Engine streaming active indicator.
* **`{colors.status-bypassed}` (`#64748b`):** Hardware/DSP bypass state.
* **`{colors.meter-green}` (`#10b981`), `{colors.meter-yellow}` (`#facc15`), `{colors.meter-red}` (`#f43f5e`):** Fast VU meter peak and limiter gain-reduction levels.

---

## 3. Typography

OpenBoom uses the Windows native variable font system (`Segoe UI Variable`) paired with a high-legibility monospace face (`Cascadia Code`) for audio measurements.

| Role | Font Family | Size | Weight | Line Height | Tracking | Usage |
|---|---|---|---|---|---|---|
| **Display** | `Segoe UI Variable Display` | `28px` | `600` (SemiBold) | `1.2` | `-0.02em` | Master volume readout, 3D intensity percentage |
| **Title** | `Segoe UI Variable Display` | `18px` | `600` (SemiBold) | `1.25` | `-0.01em` | Modal titles, Equalizer preset header, Rack titles |
| **Body** | `Segoe UI Variable Text` | `13px` | `400` (Regular) | `1.4` | `normal` | General descriptions, dropdown items, dialog text |
| **Body-Bold** | `Segoe UI Variable Text` | `13px` | `600` (SemiBold) | `1.4` | `normal` | Effect card titles, active tab headers |
| **Label** | `Segoe UI Variable Text` | `11px` | `500` (Medium) | `1.3` | `+0.04em` | Slider frequency marks (32Hz, 1kHz), channel tags (FL, FR) |
| **Metric-Mono** | `Cascadia Code` | `12px` | `500` (Medium) | `1.0` | `+0.01em` | Numeric dB readouts (`+3.5 dB`), latency (`9.2 ms`), sample rate |

---

## 4. Layout & Spacing

The layout is built upon a strict **4px baseline grid** (`{spacing.1}` = 4px).

```
+-------------------------------------------------------------------------+  ^
|  WINDOW HEADER: Master Switch | Output Picker | Volume Bar | App Mixer  |  | 56px
+-------------------------------------------------------------------------+  v
|                                                                         |
|                       [ 3D SURROUND DIAL ]                             |
|                           (192px dia)                                   |
|                      Intensity Readout: 65%                             |
|                                                                         |
+-------------------------------------------------------------------------+
|  EFFECTS RACK STRIP (5 Cards: Ambience | Fidelity | Night | Spatial | Pitch)  | 76px
+-------------------------------------------------------------------------+
|  EQUALIZER DOCK / AUTOEQ (Curve Graph / 10-31 Band Faders / Presets)    | Auto
+-------------------------------------------------------------------------+
|  FOOTER STATUS BAR (Sample Rate | Latency | Limiter Headroom | Bypass)  | 32px
+-------------------------------------------------------------------------+
```

### Spacing Scale
* **`{spacing.1}` (4px):** Micro gaps between meter segments, indicator dots, and fader tick lines.
* **`{spacing.2}` (8px):** Padding inside buttons, gap between slider thumb and label, chip padding.
* **`{spacing.3}` (12px):** Internal padding for effect cards, dropdown menus, and list rows.
* **`{spacing.4}` (16px):** Standard gutter between layout columns and card groups.
* **`{spacing.6}` (24px):** Section vertical separation.
* **`{spacing.8}` (32px):** Margin around the central 3D dial.
* **`{spacing.window-padding}` (20px):** Outer edge margin of the primary application window.

### Window Geometry & Responsiveness
* **Minimum Bounds (Compact HUD):** `780px` wide × `520px` high. Everything is fully operational without horizontal scrolling.
* **Default Desktop Workstation:** `920px` wide × `680px` high. Provides full height for the 31-band EQ fader array and live FFT spectrum background.
* **Expanded Ultrawide / Multitasking:** Flex-grid allows the Equalizer and Per-App Mixer to display side-by-side rather than stacked.

---

## 5. Elevation & Depth

OpenBoom avoids heavy diffuse drop shadows in favor of crisp dark borders, inner well shadows, and selective luminance glows.

* **Level 0 (Base Canvas):** No elevation. Hex `{colors.background}`.
* **Level 1 (Cards & Channels):** Border `1px solid {colors.border-subtle}`, surface `{colors.surface-card}`.
* **Level 2 (Active Knobs & Floating Tooltips):** Border `1px solid {colors.border-default}`, surface `{colors.surface-elevated}`, shadow: `0 4px 16px rgba(0, 0, 0, 0.45)`.
* **Fader Well (Recessed Depth):** Inner shadow `inset 0 1px 3px rgba(0, 0, 0, 0.60)`, track background `{colors.border-subtle}`.
* **Glow Layer (Signal Energized):** Active power toggles emit `box-shadow: 0 0 12px {colors.accent-cyan-glow}`.

---

## 6. Shapes

* **`{rounded.none}` (0px):** Real-time FFT spectrum bars and meter dividers.
* **`{rounded.sm}` (4px):** Slider thumbs, EQ fader caps, small numeric badges, and tooltip chips.
* **`{rounded.md}` (8px):** Effect cards, dropdown menus, text inputs, and dialog action buttons.
* **`{rounded.lg}` (12px):** Major panel groupings and modal dialogue wrappers.
* **`{rounded.full}` (9999px):** Master power switch pill, circular dials, speaker node icons, and status badges.

---

## 7. Components

### 7.1 Master Power Switch (`master-switch`)
* **Anatomy:** Pill-shaped toggle with a sliding circular thumb.
* **Active State:** Track background `{colors.accent-cyan}`, thumb `{colors.text-primary}`, glow `0 0 14px {colors.accent-cyan-glow}`.
* **Inactive State:** Track `{colors.border-default}`, thumb `{colors.text-muted}`.
* **Transition:** Smooth 180ms ease-out spring sliding.

### 7.2 3D Surround Dial (`surround-dial-3d`)
* **Anatomy:** Outer concentric progress ring (`192px`), inner matte rotary cap (`144px`), center intensity readout (`{typography.display}`), and 8 circular speaker satellite dots surrounding the rim.
* **Visual States:**
  * Inactive (0% / Bypass): Outer track `{colors.surface-card}`, readout in `{colors.text-muted}`.
  * Active (1% - 100%): Arc swept in `{colors.accent-cyan}` gradient terminating in a luminous beacon dot.
  * Hover: Subtle brightening of rotary cap to `{colors.surface-card-hover}`.

### 7.3 DSP Effect Card (`effect-card`)
* **Anatomy:** Vertical card containing top icon glyph, effect name (`{typography.body-bold}`), power toggle micro-dot, and bottom intensity mini-slider / percentage.
* **Active:** Border `{colors.accent-cyan}`, icon illuminated in `{colors.accent-cyan}`, slider fill active.
* **Bypassed:** Border `{colors.border-subtle}`, icon `{colors.text-muted}`, slider dimmed to `{colors.border-default}`.

### 7.4 EQ Frequency Fader (`eq-slider`)
* **Anatomy:** Vertical slot track (`4px` wide), center detent notch at 0 dB, horizontal fader cap (`16×10px` with 3 tactile grip ridges), top frequency label, bottom dB badge.
* **Detent Indicator:** 0 dB center point features a distinct 2px white indicator line.
* **Dragging:** Fader cap border turns `{colors.accent-cyan}`, floating tooltip badge shows instantaneous value (`+4.2 dB`).

### 7.5 Application Mixer Row (`app-mixer-row`)
* **Anatomy:** App process icon (`24×24px`), process name, mute toggle button, horizontal volume slider (`0%–100%`), and live RMS audio presence meter dot.

---

## 8. Do's and Don'ts

### Do's
* **DO** maintain strict high-contrast legibility for all audio units and frequency markings against the dark background.
* **DO** use cyan (`{colors.accent-cyan}`) exclusively for master audio flow and spatial processing; reserve magenta (`{colors.accent-magenta}`) for secondary stereo widening and vocal features.
* **DO** use Cascadia Code monospace for any numeric data that updates continuously (dB, ms, Hz, %) to eliminate text jitter.
* **DO** provide immediate visual feedback (hover, drag cursor, illuminated ring) on all tactile knobs and sliders.

### Don'ts
* **DON'T** use generic bright white backgrounds or light mode themes; OpenBoom is strictly a dark pro-audio console.
* **DON'T** apply multi-colored rainbow gradients to the Equalizer curves; stick to clean monochrome blue-cyan gradients.
* **DON'T** hide critical audio states (Bypass, Master Mute, Limiter Clipping) behind submenus or tabs.
* **DON'T** add slow, heavy animations (>250ms) to audio controls; all slider interactions must track the pointer with 0ms perceived lag.
