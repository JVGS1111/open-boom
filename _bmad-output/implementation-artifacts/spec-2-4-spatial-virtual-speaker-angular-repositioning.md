---
title: 'Story 2.4: Spatial Virtual Speaker Angular Repositioning'
type: 'feature'
created: '2026-10-10'
baseline_revision: '5738a43c536fd3840040d47e66abe842482b754b'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-2-context.md'
  - '_bmad-output/implementation-artifacts/spec-2-3-8-channel-virtual-speaker-configurator-soundstage-modal.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** Standard virtual surround sound engines use rigid, fixed speaker layout angles that fail to match the listener's individual head shape, ear physiology, and personal soundstage preference, leading to inaccurate spatial localization (especially for rear surround cues).

**Approach:** Implement smooth orbital angular repositioning for all virtual speaker nodes in the 8-channel polar soundstage modal (`SoundstageConfiguratorModal.tsx`). Dragging nodes along orbital tracks dynamically updates azimuth angles in real time with continuous HUD coordinate readouts. On the audio engine side (`Spatializer3D`), dynamically interpolate 64-tap KEMAR HRTF impulse response tables across neighboring acoustic anchors without audio dropouts or zipper noise. Provide a single-click "Reset Speaker Positions" action restoring standard ITU-R BS.775 7.1 reference azimuth angles.

## Boundaries & Constraints

**Always:**
- When dragging a speaker node along its radial orbit ring, the node slides smoothly and updates its real-time azimuth coordinate angle in the HUD display (`FR7`, `UX-DR5`).
- When an updated azimuth angle reaches the audio engine via lock-free SPSC commands, the HRTF interpolation tables dynamically adjust binaural convolution filter coefficients for that channel without dropouts (`AD-1`, `AD-4`).
- When the user clicks "Reset Speaker Positions", all 8 speaker nodes return to standard ITU-R BS.775 7.1 reference azimuth angles (`EXPERIENCE.md §4.7`).
- Keep audio thread execution zero-allocation and lock-free (`NFR2`).
- Distinguish pure click (mute toggle) from drag movement (angular repositioning) so that dragging a speaker node never accidentally mutes it.
- Persist custom speaker azimuth angles in `AppConfig` across application restarts (`AD-10`).

**Never:**
- Never block the real-time audio thread or cause buffer underruns during rapid angular dragging.
- Never produce clicks, pops, or DC offsets when interpolating HRTF coefficients across azimuth angles.
- Never lose azimuth coordinates when closing and reopening the modal.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Drag Speaker Node | Pointer down + drag on speaker node along orbit | Node follows pointer angle; live azimuth badge renders `θ: ±X°`; `SetSpeakerAzimuth` dispatched | Constrain LFE to fixed omni; wrap angle to [-180°, +180°] |
| Distinguish Drag from Click | Click with < 3px pointer travel | Speaker mute toggles; no azimuth change | Clean drag detection |
| HRTF Panning Interpolation | Engine receives `SetSpeakerAzimuth` | Dynamic 64-tap HRTF coefficients computed via linear interpolation between adjacent KEMAR anchors | Zero heap allocation; stack-only computation |
| Reset Speaker Positions | Click "Reset Speaker Positions" | All 8 nodes snap back to ITU-R BS.775 reference angles; `ResetSpeakerPositions` dispatched | Immediate update of all channels |
| Storage Persistence | Save config on modal close / azimuth change | `speaker_azimuths_deg` saved to `config.json`; restored on app launch | Atomic JSON write |

</intent-contract>

## Code Map

- `crates/open-boom-core/src/dsp/spatializer_3d.rs` -- Implement `compute_interpolated_hrtf`, dynamic `hrtf_left`/`hrtf_right` coefficient banks in `Spatializer3D`, and update `set_speaker_azimuth` / `reset_speaker_positions`.
- `crates/open-boom-core/tests/spatializer_tests.rs` -- Add unit tests for HRTF azimuth interpolation, angle limits, and ITU-R reset.
- `src-ui/src/components/SoundstageConfiguratorModal.tsx` -- Pointer capture, live azimuth badge, distinguish drag from click, and "Reset Speaker Positions" button.
- `src-ui/src/App.tsx` -- Connect `handleResetSpeakerPositions` to ITU-R reference defaults and dispatch.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-core/src/dsp/spatializer_3d.rs` -- Add dynamic HRTF interpolation table computation -- Delivers `FR7`, `AD-1`, `AD-4`.
- `crates/open-boom-core/tests/spatializer_tests.rs` -- Add azimuth interpolation & reset tests -- Delivers test verification.
- `src-ui/src/components/SoundstageConfiguratorModal.tsx` -- Enhance pointer drag physics, angle HUD display, and action buttons -- Delivers `UX-DR5`.
- `src-ui/src/App.tsx` -- Align reference default angles and reset handler -- Delivers state sync.

**Acceptance Criteria:**
- Given the 8-channel polar soundstage modal, when the user drags a speaker node along its radial orbit ring, then the node slides smoothly and updates its real-time azimuth coordinate angle in the HUD display (`FR7`, `UX-DR5`).
- Given an updated azimuth angle for any speaker, when the command reaches the audio engine via the SPSC queue, then the HRTF interpolation tables dynamically adjust the binaural convolution filter coefficients for that channel without dropouts (`AD-1`, `AD-4`).
- Given custom speaker positions have been adjusted, when the user clicks "Reset Speaker Positions", then all 8 speaker nodes return to standard ITU-R BS.775 7.1 reference azimuth angles (`EXPERIENCE.md §4.7`).

## Spec Change Log

<!-- None: clean first-pass execution -->

## Review Triage Log

<!-- Empty until first review pass -->

## Verification

**Commands:**
- `npm run build` in `src-ui` -- expected: TypeScript compiles with 0 errors and Vite bundles successfully
- `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test` -- expected: all workspace tests pass

**Auto Run Results:**
- `npm run build` in `src-ui`: Exit 0 (vite build built in 1.68s, 0 errors, 0 warnings).
- `cargo test`: 95 unit and integration tests passed across all crates (11 in `spatializer_tests`, including dynamic HRTF azimuth interpolation, angle limits, ITU-R reset, and 360-degree rapid drag zero-dropout tests).

