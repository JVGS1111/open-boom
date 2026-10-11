<p align="center">
  <img src="assets/banner.png" alt="OpenBoom Banner" width="100%" style="border-radius: 12px; box-shadow: 0 8px 32px rgba(0,0,0,0.4);" />
</p>

# OpenBoom

<p align="center">
  <strong>Next-generation, low-latency 3D spatial audio enhancer, parametric equalizer, and real-time DSP studio for Windows.</strong>
</p>

<p align="center">
  <a href="https://github.com/JVGS1111/open-boom/releases"><img src="https://img.shields.io/github/v/release/JVGS1111/open-boom?style=for-the-badge&logo=github&color=38bdf8" alt="Latest Release" /></a>
  <a href="https://github.com/JVGS1111/open-boom/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/JVGS1111/open-boom/release.yml?branch=main&style=for-the-badge&label=Release%20Build" alt="Build Status" /></a>
  <a href="https://github.com/JVGS1111/open-boom/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue?style=for-the-badge" alt="License" /></a>
  <img src="https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-0078d4?style=for-the-badge&logo=windows" alt="Platform" />
  <img src="https://img.shields.io/badge/Rust-1.80%2B-orange?style=for-the-badge&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/Tauri-v2-24c8db?style=for-the-badge&logo=tauri" alt="Tauri" />
</p>

---

## ⚡ Overview

**OpenBoom** is a modern, high-fidelity, open-source alternative to proprietary audio enhancers like Boom 3D. Engineered in **Rust** and **Tauri v2** with a clean-room architectural design, OpenBoom transforms any standard headphones or speakers into an immersive 3D soundstage with zero kernel driver instability, sub-15ms round-trip latency, and zero telemetry.

- **Zero-Crash User-Mode Architecture:** Pure WASAPI & PortCls routing without intrusive third-party kernel drivers.
- **Audiophile Grade DSP:** SSE2/AVX SIMD-accelerated filtering, 64-tap KEMAR HRTF binaural convolution, and an inviolable lookahead brickwall peak limiter.
- **Modern Fluent Dark UI:** High-contrast audiophile aesthetic with neon cyan glow, tactile circular dials, live spectrum meters, and hardware-accelerated animations.

---

## ✨ Features

### 🎧 3D Spatial Soundstage & Binaural Simulation
- **Binaural 7.1/5.1 Downfold:** Converts surround audio streams into realistic spatial audio using KEMAR HRTF tables with Interaural Time Difference (ITD 0.2–0.7ms) and pinna rear notches at 6 kHz.
- **Interactive 300° Soundstage Dial:** Intuitive circular control with polar angle physics and tactile detents.
- **8-Channel Virtual Speaker Configurator (`Ctrl + Shift + S`):** Top-down listener orbital ring view with customizable azimuth angles ($\theta$), individual node mutes, and relative gain faders ($\pm 6\text{ dB}$).

### 🎚️ Dual-Mode Equalizer Studio (`Ctrl + Shift + E`)
- **31-Band ISO 1/3-Octave Graphic EQ:** Covering 20 Hz to 20 kHz with Direct Form II Transposed biquad topology ($Q=4.31$) and Schur-Cohn stability verification.
- **10-Band Quick Graphic EQ:** Instant musical adjustment preserving smooth Bezier curves.
- **12-Band Parametric Pre-EQ Stage:** Low/High Shelves and Peaking filters with automatic pre-amp gain compensation ($A_{\text{preamp}} = -\max(0, \max_i(G_i))$).
- **15 Factory Studio Presets:** Harman Acoustic, Bass Boost, Treble Boost, Gaming, Vocal Clarity, Electronic, Rock, Jazz, Movie, and more.
- **Custom Presets:** Atomic preset persistence (`%APPDATA%\OpenBoom\presets\*.json`).

