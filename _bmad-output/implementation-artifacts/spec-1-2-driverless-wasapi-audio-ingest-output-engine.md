---
title: 'Story 1.2: Driverless WASAPI Audio Ingest & Output Engine'
type: 'feature'
created: '2026-10-10'
baseline_revision: '32a5f6dce73247b83e510ce09f84d211b1e26b94'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-1-context.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** OpenBoom requires real-time user-mode system audio capture and playback without proprietary kernel drivers, with seamless multi-format resampling to 48 kHz float32 and MMCSS Pro Audio scheduling to achieve sub-15ms round-trip latency.

**Approach:** Implement WASAPI Loopback capture (Mode B process exclusion and Mode A standard loopback fallback), format conversion and cubic Hermite resampling for 44.1k/48k/96k/192k audio streams to internal 48 kHz float32, and an MMCSS-registered low-latency render client in `crates/open-boom-wasapi`.

## Boundaries & Constraints

**Always:**
- Operate 100% in user-mode via WASAPI Loopback (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` or standard loopback) without kernel drivers (`.sys`) or admin elevation (`AD-2`, `NFR8`).
- Convert and resample all captured audio (44.1 kHz, 48 kHz, 96 kHz, 192 kHz; 16/24/32-bit PCM and 32-bit float) to continuous 32-bit floating point PCM normalized to `[-1.0, 1.0]` at 48,000 Hz (`AD-3`, `FR3`).
- Register the real-time audio thread with MMCSS (`AvSetMmThreadCharacteristicsW("Pro Audio")`) and keep end-to-end roundtrip latency < 15ms and callback duration < 60% of buffer duration (`NFR1`, `NFR3`).
- Maintain zero heap allocations and zero blocking synchronization primitives inside the active audio streaming callback loop (`AD-1`).
- Provide cross-chunk sample interpolation history to eliminate audible clicks or discontinuities at buffer boundaries.
- Gracefully handle headless/mock environments in unit tests and development systems where no physical soundcard is active.

**Never:**
- Never require `.sys` kernel driver installation, driver signing, or UAC administrator privileges.
- Never allocate dynamic heap memory (`Vec::new`, `String`, etc.) inside the audio processing callback.
- Never block the audio callback with locks (`Mutex`, `RwLock`) or disk/network I/O.
- Never introduce audio dropouts, DC pops, or pitch drift during sample rate conversion.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 48 kHz Identity Ingest | 48 kHz 32-bit float stereo audio stream | Passthrough at 48 kHz without resampling overhead (`is_identity() == true`) | Zero copy/overhead, exact float samples preserved |
| 44.1 kHz to 48 kHz Resampling | 44.1 kHz stream (e.g. CD audio / Spotify) | Clean upsampled stream at 48 kHz with continuous phase and smooth interpolation | Inter-chunk sample history prevents border clicks |
| 96 kHz / 192 kHz High-Res Downsampling | 96 kHz or 192 kHz high-res PCM stream | Clean downsampled stream at 48 kHz without aliasing bursts or buffer underrun | Polyphase/Hermite interpolation maintains smooth waveform |
| Integer PCM Format Ingest | 16-bit or 24-bit signed integer PCM | Normalized float32 samples in `[-1.0, 1.0]` range | Clamped to `[-1.0, 1.0]` to prevent overflow |
| Audio Device Missing / Headless CI | WASAPI init in headless CI environment without audio device | Falls back to synthetic loopback or returns structured `EngineError::DeviceInitialization` | Does not panic or crash; fails gracefully with typed `EngineError` |
| MMCSS Thread Registration | Audio thread startup on Windows | Calls `AvSetMmThreadCharacteristicsW("Pro Audio")`, obtaining MMCSS handle | Falls back to normal thread priority if MMCSS service unavailable |
| Audio Callback Budget | Buffer duration 10ms (480 samples) | Processing completes in < 6ms (< 60% of buffer period) | Emits budget warning telemetry if exceeded |

</intent-contract>

## Code Map

- `crates/open-boom-wasapi/Cargo.toml` -- Dependencies and feature flags for Windows CoreAudio (`Win32_Media_Audio`, `Win32_System_Threading`, `Win32_System_Com`, `Win32_Foundation`).
- `crates/open-boom-wasapi/src/resampler.rs` -- Cubic Hermite and linear sample rate converter from 44.1/96/192 kHz to 48 kHz float32 with multi-channel support and cross-buffer state continuity.
- `crates/open-boom-wasapi/src/loopback_capture.rs` -- User-mode WASAPI Loopback capture client supporting Mode B (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS`) and Mode A fallback, format conversion, and ingest buffering.
- `crates/open-boom-wasapi/src/render_client.rs` -- Low-latency WASAPI render client with MMCSS `Pro Audio` thread registration and latency telemetry.
- `crates/open-boom-wasapi/src/lib.rs` -- Public exports and module integration for the WASAPI engine boundary.
- `crates/open-boom-core/src/types.rs` -- Reusable constants (`INTERNAL_SAMPLE_RATE`), error types (`EngineError`), and buffer metrics.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-wasapi/Cargo.toml` -- Configure Windows API dependencies and build requirements -- Enables Win32 CoreAudio and MMCSS APIs for loopback capture and rendering.
- `crates/open-boom-wasapi/src/resampler.rs` -- Implement high-fidelity sample rate converter -- Converts 44.1k/88.2k/96k/192k audio streams to 48 kHz float32 with cross-chunk state preservation and zero allocations in processing loop.
- `crates/open-boom-wasapi/src/loopback_capture.rs` -- Implement WASAPI loopback capture adapter -- Ingests system audio driverlessly via Process Loopback Mode B / standard Loopback Mode A, converts PCM formats to normalized f32, and feeds resampler.
- `crates/open-boom-wasapi/src/render_client.rs` -- Implement low-latency render client with MMCSS -- Initializes render stream on target endpoint, registers audio thread under MMCSS "Pro Audio", and tracks callback execution and roundtrip latency.
- `crates/open-boom-wasapi/src/lib.rs` -- Wire up audio engine coordinator and public contracts -- Exposes unified audio ingest/render lifecycle and state controls.
- `crates/open-boom-wasapi/tests/resampler_tests.rs` -- Add unit and integration tests for resampler, loopback format conversion, and latency budget -- Validates 44.1k/48k/96k/192k conversion accuracy, chunk boundary smoothness, and MMCSS fallback.

**Acceptance Criteria:**
- Given OpenBoom is running on Windows 10 (19041+) or Windows 11 without administrative elevation, when audio playback is detected from any application, then `crates/open-boom-wasapi` captures the stream via WASAPI Loopback (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` Mode B or Virtual Bridge Mode A) without installing or invoking `.sys` kernel drivers (`FR2`, `NFR8`, `AD-2`).
- Given an incoming audio stream at 44.1 kHz, 48 kHz, 96 kHz, or 192 kHz, when samples enter the ingestion adapter, then the stream is cleanly resampled to the internal 48,000 Hz 32-bit float (`f32`) format without audible clicks, pitch shifting, or buffer underruns (`FR3`, `AD-3`).
- Given the audio render client is initialized, when the audio thread registers with MMCSS (`AvSetMmThreadCharacteristicsW("Pro Audio")`), then the end-to-end capture-to-render round-trip latency remains < 15ms and callback duration takes < 60% of the buffer period (`NFR1`, `NFR3`).

