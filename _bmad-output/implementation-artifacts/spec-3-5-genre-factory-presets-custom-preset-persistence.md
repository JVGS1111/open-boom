---
title: 'Story 3.5: Genre Factory Presets & Custom Preset Persistence'
type: 'feature'
created: '2026-10-11'
status: 'done'
baseline_revision: 'c9e5da814b1d7234325c4fed6cae676fcb3d70a2'
review_loop_iteration: 1
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-3-context.md'
  - '_bmad-output/implementation-artifacts/spec-3-1-31-band-biquad-filter-engine-direct-form-ii-transposed-topol.md'
  - '_bmad-output/implementation-artifacts/spec-3-2-dual-mode-equalizer-studio-drawer-10-band-vs-31-band-spline.md'
  - '_bmad-output/implementation-artifacts/spec-3-3-offline-autoeq-headphone-database-indexing-instant-fuzzy-sea.md'
  - '_bmad-output/implementation-artifacts/spec-3-4-decoupled-parametric-autoeq-stage-auto-gain-preamp-compensat.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** Listeners want to switch instantly between genre-optimized acoustic curves (e.g., Gaming, Electronic, Classical, Movie, Rock) and save, rename, or delete their own custom 31-band equalizer curves without risk of file corruption or losing track of unsaved fader modifications.

**Approach:** Provide 15 factory presets (Flat, Bass Boost, Treble Boost, Acoustic, Classical, Electronic, Gaming, Hip-Hop, Jazz, Movie, Pop, Rock, Vocal, Lounge, Deep) with calibrated 31-band ISO 1/3-octave curves and implement atomic custom preset persistence in `crates/open-boom-storage` writing to `%APPDATA%\OpenBoom\presets\*.json` via temporary staging files and atomic rename (`MoveFileExW` on Windows) (`FR11`, `AD-10`). Expose Tauri IPC commands (`get_eq_presets`, `save_custom_eq_preset`, `rename_custom_eq_preset`, `delete_custom_eq_preset`) and build the interactive `PresetSelectorPopover` UI component supporting dirty state detection (`Rock *`), 1-click preset loading, and custom preset CRUD (`EXPERIENCE.md §5.2`).

## Boundaries & Constraints

**Always:**
- Provide all 15 required factory presets: Flat, Bass Boost, Treble Boost, Acoustic, Classical, Electronic, Gaming, Hip-Hop, Jazz, Movie, Pop, Rock, Vocal, Lounge, Deep (`FR11`).
- Detect modified/dirty fader state whenever the current 31-band gains deviate from the active preset's curve (by > 0.05 dB), displaying an asterisk next to the preset name (e.g. `Rock *`) and enabling "Save As Custom Preset" (`EXPERIENCE.md §5.2`).
- Write custom presets to `%APPDATA%\OpenBoom\presets\<slug>.json` using atomic temporary file writes (`.tmp`) and atomic replacement (`fs::rename` / `MoveFileExW`) (`FR11`, `AD-10`).
- Support saving, renaming, and deleting user custom presets while protecting factory presets from modification or deletion.
- Persist `active_preset_name` in `AppConfig` across restarts.
- Keep AutoEq headphone calibration stage (`ParametricStage`) completely independent when switching 31-band graphic EQ presets (`AD-6`).

