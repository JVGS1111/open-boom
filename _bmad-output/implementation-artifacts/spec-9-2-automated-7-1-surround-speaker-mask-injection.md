---
title: 'Automated 7.1 Surround Physical Speaker Mask Injection'
type: 'feature'
created: '2026-10-13'
status: 'done'
baseline_revision: '8c65d08b5aec58331523a2652f2e532ba011fdcd'
review_loop_iteration: 0
followup_review_recommended: false
context: []
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** By default, third-party virtual audio endpoints install in 2-channel Stereo mode. For OpenBoom's 3D binaural convolution engine to receive unmixed surround sound, the virtual endpoint must be configured in Windows as 7.1 Surround (`0x63F`). Requiring users to manually open `mmsys.cpl` and navigate the speaker configuration wizard creates friction and configuration errors.

**Approach:** Implement a backend routine in `crates/open-boom-wasapi` to query and inject `PKEY_AudioEndpoint_PhysicalSpeakers = 0x63F` (`{1da5d803-d346-45ab-9f40-8c2f1083a47e},3`) as a DWORD in the virtual endpoint property store and registry, expose the `configure_virtual_endpoint_speaker_mask` Tauri command, and verify 8-channel audio ingestion.

## Boundaries & Constraints

**Always:**
- Use the standard Microsoft 7.1 Surround speaker mask `0x63F` (`KSAUDIO_SPEAKER_7POINT1_SURROUND` = FL | FR | FC | LFE | BL | BR | SL | SR).
- Support synthetic fallback and mock environments so all unit tests pass without requiring real Windows hardware or elevation.
- Sanitize and validate GUIDs and speaker masks.
- Pass all Cargo workspace unit tests and React frontend tests.

**Never:**
- Overwrite physical audio devices' speaker configurations.
- Panic or crash when the registry write fails due to standard user permissions.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Configure 7.1 mask on virtual device | `configure_virtual_endpoint_speaker_mask(None)` | Returns `Ok(0x63F)` and sets property in registry | Returns error if no virtual device found |
| Query speaker mask on 7.1 virtual device | Virtual device has 7.1 mask | Returns `Ok(0x63F)` | Safe default (0x63F) on synthetic fallback |
| Synthetic test environment | `force_synthetic: true` | Returns `Ok(0x63F)` without executing `reg.exe` | Zero external subprocess dependencies |

</intent-contract>

## Code Map

- `crates/open-boom-wasapi/src/device.rs` -- `SPEAKER_MASK_7_1_SURROUND`, `configure_virtual_endpoint_7_1`, `set_windows_endpoint_speaker_mask`, `get_windows_endpoint_speaker_mask`.
- `crates/open-boom-wasapi/tests/device_tests.rs` -- Unit tests for 7.1 mask injection and query.
- `src-tauri/src/lib.rs` -- Tauri command `configure_virtual_endpoint_speaker_mask` exposed and tested in IPC handler.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-wasapi/src/device.rs` -- Add `SPEAKER_MASK_7_1_SURROUND = 0x63F`, `configure_virtual_endpoint_7_1`, and registry speaker mask methods.
- `src-tauri/src/lib.rs` -- Expose `configure_virtual_endpoint_speaker_mask` in Tauri command registry.
- `crates/open-boom-wasapi/tests/device_tests.rs` -- Add unit tests for 7.1 speaker mask formatting and configuration.

**Acceptance Criteria:**
- Given a virtual audio device, when `configure_virtual_endpoint_7_1` is called, then the 7.1 Surround mask `0x63F` is applied and returned.
- Given synthetic mode, when the command is called, then `Ok(0x63F)` is returned with zero side effects.

## Verification

**Commands:**
- `cargo test --workspace` -- expected: All Cargo tests pass.
- `npm --prefix src-ui test` -- expected: All UI tests pass.
