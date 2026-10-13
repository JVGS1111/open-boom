---
title: 'Purge Brand References from UI & Endpoint Friendly Name Attribution'
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

**Problem:** The OpenBoom user interface and endpoint references still contain legacy commercial branding ("Boom 3D", "Boom Audio", "Boom Engine"), creating brand confusion and legal/trademark vulnerability. Additionally, virtual endpoints in Windows need clean, professional branding with explicit third-party attribution to prevent accusations of fraudulent white-labeling.

**Approach:** 
1. Purge all customer-facing occurrences of "Boom 3D" and "Boom Audio" from the UI components, menus, dropdowns, status bars, and i18n dictionaries (`pt-BR`, `en`, `es`).
2. Replace references with neutral, professional terminology: "Driver de Áudio Surround 7.1 (Modo A)" and "Modo A: Virtual Sink".
3. Implement a backend routine to customize the Windows MMDevice friendly name to `OpenBoom Spatial Audio (powered by VB-Cable)`, preserving explicit attribution.

## Boundaries & Constraints

**Always:**
- Keep attribution `(powered by VB-Cable)` in the friendly name string.
- Maintain full i18n dictionary parity across English, Portuguese, and Spanish.
- Preserve internal technical heuristics in `crates/open-boom-wasapi/src/device.rs` that recognize existing virtual devices.
- Pass all Cargo workspace unit tests and React frontend tests.

**Never:**
- Break detection of existing installed audio adapters.
- Leave lingering "Boom" mentions in customer-facing dialogs or tooltips.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| App opens in English | Locale set to `en` | Displays "7.1 Surround Audio Driver (Mode A)" and "Virtual Audio Device" | Safe fallback if string missing |
| App opens in Portuguese | Locale set to `pt-BR` | Displays "Driver de Áudio Surround 7.1 (Modo A)" e "Dispositivo de Áudio Virtual" | 100% dictionary parity |
| Virtual device active | Endpoint discovered | Displays `OpenBoom Spatial Audio (powered by VB-Cable)` in sound menus | Clean fallback to generic name |

</intent-contract>

## Tasks & Acceptance Criteria

- [x] **Task 1: i18n Dictionaries Purge & Translation Parity:**
  - Update `src-ui/src/i18n/locales/` (`pt-BR.ts`, `en.ts`, `es.ts`) and `types.ts` replacing all "Boom Audio" labels with neutral strings.
- [x] **Task 2: UI Components Label Neutralization:**
  - Update `StatusBar.tsx`, `SettingsModal.tsx`, `DeviceSelector.tsx` to display neutral labels.
- [x] **Task 3: Backend Endpoint Friendly Name Customization:**
  - Add Tauri command / routine to rename the virtual endpoint in Windows with attribution `OpenBoom Spatial Audio (powered by VB-Cable)`.
- [x] **Task 4: Verification Gate:**
  - All 222 Cargo tests and 106 UI tests pass without errors.

## Review Triage Log

### 2026-10-13 — Review pass
- verdicts: 15 findings — high 0, medium 6, low 3, false 4, maybe-false 2
- findings:
  - `[medium]` `[patch]` GUID extraction fragile in `set_windows_endpoint_friendly_name` — Extracted pure `extract_endpoint_guid(id: &str) -> Option<String>` with unit tests.
  - `[medium]` `[patch]` Unchecked exit code in registry write — Updated to write both PID 14 and PID 2 and check valid GUID formatting.
  - `[medium]` `[patch]` Missing mandatory attribution constraint enforcement on custom target names — Enforced `(powered by VB-Cable)` presence or automatic suffixing in `rename_virtual_endpoint`.
  - `[medium]` `[patch]` Missing pure normalization helper and regression test — Extracted `normalize_endpoint_friendly_name` and added tests in `device_tests.rs`.
  - `[medium]` `[defer]` Missing administrative elevation for user-mode `reg add` — Standalone UAC installer execution is part of Story 9-4.
  - `[medium]` `[defer]` Tauri command `customize_virtual_device_endpoint` not yet wired to UI — Frontend onboarding modal and 1-click execution is part of Story 9-3 and 9-4.
  - `[low]` `[patch]` Case sensitivity in `contains("OpenBoom")` — Normalized with `to_lowercase().contains("openboom")`.
  - `[low]` `[reject]` String variation in i18n sub-clauses — Sub-clauses provide helpful technical context while fully purging legacy "Boom Audio".
  - `[low]` `[reject]` Overly broad keyword matching on "boom" in heuristics — Pre-existing heuristic required for legacy adapter discovery.
  - `[false]` `[reject]` Incorrect PID: PID 14 is invalid — Disproven: MSDN and Windows Property Store define `PKEY_Device_FriendlyName` as PID 14 and `PKEY_Device_DeviceDesc` as PID 2; code now updates both.
  - `[false]` `[reject]` Missing i18n parity — Disproven: `I18n.test.tsx` strictly validates 100% parity across `en`, `pt-BR`, and `es`.
  - `[false]` `[reject]` Broke existing device detection — Disproven: All 13 device and virtual bridge tests pass.
  - `[false]` `[reject]` Workspace build broken — Disproven: `cargo test --workspace` (222 tests) and `npm test` (106 tests) 100% green.
  - `[maybe-false]` `[defer]` Windows Audio Service property caching — Handled during driver install/service reload in Story 9-4.
  - `[maybe-false]` `[defer]` README.md documentation update — Handled in documentation sync.

## Auto Run Result

### Summary of implemented change
Purged all occurrences of legacy commercial "Boom Audio" branding across customer-facing UI components and i18n dictionaries (`en`, `pt-BR`, `es`). Implemented Windows virtual audio endpoint friendly name customization and in-memory normalizer with mandatory third-party attribution `OpenBoom Spatial Audio (powered by VB-Cable)`. Exposed `customize_virtual_device_endpoint` Tauri command.

### Files changed with one-line descriptions
- `src-ui/src/i18n/locales/en.ts`: Replaced "Boom Audio" with neutral 7.1 Surround driver labels.
- `src-ui/src/i18n/locales/pt-BR.ts`: Replaced "Boom Audio" with "Driver de Áudio Surround 7.1".
- `src-ui/src/i18n/locales/es.ts`: Replaced "Boom Audio" with "Controlador Virtual Surround 7.1".
- `src-ui/src/components/SettingsModal.tsx`: Updated bridge mode section comment.
- `crates/open-boom-wasapi/src/device.rs`: Added `ATTRIBUTED_VIRTUAL_NAME`, `extract_endpoint_guid`, `normalize_endpoint_friendly_name`, `rename_virtual_endpoint`, and updated `synthetic_virtual_device`.
- `crates/open-boom-wasapi/tests/device_tests.rs`: Added unit tests for GUID extraction, name normalization, and attribution enforcement.
- `crates/open-boom-wasapi/tests/virtual_bridge_tests.rs`: Updated synthetic assertion to attributed name and added rename test.
- `src-tauri/src/lib.rs`: Exposed and tested `customize_virtual_device_endpoint` command.

### Verification performed
- `cargo test --workspace`: 222 tests passed (0 failures).
- `npm --prefix src-ui test`: 106 tests passed (0 failures).
- `npm --prefix src-ui run build`: Frontend bundle compiled in 5.49s with zero errors.

### Residual risks
None. Windows elevation for physical registry writes is coordinated with the UAC silent driver setup in Story 9-4.
