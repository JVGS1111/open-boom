# Epic 1 Context: Desktop Shell & Driverless Audio Foundation

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

The user can launch OpenBoom in a sleek Dark Fluent desktop HUD, capture and stream Windows audio driverlessly with sub-15ms latency, adjust master volume (with clean 150% boost), toggle the master power switch, perform instant click-free A/B bypass comparisons, hear output protected by the master lookahead peak limiter (-0.2 dBFS), and minimize the app to the Windows system tray.

## Stories

- Story 1.1: Project Workspace Scaffolding & Desktop Shell Window
- Story 1.2: Driverless WASAPI Audio Ingest & Output Engine
- Story 1.3: Lock-Free POD Command Dispatcher & Master Power Control
- Story 1.4: Master Volume, Boost Control & Device Selection
- Story 1.5: Master Lookahead Peak Limiter & Telemetry Metering
- Story 1.6: Phase-Continuous Instant A/B Bypass
- Story 1.7: Application Preferences, Window Geometry & System Startup Persistence

## Requirements & Constraints

- Driverless Loopback Audio Capture: User-mode audio capture via WASAPI Loopback (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` with process tree exclusion on Windows 10 build 19041+ and Windows 11; optional virtual bridge fallback). Operating completely without proprietary kernel drivers (`.sys`) or administrative privileges to eliminate BSOD crash risks.
- Audio Device Management: Continuous enumeration and monitoring of active Windows render endpoints via `MMDeviceAPI`. Hot-plug connect and disconnect events handled within 500ms with automatic failover to the Windows default endpoint without hanging or crashing the audio thread.
- Format Ingestion & Resampling: Transparent ingestion of audio streams across arbitrary sample rates (44.1 kHz, 48 kHz, 96 kHz, 192 kHz) and channel configurations (Stereo 2.0 to 7.1), resampled cleanly to a fixed internal 48 kHz 32-bit float format without clicks, pitch drift, or buffer underruns.
- Master Volume & Software Boost: Linear/logarithmic master volume attenuation from 0% to 100%, with clean software preamp gain boost up to 150% (+6.0 dB).
- Lookahead Peak Limiter: Mandatory terminal brickwall limiter with 2ms lookahead delay and transparent release envelope clamping inter-sample peaks at -0.2 dBFS, guaranteeing 0% digital clipping distortion and maximum output <= -0.1 dBFS.
- Limiter Gain Reduction Telemetry: Status bar indicator that dynamically pulses amber/red displaying real-time dB gain reduction applied by the limiter.
- Master Power Control: Illuminated master toggle switch that activates DSP processing or drops the pipeline into zero-latency bit-perfect passthrough with 0.0% DSP CPU usage.
- Phase-Continuous A/B Bypass: Instant sub-10ms bypass toggling between processed DSP and uncolored direct audio without DC pops, clicks, or volume surges.
- System Tray & Desktop Lifecycle: Closing the main window minimizes to the Windows notification tray; clicking or double-clicking the tray icon restores the window within <200ms; right-click tray context menu provides toggles for Master Power, Presets, Mute, and Exit.
- State & Startup Persistence: Atomic persistence of volume, active output device, window coordinates, and optional Windows startup launch without admin elevation.
- Latency & Priority Budgets: Total round-trip latency between audio capture and output must stay strictly < 15ms at 48 kHz. Real-time audio callback thread must register under MMCSS (`AvSetMmThreadCharacteristicsW("Pro Audio")`) with callback processing time strictly < 60% of buffer duration.
- Real-Time Thread Safety: Zero heap allocations (`malloc`, `new`, vector resizing) and zero blocking synchronization primitives (`Mutex`, condition variables) or blocking I/O on the real-time audio thread.
- Resource Budgets: Idle RAM consumption < 40 MB; active DSP CPU usage < 2.0% on Intel Core i5 / AMD Ryzen 5 or equivalent; cold start to audio readiness < 800ms.
- Privacy & Local Operation: 100% local execution with zero outbound network calls, crash telemetry, or user tracking.

## Technical Decisions

- Architecture & Workspace Partitioning: Greenfield Cargo workspace partitioned into decoupled crates and a Tauri v2 shell:
  - `crates/open-boom-core`: Real-time DSP engine coordinator, SPSC ring buffers, and pure mathematical filters (exportable as C-ABI shared library).
  - `crates/open-boom-wasapi`: Windows audio boundary managing WASAPI Process Loopback capture, MMCSS render client, and MMDeviceAPI endpoint monitoring.
  - `crates/open-boom-storage`: Atomic JSON persistence in `%APPDATA%\OpenBoom\`.
  - `src-tauri` & `src-ui`: Lightweight desktop shell (Rust 1.81.0, Node 24, React 18, TypeScript, Tailwind CSS) utilizing Windows WebView2.
- Lock-Free POD Boundary: Communication from UI to the audio thread uses a Single-Producer Single-Consumer (SPSC) lock-free ring buffer transferring flat `#[repr(C)]` Plain Old Data command structs (`AudioCommand` enum). Tauri async handlers emit into a dedicated UI Dispatcher channel that feeds the ring buffer. Deserialization and memory allocation on the audio thread are strictly prohibited. Telemetry (limiter reduction, latency) flows back via atomic primitives.
- Slapback Echo Prevention: Dual-mode routing to prevent double-playback:
  - Mode B (Primary): WASAPI Process Loopback (`PROCESS_LOOPBACK_MODE_EXCLUDE_PROCESS_TREE`) attenuates the captured application session via `ISimpleAudioVolume` while capturing the unattenuated stream.
  - Mode A (Fallback): Ingress from a virtual loopback bridge endpoint.
- Unified 48 kHz Float32 Pipeline Contract: All internal DSP stages process continuous 32-bit floating point PCM (`f32`) normalized to `[-1.0, +1.0]` at fixed 48,000 Hz. Sample rate conversion occurs strictly at capture ingress. Ingress normalizes to 8 channels (7.1) and subsequent stages operate on stereo.
- Limiter & Phase-Continuous Bypass Pipeline: The Master Peak Limiter is the mandatory terminal DSP stage with a fixed 2ms lookahead delay buffer. A/B Bypass routes uncolored audio around filter stages while retaining the 2ms lookahead delay and limiter, using a 5ms equal-power crossfade to guarantee seamless phase continuity without DC pops, clicks, or timing drift.
- Atomic Persistence Contract: Presets, window state, and settings write to temporary `.tmp` files before atomic renaming (`MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`) in `%APPDATA%\OpenBoom\config.json`.

## UX & Interaction Patterns

- Visual System & Tokens: Dark Studio Fluent theme with obsidian canvas (`#121316`), elevated dark surface (`#16171b`), subtle borders (`#262933` / `#2e323e`), electric cyan accent (`#38bdf8`), and neon glow indicators (`rgba(56, 189, 248, 0.35)`). Zero white flashbang glitches during window initialization or resize.
- Window Geometry: HUD dimensions default to `840px × 560px` (clamped min `780px × 520px`) with frameless custom title bar and minimize-to-tray on close.
- Master Power Control: Prominent pill switch (`48px × 26px`) with cyan LED glow toggling the entire DSP pipeline.
- Master Volume & Boost Slider: Horizontal fader track (6px height) supporting 0% to 150% volume. Hover displays exact dB value; double-click snaps to 100% (0.0 dB); speaker icon toggles mute.
- Device Routing & Hot-Plug UI: Header dropdown enumerating active WASAPI endpoints with latency badges. Disconnections trigger automatic fallback within 500ms and surface an informative notification banner.
- Status Bar & Real-Time Telemetry: Bottom dock displaying engine status (`48 kHz`, `32-bit Float`, real-time round-trip latency badge, Limiter headroom meter with amber/red gain reduction pulse), and 1-click A/B Bypass button.
- Accessibility & Shortcuts: Logical Tab focus traversal; full Windows Contrast Themes support (replacing glows with solid 2px outline focus borders); UIA `RangeValuePattern` and `TogglePattern` bindings; global hotkeys (`Ctrl + Shift + B` for Bypass, `Ctrl + Shift + M` for Mute, `Ctrl + Shift + Up/Down` for Volume).

## Cross-Story Dependencies

- Story 1.1 establishes the workspace layout, Tauri v2 bridge, and styling tokens required by all UI and audio stories.
- Story 1.2 creates the WASAPI capture/render loop, resampler, and MMCSS thread setup foundational to Stories 1.3, 1.4, 1.5, and 1.6.
- Story 1.3 implements the SPSC lock-free command queue, enabling Stories 1.4, 1.5, and 1.6 to send runtime parameter changes from UI to audio without locking.
- Story 1.4 routes volume, boost, and device endpoints through the command dispatcher and WASAPI engine.
- Story 1.5 builds the 2ms lookahead delay buffer and peak limiter stage, which Story 1.6 depends upon to maintain phase alignment during A/B bypass crossfades.
- Story 1.6 links bypass state transitions with the limiter lookahead buffer and UI status dock.
- Story 1.7 persists settings (volume, window coordinates, device choice) introduced across Stories 1.1–1.4 into atomic storage.