## Spec Change Log
- 2026-10-10: Implemented cubic Hermite resampler with C1 inter-chunk continuity and zero heap allocations.
- 2026-10-10: Implemented WASAPI loopback capture supporting Mode B process exclusion, Mode A fallback, and format conversion.
- 2026-10-10: Implemented low-latency render client with MMCSS "Pro Audio" registration guard and latency budget telemetry.
- 2026-10-10: Implemented AudioEngineCoordinator wiring capture and render lifecycle.
- 2026-10-10: Added comprehensive test suite in resampler_tests.rs (21 unit and integration tests passing).

## Review Triage Log

## Design Notes

- Resampler uses cubic Hermite interpolation with a 4-point kernel across buffer boundaries to ensure C1 continuity and eliminate high-frequency aliasing and clicks.
- MMCSS registration wraps `AvSetMmThreadCharacteristicsW` in RAII guard `MmcssGuard` that calls `AvRevertMmThreadCharacteristics` on drop.
- Ingest formats supported: IEEE Float 32-bit, PCM 16-bit, PCM 24-bit (packed 3-byte and padded 4-byte), and PCM 32-bit integer, converting to standard `f32` in `[-1.0, 1.0]`.

## Verification

**Commands:**
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" check --workspace` -- expected: All crates compile with zero errors.
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --workspace` -- expected: All workspace tests pass, including resampler fidelity and loopback format conversion tests.
- `npm --prefix src-ui run build` -- expected: Frontend builds with zero errors.