### 🎯 Embedded AutoEq Headphone Calibration
- **Over 4,200 Headphone Profiles:** Instant offline access to the Jaakkopasanen/AutoEq measurement database compressed to just 209 KB.
- **Sub-Millisecond Fuzzy Search:** Find your exact headphone model (Sennheiser, Sony, Audio-Technica, Beyerdynamic, AirPods, HyperX, etc.) in under 0.1ms.

### 🍬 Psychoacoustic Audio Sweeteners Rack
A dedicated 5-module real-time audio enhancement strip:
- **Ambience:** Schroeder-Moorer / Feedback Delay Network (FDN) algorithmic reverb with 4 comb filters, 2 series all-pass diffusers, and asymmetric stereo decorrelation.
- **Fidelity:** Psychoacoustic harmonic exciter generating second ($x^2$) and third ($x^3$) harmonics with soft tanh saturation and high-frequency air shelf.
- **Night Mode:** Dynamic Range Compressor (DRC) with 2.5ms fast attack, 120ms release, +6 dB dialogue clarity boost, and a -55 dBFS noise-gate.
- **Spatial:** Mid/Side stereo width expander with mono sub-bass anchor (< 200 Hz) and mathematically guaranteed mono compatibility ($L=R \implies S=0$).
- **Pitch:** Harmonic pitch modulation engine preview.

### 🔊 150% Master Volume Boost & Lookahead Peak Limiter
- **Perceptual Volume Curve:** $p^2$ taper (-12 dB at 50%, 0 dB at 100%, +3.5 dB boost at 150%) with snap-to-100% on double click.
- **Master Brickwall Limiter:** 2ms lookahead window (96 frames @ 48 kHz), 0.2ms attack, transparent 50ms release, and an inviolable -0.2 dBFS ceiling with 0% digital clipping.

### 🎛️ Per-Application Audio Session Mixer (`Ctrl + Shift + M`)
- **Live Windows Core Audio Session Manager:** Dedicated COM MTA background worker dynamically enumerating active processes (Spotify, Discord, Chrome, games).
- **Individual Process Faders:** Fine-tune volume levels from 0% to 150% and toggle individual process mutes.
- **Persistent Volume Memory:** Automatically remembers custom volume levels across process and system restarts via `%APPDATA%\OpenBoom\app_volumes.json`.

### ⚡ Low-Latency WASAPI Dual-Mode Pipeline
- **Mode A (Virtual Audio Bridge):** Echo-free sink bridge routing with sub-15ms round-trip latency.
- **Mode B (Loopback Fallback):** Real-time capture with process tree exclusion.
- **MMCSS Pro Audio Scheduling:** Windows Multimedia Class Scheduler Service integration preventing audio buffer dropouts.
- **Phase-Continuous Instant A/B Bypass (`Ctrl + Shift + B`):** 5ms equal-power crossfade ($g_A^2 + g_B^2 = 1.0$) allowing instant click-free comparison between raw and processed audio.

---

## 📥 Download & Installation

Pre-compiled binaries for Windows 10/11 (64-bit) are automatically published with every release:

