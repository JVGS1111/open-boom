---
title: 'First-Run Driver Detection Gating & Controls Lockdown'
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

**Problem:** When a user launches OpenBoom for the first time without an installed virtual audio driver, operating the interactive controls (Power Switch, Volume, 3D Dial, EQ, Effects Rack) without an isolated virtual sink leads to silent failure, device initialization errors, or unintended fallback to double-rendering Mode B (`BUG-MODE-B-01`).

**Approach:** 
1. Implement a backend Tauri command `check_virtual_driver_status` returning driver presence and endpoint details.
2. On frontend startup, query `check_virtual_driver_status`.
3. If no virtual driver is installed, set `isDriverMissing = true` in app state.
4. Visually and interactively gate controls (Master Power Switch, Volume Slider, Soundstage Dial, Equalizer Preview / Studio Drawer trigger, and Effects Rack Strip) by applying `opacity-40 pointer-events-none select-none cursor-not-allowed` and explanatory tooltips.
5. Automatically open the `<DriverOnboardingModal />` on first launch when no driver is detected, allowing user to view driver requirements and prepare for 1-click installation.
6. Provide i18n parity across `en.ts`, `pt-BR.ts`, and `es.ts`.

## Boundaries & Constraints

**Always:**
- Ensure non-elevated boot queries never fail or throw unhandled exceptions.
- Provide full i18n parity in English, Portuguese, and Spanish.
- Allow users to dismiss or reopen `<DriverOnboardingModal />` via a persistent top banner or status indicator when the driver is missing.
- Pass all Cargo workspace unit tests and React frontend tests.

**Never:**
- Allow audio processing controls to be interacted with when `isDriverMissing` is true.
- Crash or enter infinite loops during status queries.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| First launch without driver | Virtual driver absent | `has_virtual_driver: false`, controls locked (`opacity-40 pointer-events-none`), `<DriverOnboardingModal />` opens | Clean fallback, no unhandled exceptions |
| Launch with driver installed | Virtual driver present | `has_virtual_driver: true`, controls enabled, modal stays closed | Standard startup flow |
| User dismisses modal without driver | Modal closed | Top warning banner / badge remains visible, controls remain locked | User can click banner to re-open modal |
| Synthetic test environment | Synthetic mode active | Configurable virtual driver status simulation | Unit tests pass reliably |

</intent-contract>

## Code Map

- `crates/open-boom-wasapi/src/device.rs` -- Method `check_virtual_driver_presence(&self) -> (bool, Option<AudioDeviceInfo>)`.
- `src-tauri/src/lib.rs` -- Tauri command `check_virtual_driver_status` returning `VirtualDriverStatus`.
- `src-ui/src/types.ts` -- Type definition `VirtualDriverStatus`.
- `src-ui/src/i18n/locales/` (`en.ts`, `pt-BR.ts`, `es.ts`) -- Translation keys for gating, banners, and `<DriverOnboardingModal />`.
- `src-ui/src/components/DriverOnboardingModal.tsx` -- Onboarding modal component with driver status, explanation, and action triggers.
- `src-ui/src/App.tsx` -- Boot status check, `isDriverMissing` state, gating classes on controls, and modal rendering.
- `src-ui/src/test/DriverOnboarding.test.tsx` -- Frontend unit tests for gating and modal rendering.

## Tasks & Acceptance Criteria

- [x] **Task 1: Backend Driver Status Query:**
  - Implement `check_virtual_driver_status` in `src-tauri/src/lib.rs` and `crates/open-boom-wasapi/src/device.rs`.
- [x] **Task 2: i18n Localization Parity:**
  - Add translations for onboarding modal, driver status, and gating tooltips in `en.ts`, `pt-BR.ts`, and `es.ts`.
- [x] **Task 3: Driver Onboarding Modal Component:**
  - Create `src-ui/src/components/DriverOnboardingModal.tsx` with Fluent Dark design, explanations, 1-click button, and manual fallback link.
- [x] **Task 4: App Integration & Controls Gating:**
  - Wire boot detection in `App.tsx`, apply gating classes to Master Switch, Volume, 3D Dial, EQ, and Rack, and auto-open modal.
- [x] **Task 5: Verification Gate:**
  - Cargo unit tests and React frontend tests pass 100%.
