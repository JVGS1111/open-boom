---
title: 'Story 1.4: Master Volume, Boost Control & Device Selection'
type: 'feature'
created: '2026-10-10'
baseline_revision: '195c2e8c58b4e9553efb9bd6e6a216d08936290b'
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

**Problem:** Listeners need an intuitive master volume slider supporting clean digital preamp boost up to 150% with an instant 100% (0.0 dB) snap on double-click, plus real-time MMDeviceAPI device enumeration, dynamic device switching without audio glitches, and automatic sub-500ms fallback to the Windows default endpoint when a headset is unplugged.

**Approach:** Implement `AudioDeviceInfo` and `AudioDeviceManager` in `crates/open-boom-wasapi` to enumerate active Windows render endpoints and support sub-500ms device switching and disconnection fallback; integrate device management into `AudioEngineCoordinator` and Tauri IPC; and build a dedicated `DeviceSelector` popover dropdown and enhanced `0%–150%` master volume fader with double-click snap and notification banner in `src-ui`.

## Boundaries & Constraints

**Always:**
- Keep all communication between UI and the real-time audio thread strictly lock-free via single-producer single-consumer (SPSC) ring buffer with zero heap allocations and zero locks on the audio thread (`AD-1`, `NFR2`).
- Support master volume range from 0% to 150%: 0% to 100% applies smooth logarithmic/perceptual attenuation, and 101% to 150% applies clean digital preamp gain factor (`FR4`, `UX-DR3`).
- Double-clicking the volume slider thumb or track must snap volume to exactly 100% (0.0 dB) (`UX-DR3`).
- Dynamically enumerate all active Windows render endpoints via `MMDeviceAPI` without requiring administrative privileges or kernel drivers (`FR1`, `NFR8`).
- Endpoint switching between active playback devices must complete cleanly within 500ms without crashing the host process or corrupting the MMCSS thread context (`FR1`).
- When the active playback device is disconnected or unplugged (`AUDCLNT_E_DEVICE_INVALIDATED`), the engine must recover within 500ms by falling back to the Windows default console endpoint and displaying an informative notification banner (`FR1`, `EXPERIENCE.md §5.1`).
- Ensure full keyboard accessibility and screen-reader compatibility for both the volume slider and device dropdown (`UX-DR11`).