👉 **[Download the Latest OpenBoom Release](https://github.com/JVGS1111/open-boom/releases)**

| Package | Description |
| :--- | :--- |
| **`OpenBoom_x.x.x_x64-setup.exe`** | **Recommended:** Official NSIS Installer with Start Menu shortcuts, tray auto-start option, and clean uninstaller. |
| **`OpenBoom-vx.x.x-portable-x64.exe`** | **Portable:** Standalone zero-installation executable. Run anywhere without administrator rights. |
| **`SHA256SUMS.txt`** | Cryptographic checksums for verifying binary integrity. |

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>B</kbd> | Toggle Instant A/B Hardware Bypass |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>E</kbd> | Toggle Equalizer Studio Drawer (10-Band / 31-Band / AutoEq) |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>S</kbd> | Open 3D Soundstage Speaker Configurator Modal |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>M</kbd> | Open Per-Application Audio Session Mixer Drawer |
| <kbd>Space</kbd> / <kbd>Enter</kbd> | Toggle Master Power Switch (when focused) |
| <kbd>Double Click</kbd> | Reset Volume / Sliders / Dial to default (0 dB / 50%) |
| <kbd>Shift</kbd> + <kbd>Drag</kbd> | 5x High-Precision Fader Adjustment in Equalizer |
| <kbd>Esc</kbd> | Dismiss any open modal or drawer |

---

## 🛠️ Building from Source

### Prerequisites
- **Windows 10 / 11** (x86_64)
- **Node.js** 20.x or later & `npm`
- **Rust** 1.80+ (MSVC toolchain: `x86_64-pc-windows-msvc`)
- **Visual Studio C++ Build Tools** (with Windows 10/11 SDK)

### Step-by-Step Instructions

1. **Clone the repository:**
   ```bash
   git clone https://github.com/JVGS1111/open-boom.git
   cd open-boom
   ```

2. **Install frontend dependencies:**
   ```bash
   npm install
   ```

3. **Run in development mode (hot-reloading):**
   ```bash
   npm run dev
   ```

4. **Run the automated test suites:**
   ```bash
   # Run all Cargo workspace tests (DSP, WASAPI, Storage)
   cargo test --workspace

   # Run frontend React & component unit tests
   npm run test
   ```

5. **Build the production release:**
   ```bash
   # Build the official NSIS Windows Installer
   npm run build:nsis

   # Build standalone portable executable
   npm run build:portable
   ```
   Artifacts will be located in:
   - NSIS Setup: `target/release/bundle/nsis/OpenBoom_*.exe`
   - Portable Binary: `target/release/open-boom.exe`

---

## 🚀 CI/CD & Automated Releases

OpenBoom features a fully automated **GitHub Actions CI/CD pipeline**:

- **Continuous Integration (`.github/workflows/ci.yml`):** Runs on every push and pull request to `main`, validating Cargo compilation, test suites, and frontend typechecks.
- **Automated Releases (`.github/workflows/release.yml`):** Triggered automatically whenever a semantic version tag (`v*.*.*`) is pushed:
  1. Runs all test suites (`cargo test --workspace`, `npm run test`).
  2. Compiles optimized release binaries with MSVC.
  3. Packages the NSIS installer and standalone portable `.exe`.
  4. Generates `SHA256SUMS.txt` cryptographic digests.
  5. Creates a GitHub Release and attaches all binaries.

### Creating a Release Tag

To publish a new release, you can use the included PowerShell helper:

```powershell
# Create an annotated release tag matching package.json version
npm run release:tag

# Push the commits and tag to GitHub to trigger the release build
git push origin main
git push origin v0.1.0
```

---

## 📐 Architecture & Workspace

OpenBoom is structured as a modular Cargo workspace orchestrated by Tauri v2:

```
open-boom/
├── crates/
│   ├── open-boom-core/       # Pure DSP pipeline: Biquad EQ, Limiter, HRTF Spatializer, Sweeteners Rack
│   ├── open-boom-wasapi/     # Windows Core Audio WASAPI ingest, MMCSS, Virtual Bridge, COM MTA Session Manager
│   └── open-boom-storage/    # Atomic NTFS configuration, AutoEq database, custom presets, startup registry
├── src-tauri/                # Tauri v2 native desktop shell, IPC commands, system tray
├── src-ui/                   # React 18, TypeScript, Tailwind CSS, Lucide icons, Fluent Dark HUD
├── scripts/                  # Release and AutoEq generator automation scripts
└── .bmad/                    # BMAD Memory Harness (Architecture, ADRs, Task Backlog, Milestones)
```

The reverse engineering methodology and audio sink driver analysis are documented under `.bmad/evidence/rea-boom-engine-reverse-engineering-report.md`.

---

## 📄 License

This project is dual-licensed under either:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

---

<p align="center">
  Built with ❤️ for audiophiles, gamers, and music creators.
</p>
