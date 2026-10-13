---
title: '1-Click Silent Setup, Try/Catch Error Handling & Manual Fallback Link'
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

**Problem:** Installing a virtual audio driver manually requires navigating third-party websites, unzipping archives, running elevated installers with custom switches, and configuring sound cards in Windows control panel. If anything fails (UAC denial, network timeout, corrupt download), users are left without working audio or guidance.

**Approach:** 
1. Implement a backend Tauri command `install_virtual_driver` that:
   - Downloads/prepares the WHQL-certified setup package in a temporary directory.
   - Executes `VBCABLE_Setup_x64.exe -i -h` with `-Verb RunAs` (standard UAC elevation prompt) and waits for completion.
   - Detects UAC cancellation (exit code / permission denied) and network errors, returning typed error messages.
   - Upon successful driver installation, automatically executes endpoint friendly name customization (`OpenBoom Spatial Audio (powered by VB-Cable)`) and 7.1 speaker mask injection (`0x63F`).
   - Supports synthetic and test environments for 100% reliable automated unit testing without real hardware modification.
2. In frontend `<DriverOnboardingModal />`:
   - Connect the primary 1-click install button to `install_virtual_driver`.
   - Display active loading spinner and progress status during installation.
   - Render error alert banner with specific failure reasons, retry button, and external manual download link on failure.
   - Automatically close modal and ungate controls on success.
3. Pass all Cargo workspace tests and React frontend tests.

## Boundaries & Constraints

**Always:**
- Keep digital signature WHQL integrity (never use unsigned or modified drivers).
- Encapsulate execution in try/catch and tagged `Result` without panics or crashes.
- Support synthetic fallback in tests.
- Pass all unit tests and local compilation gates.

**Never:**
- Redirect users to shady or unverified mirrors.
- Force background installations without user consent.
- Leave orphaned processes running in the background.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| User clicks 1-Click Install | Clicks button, approves UAC | Downloads, runs silently, injects 7.1 mask, un-gates UI | Returns `Ok(DriverInstallResult)` |
| User cancels UAC prompt | UAC prompt denied | Modal displays UAC denial message and retry/manual links | Caught and displayed in error alert |
| Network failure | Download fails/offline | Modal displays network error message and manual link | Caught and returned as readable error string |
| Synthetic/mock mode | `force_synthetic: true` | Instant success without spawning external processes | `Ok(DriverInstallResult)` with synthetic virtual endpoint |

</intent-contract>

## Code Map

- `src-tauri/src/lib.rs` -- Command `install_virtual_driver` and `DriverInstallResult` struct.
- `src-ui/src/App.tsx` -- Integration with `install_virtual_driver`, state handling, and un-gating on success.
- `src-ui/src/components/DriverOnboardingModal.tsx` -- Wiring 1-click button, loading state, error alerts, and manual fallback link.
- `src-ui/src/test/DriverOnboarding.test.tsx` -- Unit tests for installation states and error recovery.

## Tasks & Acceptance Criteria

- [x] **Task 1: Backend 1-Click Installer Routine:**
  - Implement `install_virtual_driver` command with UAC execution, error handling, post-install friendly name + 7.1 mask injection, and synthetic test fallback.
- [x] **Task 2: Frontend Wiring & Try/Catch Flow:**
  - Wire `handleInstallVirtualDriver` in `App.tsx` and `<DriverOnboardingModal />` with error banner, retry button, and fallback link.
- [x] **Task 3: Unit Tests & Verification:**
  - Add backend and frontend unit tests covering installation success, failure handling, and retry flows.
- [x] **Task 4: Mandatory Local Compilation & Smoke Check:**
  - Verify `npm --prefix src-ui run build` and `cargo build -p open-boom`.
  - Perform live smoke launch of `open-boom.exe`.