**Never:**
- Never allow overwriting, renaming, or deleting any of the 15 built-in factory presets.
- Never allow path traversal or invalid filesystem characters in custom preset filenames (sanitize slugs strictly to alphanumeric and hyphens).
- Never leave orphaned `.tmp` files in `%APPDATA%\OpenBoom\presets\` on write failure.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Open Preset Selector | Click preset pill in EQ dock or drawer | Opens dropdown showing 15 Factory Presets and User Custom Presets | Escape or outside click closes |
| Select Factory Preset | Click "Rock" or "Gaming" | Updates 31-band faders, dispatches `SetEqGains` to audio engine, clears `*` indicator | Smooth sub-buffer transition |
| Modify Fader on Preset | Drag any fader while on "Rock" | Preset label displays `Rock *`; "Save As Custom Preset" button becomes active | Real-time dirty comparison |
| Save Custom Preset | Enter "My Late Night EQ" and click Save | Writes `%APPDATA%\OpenBoom\presets\my-late-night-eq.json` atomically; sets active preset to `My Late Night EQ` (no `*`) | Reject empty names or factory name collisions |
| Rename Custom Preset | Rename "My Late Night EQ" to "Warm Jazz" | Atomically writes new file, removes old file, updates active preset label | Reject empty or factory names |
| Delete Custom Preset | Click delete on custom preset | Removes `.json` file from `presets/`; if it was active, falls back to "Flat" or retains curve as `Flat *` | Prevent deleting factory presets |
| Corrupt Preset JSON | Malformed `.json` file in `presets/` | Logs warning and skips corrupt file without crashing `list_presets()` | Resilient directory scan |

</intent-contract>

## Code Map

- `crates/open-boom-storage/src/preset_store.rs` -- Define `EqPreset`, 15 factory presets with 31-band curves, `presets_dir()`, and atomic CRUD methods (`list_presets`, `save_custom_preset`, `rename_custom_preset`, `delete_custom_preset`). Add `active_preset_name` to `AppConfig`.
- `crates/open-boom-storage/src/lib.rs` -- Export `EqPreset` and factory preset helpers.
- `crates/open-boom-storage/tests/storage_tests.rs` -- Add unit/integration tests for 15 factory presets, atomic custom preset save/load/rename/delete, path sanitization, factory protection, and corrupt file resilience.
- `src-tauri/src/lib.rs` -- Expose Tauri IPC commands `get_eq_presets`, `save_custom_eq_preset`, `rename_custom_eq_preset`, and `delete_custom_eq_preset`.
- `src-ui/src/components/PresetSelectorPopover.tsx` -- Dropdown/modal component for browsing 15 factory presets + custom presets, dirty state indicator (`*`), "Save As Custom Preset" input form, and inline rename/delete controls.
- `src-ui/src/components/EqualizerPreview.tsx` & `src-ui/src/components/EqualizerStudioDrawer.tsx` -- Display active preset pill with dirty asterisk (`*`) and trigger `PresetSelectorPopover`.
- `src-ui/src/App.tsx` -- Manage `presets`, `activeEqPreset`, dirty state detection (`isPresetDirty`), preset selection, custom preset CRUD handlers, and `AppConfig` persistence.
- `src-ui/src/test/PresetSelector.test.tsx` -- Frontend unit and component tests verifying 15 factory presets, dirty asterisk detection, preset switching, and custom preset UI.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-storage` -- Implement 15 factory presets and atomic custom preset CRUD in `%APPDATA%\OpenBoom\presets\`.
- `src-tauri` -- Register and test IPC commands for preset listing, saving, renaming, and deleting.
- `src-ui` -- Implement `PresetSelectorPopover`, dirty state detection (`*`), and wire into `EqualizerPreview`, `EqualizerStudioDrawer`, and `App.tsx`.
- Verification -- Run Rust and UI test suites and full production build.

**Acceptance Criteria:**
- Given the preset selector dropdown on the main HUD or studio drawer, when opened, then at least 15 factory presets are available: Flat, Bass Boost, Treble Boost, Acoustic, Classical, Electronic, Gaming, Hip-Hop, Jazz, Movie, Pop, Rock, Vocal, Lounge, Deep (`FR11`).
- Given the user alters faders while on a preset, when changes occur, then the preset label displays an asterisk (e.g. `Rock *`) and enables "Save As Custom Preset" (`EXPERIENCE.md §5.2`).
- Given a custom preset is saved, renamed, or deleted, when executed, then `crates/open-boom-storage` writes to `%APPDATA%\OpenBoom\presets\` using atomic temporary-file writes and `fs::rename` (`MoveFileExW`) replacement (`FR11`, `AD-10`).

## Spec Change Log
- 2026-10-11: Created specification for Story 3.5: Genre Factory Presets & Custom Preset Persistence.
- 2026-10-11: Hardened `load_custom_preset` against factory preset shadowing and empty preset names; ensured `Escape` key capture in `PresetSelectorPopover` does not simultaneously dismiss the underlying `EqualizerStudioDrawer`; propagated Tauri IPC errors in `App.tsx` custom preset CRUD handlers.

## Review Triage Log
- **Round 1 (Blind Hunter / Edge Case Hunter / Verification Gap / Intent Alignment)**:
  - *Fix*: Hardened `PresetStore::load_custom_preset` to reject built-in factory preset identifiers (`PermissionDenied`) and normalize empty `name`/`category` fields on disk.
  - *Fix*: Registered `PresetSelectorPopover` Escape key listener in capture phase with `e.stopImmediatePropagation()` and checked `e.defaultPrevented` in `EqualizerStudioDrawer` so pressing `Escape` closes only the popover when opened from inside the studio drawer.
  - *Fix*: Updated `App.tsx` custom preset CRUD handlers (`handleSaveCustomPreset`, `handleRenameCustomPreset`, `handleDeleteCustomPreset`) to rethrow backend errors when running inside Tauri (`isTauriRuntime()`) so `PresetSelectorPopover` displays validation/filesystem errors to the user.
  - *Fix*: Added unit tests 7 and 8 in `src-ui/src/test/PresetSelector.test.tsx` for custom preset rename collision validation and closed popover accessibility attributes.

## Auto Run Result
- **Status**: `done`
- **Tests**: 122 Rust workspace tests passing (`cargo test`), 47 UI tests passing (`npm test`), production frontend bundle built cleanly (`npm run ui:build`).

## Verification

**Commands:**
- `cargo test -p open-boom-storage` -- expected: all factory and custom preset persistence tests pass (19 passed, 0 failed)
- `cargo test` -- expected: workspace tests pass with 0 failures (122 passed, 0 failed)
- `npm test` -- expected: all UI tests pass including PresetSelector suite (47 passed, 0 failed)
- `npm run ui:build` -- expected: TypeScript and Vite build cleanly with 0 errors (built in 1.76s)