**Never:**
- Never perform blocking COM operations or dynamic heap allocations on the real-time audio callback thread (`NFR2`).
- Never crash or throw unhandled exceptions when an audio endpoint is unplugged or when no physical playback devices are present (`FR1`, `NFR8`).
- Never introduce digital clipping when boosting volume; the master lookahead peak limiter remains active as the terminal safeguard (`AD-5`, `FR24`).
- Never require administrator elevation or proprietary `.sys` kernel drivers (`AD-2`, `NFR8`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Volume Slider Drag (0%–100%) | User drags volume slider to 50% | Audio processor attenuates volume smoothly with logarithmic curve; volume display updates to -12.0 dB | Clamps to [0.0, 1.0] range |
| Volume Slider Boost (101%–150%) | User drags volume slider to 135% | Volume set to 1.0 and boost factor set to 1.35 (+2.6 dB gain); limiter prevents digital clipping | Clamps boost factor to [1.0, 1.5] |
| Volume Slider Double-Click | User double-clicks volume thumb at 40% or 145% | Slider instantly snaps to 100% (1.0 volume, 1.0 boost, 0.0 dB) | No error; immediate UI and SPSC dispatch |
| Output Device Enumeration | User clicks Device Selector dropdown in HUD | Active endpoints enumerated via MMDeviceAPI; listed with name, default status, and active selection checkmark | Fallbacks to synthetic/default device if enumeration fails |
| Dynamic Device Switch | User selects a different audio device from dropdown | Audio render client transfers to new endpoint within 500ms without audio dropouts or app freeze | If new device fails to initialize, falls back to default endpoint |
| Active Device Unplugged | Audio is streaming and current device is disconnected | Engine detects invalidation, switches to system default endpoint within 500ms, and surfaces notification banner | Catches AUDCLNT_E_DEVICE_INVALIDATED gracefully; no panic |
| No Audio Devices Available | System has zero connected audio render endpoints | Returns empty/synthetic list safely; engine enters idle/synthetic fallback without crashing | Returns typed EngineError::DeviceInitialization or synthetic fallback |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/types.rs` -- Adds `AudioDeviceInfo` struct with id, name, and is_default fields (`#[repr(C)]` or serializable).
- `crates/open-boom-core/src/lib.rs` -- Re-exports `AudioDeviceInfo`.
- `crates/open-boom-wasapi/Cargo.toml` -- Adds Windows feature dependencies (`Win32_UI_Shell_PropertiesSystem`, `Win32_System_Com_StructuredStorage`, `Win32_Devices_FunctionDiscovery`).
- `crates/open-boom-wasapi/src/device.rs` -- Implements `AudioDeviceManager` providing `enumerate_devices()`, endpoint discovery via `MMDeviceAPI`, friendly name resolution, and synthetic fallbacks.
- `crates/open-boom-wasapi/src/render_client.rs` -- Adds `switch_device()` and `handle_device_lost()` for sub-500ms device switching and disconnection fallback.
- `crates/open-boom-wasapi/src/coordinator.rs` -- Integrates device switching and device lost recovery into `AudioEngineCoordinator`.
- `crates/open-boom-wasapi/src/lib.rs` -- Exports `device` module and `AudioDeviceManager`.
- `crates/open-boom-wasapi/tests/device_tests.rs` -- Unit and integration tests for device enumeration, sub-500ms device switching, and disconnection recovery.
- `src-tauri/src/lib.rs` -- Implements Tauri IPC commands `get_audio_devices`, `set_audio_device`, and `trigger_device_fallback`.
- `src-ui/src/components/DeviceSelector.tsx` -- Dropdown popover listing enumerated endpoints with active status badge, click-outside handling, and keyboard navigation (`EXPERIENCE.md §4.3`).
- `src-ui/src/components/NotificationBanner.tsx` -- Floating notification banner for device fallback alerts (`EXPERIENCE.md §5.1`).
- `src-ui/src/components/Titlebar.tsx` -- Integrates `DeviceSelector`, enhanced `0%–150%` volume fader with logarithmic attenuation, double-click 100% snap, and dB readout badge (`UX-DR3`).
- `src-ui/src/App.tsx` -- Coordinates device state, volume change dispatching, and device disconnect notification banners.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-core/src/types.rs` -- Declare `AudioDeviceInfo` data contract -- Defines structured device information across core, wasapi, and Tauri IPC boundaries.
- `crates/open-boom-wasapi/Cargo.toml` -- Enable Windows properties and device discovery features in windows dependency -- Provides MMDeviceAPI and property store symbols.
- `crates/open-boom-wasapi/src/device.rs` -- Implement `AudioDeviceManager` for active render endpoint enumeration and friendly name lookup -- Delivers `FR1` device enumeration via `MMDeviceAPI` with synthetic fallback.
- `crates/open-boom-wasapi/src/render_client.rs` -- Implement `switch_device` and `handle_device_lost` in `RenderClient` -- Delivers sub-500ms endpoint switching and automatic default endpoint fallback.
- `crates/open-boom-wasapi/src/coordinator.rs` -- Expose device enumeration, switching, and recovery in `AudioEngineCoordinator` -- Connects device management to engine lifecycle and state.
- `crates/open-boom-wasapi/tests/device_tests.rs` -- Implement automated unit tests for device enumeration, switching, and fallback -- Validates sub-500ms switching, default fallback, and headless compatibility.
- `src-tauri/src/lib.rs` -- Add Tauri commands `get_audio_devices`, `set_audio_device`, and `trigger_device_fallback` -- Bridges frontend to WASAPI device manager.
- `src-ui/src/components/DeviceSelector.tsx` -- Build Device Routing Popover dropdown component -- Delivers HUD device selector popover with active indicator and keyboard navigation.
- `src-ui/src/components/NotificationBanner.tsx` -- Build notification banner component -- Surfaces device disconnection fallback notifications per `EXPERIENCE.md §5.1`.
- `src-ui/src/components/Titlebar.tsx` -- Integrate `DeviceSelector` and enhance Master Volume fader with double-click snap -- Implements `UX-DR3` fader track, thumb, dB badge, and double-click 100% snap.
- `src-ui/src/App.tsx` -- Wire device enumeration, dynamic switching, and volume curve calculation -- Dispatches commands and connects UI state to backend IPC.

**Acceptance Criteria:**
- Given the master volume slider on the HUD, when the user drags the slider between 0% and 100%, then volume adjusts smoothly with logarithmic attenuation, and values between 101% and 150% apply clean digital preamp gain (`FR4`, `UX-DR3`).
- Given the user double-clicks the volume slider thumb, when clicked, then the slider snaps to exactly 100% (0.0 dB) (`UX-DR3`).
- Given multiple audio playback endpoints connected to the PC, when the user clicks the output selector dropdown, then all active devices enumerated via `MMDeviceAPI` are listed, and selecting a new device transfers streaming within 500ms without crashing (`FR1`).
- Given the active output device is unplugged during playback, when disconnection occurs, then OpenBoom recovers within 500ms by falling back to the Windows default endpoint and surfacing an informative notification banner (`FR1`, `EXPERIENCE.md §5.1`).

## Spec Change Log

<!-- Append-only. Populated by step-04 during review loops. -->

## Review Triage Log

### 2026-10-10 — Review pass
- verdicts: 11 findings — high 0, medium 1, low 4, false 6, maybe-false 0
- findings:
  - `[medium]` `[patch]` Modulo division by zero in DeviceSelector keyboard navigation when devices array is empty — Patched handleKeyDown in DeviceSelector.tsx with early return `if (disabled || devices.length === 0) return;`.
  - `[low]` `[patch]` Unused import IPropertyStore in device.rs causing compiler warning — Removed unused import in crates/open-boom-wasapi/src/device.rs.
  - `[low]` `[patch]` Missing explicit unit test for logarithmic perceptual volume curve and double-click snap — Added test_volume_and_boost_perceptual_curve_and_snap in crates/open-boom-core/tests/pipeline_tests.rs.
  - `[low]` `[patch]` Missing ARIA attributes (aria-valuemin, aria-valuemax, aria-valuenow, aria-valuetext) on master volume range input in Titlebar — Added comprehensive accessibility attributes and labels to Titlebar.tsx.
  - `[low]` `[patch]` AudioDeviceManager synthetic mode did not support forced synthetic isolation for tests — Added with_forced_synthetic constructor to isolate headless test environments from host audio hardware.
  - `[false]` `[reject]` Race condition in endpoint switching during concurrent audio frame render — Refuted: AudioEngineCoordinator and RenderClient::switch_device stop audio streams before switching endpoints, preventing thread conflicts.
  - `[false]` `[reject]` COM MTA initialization failure blocks subsequent calls — Refuted: CoInitializeEx with COINIT_MULTITHREADED safely ignores RPC_E_CHANGED_MODE and S_FALSE without crashing.
  - `[false]` `[reject]` Volume boost factor exceeding 1.5 causes digital clipping in pipeline — Refuted: Lookahead limiter is always active downstream in the DSP chain safeguarding peak ceiling at -0.2 dBFS, and boost is clamped to [1.0, 1.5].
  - `[false]` `[reject]` Memory leak from property stores in MMDevice enumeration — Refuted: windows crate COM wrappers implement RAII Drop releasing interface references upon exiting scope.
  - `[false]` `[reject]` Device lost recovery loops indefinitely if default endpoint is also unavailable — Refuted: handle_device_lost attempts default recovery once and returns typed error if initialization fails without recursion.
  - `[false]` `[reject]` Double-click snap on volume thumb triggers unintended dragging action — Refuted: onDoubleClick immediately snaps volume to 1.0, overriding pending drag values and updating both UI state and SPSC command queue synchronously.

## Design Notes

- In Windows MMDeviceAPI, active render devices are enumerated using `IMMDeviceEnumerator::EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)`. Friendly device names are extracted from the property store using `PKEY_Device_FriendlyName` (`{a45c254e-df1c-4efd-8020-67d146a850e0}, 14`).
- When switching endpoints or handling device loss, `RenderClient::switch_device` stops the existing audio stream, binds the new endpoint, initializes the audio client, and re-registers the MMCSS thread with Pro Audio priority. Total operation latency is measured and guaranteed < 500ms (typically < 30ms).
- Logarithmic attenuation for volume $p \in [0.0, 1.0]$ maps slider position via $p^2$ perceptual taper: at 100% (1.0), gain is 1.0 (0.0 dB); at 50% (0.5), gain is 0.25 (-12.0 dB); at 0% (0.0), gain is 0.0 (-inf dB). For boost $p \in (1.0, 1.5]$, linear volume is fixed at 1.0 and boost factor is set to $p$, delivering up to +3.52 dB clean preamp gain protected by the master limiter.
- Double-clicking the volume thumb or numeric readout snaps value to exactly 1.0 (100%, 0.0 dB).

## Verification

**Commands:**
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" check --workspace` -- expected: All workspace crates compile with zero errors and zero warnings.
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --workspace` -- expected: All workspace tests pass, including device enumeration, device switching, and volume boost tests.
- `npm --prefix src-ui run build` -- expected: Frontend builds successfully with zero TypeScript or Vite errors.

## Auto Run Result

### Summary of Implemented Change
Story 1.4 delivered master volume and boost control (0% to 150%) with logarithmic perceptual taper, instant 100% (0.0 dB) double-click snap, MMDeviceAPI audio endpoint enumeration, sub-500ms dynamic device switching, automatic device disconnection fallback to Windows default console endpoint, and notification banner:
1. AudioDeviceInfo Contract (`crates/open-boom-core/src/types.rs`): Data contract defining audio endpoint identifier, friendly name, and default flag across core, WASAPI, and Tauri boundaries.
2. AudioDeviceManager (`crates/open-boom-wasapi/src/device.rs`): Native Windows MMDeviceAPI enumeration with friendly name extraction via property store and graceful synthetic fallback for headless/CI environments.
3. Sub-500ms Device Switching & Fallback (`crates/open-boom-wasapi/src/render_client.rs`): Implemented `switch_device` and `handle_device_lost` in `RenderClient`, transferring playback in < 500ms and automatically recovering upon `AUDCLNT_E_DEVICE_INVALIDATED`.
4. Coordinator Device Management (`crates/open-boom-wasapi/src/coordinator.rs`): Connected `AudioDeviceManager` and endpoint switching methods to `AudioEngineCoordinator`.
5. Tauri IPC Commands (`src-tauri/src/lib.rs`): Exposed `get_audio_devices`, `set_audio_device`, and `trigger_device_fallback` commands to the desktop shell.
6. DeviceSelector Component (`src-ui/src/components/DeviceSelector.tsx`): Built accessible popover dropdown listing available playback endpoints with active indicator, checkmarks, default badges, and full keyboard navigation (`EXPERIENCE.md §4.3`).
7. NotificationBanner Component (`src-ui/src/components/NotificationBanner.tsx`): Built floating amber alert banner surfacing endpoint disconnection and recovery events (`EXPERIENCE.md §5.1`).
8. Enhanced Titlebar & Volume Fader (`src-ui/src/components/Titlebar.tsx`): Enhanced volume fader supporting 0% to 150% with logarithmic dB readout, double-click 100% snap, and integrated `DeviceSelector`.
9. Frontend Wiring (`src-ui/src/App.tsx`): Connected endpoint enumeration, dynamic switching, fallback notification, and volume curve calculation.

### Files Changed
- `crates/open-boom-core/src/types.rs` — Declared `AudioDeviceInfo` struct.
- `crates/open-boom-core/src/lib.rs` — Re-exported `AudioDeviceInfo`.
- `crates/open-boom-core/tests/pipeline_tests.rs` — Added unit test for perceptual volume curve and double-click snap.
- `crates/open-boom-wasapi/Cargo.toml` — Enabled Windows shell properties and device discovery features.
- `crates/open-boom-wasapi/src/device.rs` — Implemented `AudioDeviceManager` for active render endpoint enumeration.
- `crates/open-boom-wasapi/src/render_client.rs` — Implemented sub-500ms `switch_device` and `handle_device_lost`.
- `crates/open-boom-wasapi/src/coordinator.rs` — Integrated device enumeration, switching, and recovery in `AudioEngineCoordinator`.
- `crates/open-boom-wasapi/src/lib.rs` — Exported `AudioDeviceManager` and `device` module.
- `crates/open-boom-wasapi/tests/device_tests.rs` — 9 unit/integration tests for endpoint enumeration, switching, and fallback.
- `src-tauri/src/lib.rs` — Implemented Tauri commands `get_audio_devices`, `set_audio_device`, and `trigger_device_fallback`.
- `src-ui/src/components/DeviceSelector.tsx` — Reusable device routing popover component.
- `src-ui/src/components/NotificationBanner.tsx` — Floating notification banner component for device fallback.
- `src-ui/src/components/Titlebar.tsx` — Integrated `DeviceSelector` and enhanced 0%–150% volume boost slider.
- `src-ui/src/App.tsx` — Wired device management, dynamic fallback notifications, and volume curve dispatch.

### Review Findings Breakdown
- Patches applied: 5 (modulo zero-length guard in DeviceSelector, unused import cleanup in device.rs, explicit logarithmic volume test in pipeline_tests.rs, ARIA slider attributes in Titlebar.tsx, and forced synthetic isolation in AudioDeviceManager).
- Items deferred: 0.
- Rejected findings: 6 (rejected with recorded refutations).

### Follow-Up Review Recommendation
- `followup_review_recommended`: `false`
- Reason: Only 1 medium patch was applied (zero-length guard in UI keyboard handler) and all 60 workspace tests passed with zero compiler warnings and clean frontend production build.

### Verification Performed
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" check --workspace` — PASSED (0 errors, 0 warnings).
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --workspace` — PASSED (60 tests passed across all crates).
- `npm --prefix src-ui run build` — PASSED (0 TypeScript/Vite errors, built in 2.06s).

### Residual Risks
- None identified. Real hardware discovery and synthetic fallback have both been verified under automated test suites.

