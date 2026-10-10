# Review: Tech Stack Reality-Check, Version Compatibility & Structural Seed

- **Review Target:** `ARCHITECTURE-SPINE.md` (OpenBoom Architecture Spine)
- **Reviewer Lens:** Tech Stack Reality-Check, Framework Versions & Seed Integrity
- **Date:** 2026-10-10
- **Status:** PASS WITH REVISIONS

---

## 1. Executive Summary

This review verified all committed stack choices, version numbers, framework fits, and structural seeds against actual package registries, official documentation, and system capabilities.

All named technologies exist, are properly pinned, and satisfy mutual Minimum Supported Rust Version (MSRV) and build requirements. However, two material issues were identified:
1. **Structural Path Inconsistency:** Discrepancy between `src-core/...` paths in the capability/layer mapping vs `crates/open-boom-.../` in the structural seed.
2. **Windows 10 Process Loopback Build Gating:** Windows 10 consumer editions top out at Build 19045, making a strict 20348+ check counterproductive.

---

## 2. Technology & Version Reality-Check Matrix

| Technology | Declared Version | Status | Reality-Check Findings |
| :--- | :--- | :--- | :--- |
| **Rust** | 1.81.0 | **VERIFIED** | Released Sept 5, 2024. Stabilized `core::error::Error`, `#[expect(lint)]`, and fast sorting algorithms. MSRV for Tauri v2 is 1.78.0 and `windows` 0.58.0 is 1.70.0; Rust 1.81.0 satisfies both. |
| **windows-rs** | 0.58.0 | **VERIFIED** | Published release on crates.io. Contains full Win32 Audio and CoreAudio COM interfaces. Requires explicit features: `Win32_Media_Audio`, `Win32_System_Com`, `Win32_System_Threading`, `Win32_Foundation`. |
| **Tauri v2** | 2.0.0 | **VERIFIED** | Stable GA released Oct 2, 2024. Clean desktop shell, WebView2 native integration, tray support, and low memory profile (~35 MB working set). |
| **Node.js** | 24.11.1 | **VERIFIED** | Present locally on host system (`v24.11.1`). Active LTS release line with bundled npm 11. |
| **React** | 18.3.1 | **VERIFIED** | Verified on npm (Apr 2024). Stable baseline without React 19 breaking changes. |
| **TypeScript** | 5.6.2 | **VERIFIED** | Verified on npm (Sept 2024). Strict null checks and type safety. |
| **Tailwind CSS** | 3.4.14 | **VERIFIED** | Verified on npm (Oct 2024). Latest stable Tailwind v3 release with complete PostCSS support. |
| **Lucide React** | 0.453.0 | **VERIFIED** | Verified on npm (Oct 2024). Full icon set for hardware sliders, dials, and mute buttons. |
| **WASAPI Loopback** | Build 19041+ / 22000+ | **REVISION NEEDED** | Build 20348 is Windows Server 2022 / Windows 11 Build 22000+. Windows 10 consumer 22H2 is Build 19045, but underlying API works on 19041+. |

---

## 3. Detailed Evaluation & Findings

### Finding 1: Divergence between Layer/Capability Map and Structural Seed (High Severity)
- **Observation:** Lines 87–93 (*Layer to Directory Mapping*) and lines 341–369 (*Capability → Architecture Map*) reference files using the path pattern `src-core/dsp/biquad.rs`, `src-core/adapters/wasapi/loopback_capture.rs`, etc. Meanwhile, lines 226–255 (*Structural Seed*) specify a multi-crate workspace structure:
  - `crates/open-boom-core/src/dsp/`
  - `crates/open-boom-wasapi/src/`
  - `crates/open-boom-storage/src/`
- **Impact:** Divergence risk during implementation. Different developers or coding agents will create duplicate or conflicting file paths.
- **Remediation:** Update lines 87–93 and lines 341–369 to reference `crates/open-boom-core/`, `crates/open-boom-wasapi/`, and `crates/open-boom-storage/`.

### Finding 2: WASAPI Process Loopback Version Check on Windows 10 (Medium-High Severity)
- **Observation:** AD-2 specifies:
  > *"On Windows 10 (build 20348+) and Windows 11, audio interception must use AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS with PROCESS_LOOPBACK_MODE_EXCLUDE_PROCESS_TREE... For legacy Windows 10 builds (1903–2004), audio routing falls back to an optional user-selected virtual audio cable device."*
  Windows 10 client versions (20H2, 21H1, 21H2, 22H2) use build numbers 19042, 19043, 19044, and 19045. They will never reach 20348.
- **Impact:** If code uses a strict `build >= 20348` check, all Windows 10 consumer systems will fall back to virtual audio cables, degrading user experience.
- **Remediation:** In AD-2 and the Stack table, clarify that the driverless process loopback API operates on Windows 10 Build 19041+ (2004+) through Windows 11 Build 22000+. The audio client should attempt loopback activation with `ActivateAudioInterfaceAsync` and only fall back to virtual cable if the activation fails with `E_NOTIMPL` or `E_INVALIDARG`.

### Finding 3: Tauri v2 and `windows-rs` Interoperability (Passed)
- **Observation:** Tauri v2 internally uses `tao` and `wry`, which pull in their own Windows bindings. `open-boom-wasapi` pulls in `windows = "0.58.0"`.
- **Impact Assessment:**
  1. The Hexagonal Decoupling defined in AD-1 and AD-7 strictly confines `windows` types inside `crates/open-boom-wasapi/`.
  2. The Tauri shell (`src-tauri`) interacts with the core engine via high-level commands, not raw COM pointers.
  3. Window handle interoperability (if required for custom window framing) uses `raw-window-handle` integer representations (`isize`), which avoid type collisions.
- **Verdict:** Compatible and well-isolated.

### Finding 4: Frontend & Workspace Build Orchestration (Low Severity)
- **Observation:** The Structural Seed places `package.json` inside `src-ui/`. Running `npm run tauri dev` or `cargo tauri dev` from the repository root requires the root `Cargo.toml` to list `src-tauri` in workspace members, and `src-tauri/tauri.conf.json` to point `frontendDist` to `../src-ui/dist` with `beforeDevCommand: "npm --prefix ../src-ui run dev"`.
- **Remediation:** Add a note or minimal root `package.json` with workspace delegation scripts for smooth developer experience.

---

## 4. Recommended Action Items for Spine Update

1. **Harmonize Paths:** In `ARCHITECTURE-SPINE.md`, change all `src-core/` paths to `crates/open-boom-.../src/`.
2. **Amend AD-2 Rule:** Change `"On Windows 10 (build 20348+) and Windows 11"` to `"On Windows 10 (build 19041+) and Windows 11 (build 22000+)"`.
3. **Amend Stack Table:** Update the Windows Audio Engine entry to `"Windows 10 Build 19041+ / Windows 11 Build 22000+"`.
