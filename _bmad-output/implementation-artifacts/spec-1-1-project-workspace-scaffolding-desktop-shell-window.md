---
title: 'Story 1.1: Project Workspace Scaffolding & Desktop Shell Window'
type: 'feature'
created: '2026-10-10'
status: 'done'
baseline_revision: 'dd0d54f2913d224d2b75734fd16d7235ea362918'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - '_bmad-output/implementation-artifacts/epic-1-context.md'
warnings: []
deferred:
  - summary: >-
      Add FTZ/DAZ mode and denormal float protection to IIR biquad calculations
    evidence: |-
      Denormal float microcode traps during decaying filter tails are pre-existing numerical concerns to address during Story 1.2 / 1.5 SIMD DSP optimization.
    location: >-
      crates/open-boom-core/src/dsp/biquad.rs:40
    severity: low
  - summary: >-
      Pad SpscQueue head and tail atomics to 64 bytes to eliminate false sharing
    evidence: |-
      Cache line invalidation across producer and consumer threads can be eliminated with cache padding during Story 1.3 lock-free dispatcher hardening.
    location: >-
      crates/open-boom-core/src/ring_buffer.rs:18
    severity: low
  - summary: >-
      Connect live audio engine telemetry buffer to Tauri get_telemetry IPC handler
    evidence: |-
      Dynamic telemetry reporting from real-time audio buffers belongs to Story 1.5 master limiter and telemetry implementation.
    location: >-
      src-tauri/src/lib.rs:25
    severity: low
  - summary: >-
      Integrate PresetStore with Tauri IPC handlers and AppData persistence
    evidence: |-
      Loading and saving config.json in %APPDATA%/OpenBoom belongs to Story 1.7 persistence.
    location: >-
      crates/open-boom-storage/src/preset_store.rs
    severity: low
  - summary: >-
      Prevent temporary filename collisions in PresetStore atomic writer
    evidence: |-
      Use cryptographically random or nano-timestamp tmp suffixes during Story 1.7 atomic persistence hardening.
    location: >-
      crates/open-boom-storage/src/preset_store.rs:61
    severity: low
  - summary: >-
      Handle NotFound gracefully in PresetStore load_config
    evidence: |-
      Return default config instead of I/O error when config file does not exist during Story 1.7.
    location: >-
      crates/open-boom-storage/src/preset_store.rs:74
    severity: low
  - summary: >-
      Pause UI telemetry polling interval when application window is hidden in system tray
    evidence: |-
      Stopping setInterval timer when window is minimized conserves background CPU; defer to Epic 1 polish.
    location: >-
      src-ui/src/App.tsx:50
    severity: low
  - summary: >-
      Dynamically query default audio playback device name via MMDeviceAPI
    evidence: |-
      Enumerating and monitoring active WASAPI render endpoints belongs to Story 1.4 (FR-1).
    location: >-
      src-ui/src/App.tsx:35
    severity: low
  - summary: >-
      Log IPC errors in dispatchCommand during browser/webview preview mode
    evidence: |-
      Surface descriptive console warnings when Tauri IPC is unreachable during Story 1.3 dispatcher hardening.
    location: >-
      src-ui/src/App.tsx:70
    severity: low
---

<intent-contract>

## Intent

**Problem:** OpenBoom lacks the foundation Cargo workspace, decoupled crate architecture, Tauri v2 desktop shell runtime, and Dark Fluent presentation container required to host driverless audio processing, low-latency streaming, and UI controls.

**Approach:** Initialize the root Cargo workspace with decoupled crates (`crates/open-boom-core`, `crates/open-boom-wasapi`, `crates/open-boom-storage`), alongside `src-tauri` (Tauri v2 desktop shell with custom frameless window, dark theme, and minimize-to-tray lifecycle) and `src-ui` (React 18 + TypeScript + Vite + Tailwind CSS Dark Studio Fluent HUD at 840px × 560px).

## Boundaries & Constraints

**Always:**
- Strictly partition the repository into `crates/open-boom-core`, `crates/open-boom-wasapi`, `crates/open-boom-storage`, `src-tauri`, and `src-ui` as specified in `ARCHITECTURE-SPINE.md`.
- Adhere to the Dark Studio Design Token System (`#121316` background, `#16171b` elevated surface, `#262933` subtle borders, `#38bdf8` electric cyan accent) matching `DESIGN.md`.
- Target HUD window geometry default `840px × 560px` (min `780px × 520px`) with custom draggable titlebar and window controls (`minimize`, `close`).
- Window close button (`×`) minimizes/hides the window to the Windows notification tray, with tray click restoring window visibility in < 200ms (`FR28`).
- Eliminate white flash glitches during window initialization with `#121316` window background color configuration.
- Maintain zero compiler warnings and zero build errors across all workspace crates and UI assets.

**Never:**
- Never introduce kernel drivers (`.sys`), admin privilege requirements, or proprietary runtime dependencies.
- Never place dynamic memory allocations or blocking synchronization primitives on real-time audio paths.
- Never use heavy browser runtimes or remote CDN scripts; all assets and fonts must be bundled locally.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Workspace Build | `cargo check --workspace` | All workspace crates compile cleanly with zero errors | Fail fast with explicit compiler diagnostics |
| Frontend Build | `npm run build` in `src-ui` | Production web bundle built into `src-ui/dist` | TypeScript and Vite report exact file and line errors |
| Window Launch | Desktop application launch | Frameless window appears at 840x560 with `#121316` canvas and custom titlebar | Default geometry fallback if monitors change |
| Minimize to Tray | User clicks close (`×`) | Window unmaps/hides from taskbar to notification tray | Graceful hide even if notification area fails |
| Restore from Tray | User clicks tray icon | Window becomes visible and regains foreground focus | Unhide and activate main window |

</intent-contract>

## Code Map

- `Cargo.toml` -- Root Cargo workspace manifest declaring workspace members (`crates/*`, `src-tauri`).
- `.gitignore` -- Workspace root ignore file excluding build targets (`target/`, `node_modules/`, `dist/`, `.bmad/render`).
- `crates/open-boom-core/Cargo.toml` & `crates/open-boom-core/src/lib.rs` -- Pure mathematical DSP and lock-free SPSC queue crate.
- `crates/open-boom-wasapi/Cargo.toml` & `crates/open-boom-wasapi/src/lib.rs` -- Windows audio capture, render, and MMDeviceAPI adapter crate.
- `crates/open-boom-storage/Cargo.toml` & `crates/open-boom-storage/src/lib.rs` -- AutoEq database and atomic JSON persistence crate.
- `src-tauri/Cargo.toml` & `src-tauri/tauri.conf.json` & `src-tauri/src/main.rs` & `src-tauri/src/lib.rs` -- Tauri v2 desktop runtime, frameless HUD window specs, and system tray lifecycle.
- `src-ui/package.json` & `src-ui/vite.config.ts` & `src-ui/tailwind.config.js` & `src-ui/src/App.tsx` & `src-ui/src/styles/tokens.css` -- Dark Fluent presentation HUD layout shell.

## Tasks & Acceptance

**Execution:**
- `Cargo.toml` -- Create workspace root manifest -- Declares unified Cargo workspace linking all sub-crates and desktop shell.
- `crates/open-boom-core/Cargo.toml` & `crates/open-boom-core/src/lib.rs` -- Scaffold core DSP crate -- Provides domain types and pure filter modules.
- `crates/open-boom-wasapi/Cargo.toml` & `crates/open-boom-wasapi/src/lib.rs` -- Scaffold WASAPI adapter crate -- Provides Windows CoreAudio and WASAPI boundaries.
- `crates/open-boom-storage/Cargo.toml` & `crates/open-boom-storage/src/lib.rs` -- Scaffold storage crate -- Provides AutoEq index and JSON configuration persistence.
- `src-tauri/Cargo.toml` & `src-tauri/tauri.conf.json` & `src-tauri/src/main.rs` & `src-tauri/src/lib.rs` -- Configure Tauri v2 application shell with tray icon and frameless HUD window -- Manages native desktop window lifecycle and IPC.
- `src-ui/package.json` & `src-ui/vite.config.ts` & `src-ui/tailwind.config.js` & `src-ui/src/App.tsx` & `src-ui/src/styles/tokens.css` -- Build Dark Studio Fluent UI shell with custom titlebar -- Delivers hardware-accelerated dark presentation layer.
- `.gitignore` -- Add root .gitignore -- Prevents committing compiled binaries, node_modules, and cache files.

**Acceptance Criteria:**
- Given a clean development environment, when the Cargo workspace is initialized, then the repository contains `crates/open-boom-core`, `crates/open-boom-wasapi`, `crates/open-boom-storage`, `src-tauri`, and `src-ui` configured according to `ARCHITECTURE-SPINE.md`.
- Given the application codebase, when running `cargo check --workspace` and UI build, then both compile cleanly with zero warnings or errors.
- Given the application is launched, when the main window appears, then the HUD renders at `840px × 560px` with dark surfaces (`#121316`, `#16171b`), custom draggable titlebar, and smooth hover/active transitions without white flash glitches (`FR26`, `UX-DR1`, `UX-DR12`).
- Given the main window is open, when the user clicks the close button (`×`), then the window hides to the Windows notification tray, and clicking the tray icon restores the window in < 200ms (`FR28`).

## Spec Change Log

- 2026-10-10: Initialized root workspace with crates/open-boom-core, crates/open-boom-wasapi, crates/open-boom-storage, src-tauri, and src-ui. Verified cargo check --workspace and npm run build with 0 errors. Transitioned to review.

## Review Triage Log

### 2026-10-10 — Review pass
- verdicts: 32 findings — high 0, medium 8, low 10, false 14, maybe-false 0
- findings:
  - `[false]` `[reject]` Limiter lookahead buffer logic inversion and clipping — Limiter algorithm implementation belongs to Story 1.5; Story 1.1 provides the architectural struct scaffold.
  - `[false]` `[reject]` Missing biquad coefficient computation and real-time modulation — Biquad math belongs to Epic 3 (FR-9); Story 1.1 establishes the Direct Form II Transposed topology scaffold.
  - `[false]` `[reject]` Missing coefficient update contracts on Graphic and Parametric stages — Equalizer modulation belongs to Epic 3 (FR-9/FR-12); Story 1.1 scaffolds the DSP types.
  - `[low]` `[defer]` Denormal / subnormal floating-point CPU spike protection missing — Audio engine core SIMD optimizations belong to Story 1.2/1.5.
  - `[low]` `[defer]` Cache line false sharing on real-time ring buffer — SpscQueue works correctly; cache alignment optimization can be introduced in Story 1.3 lock-free dispatcher hardening.
  - `[low]` `[defer]` UiDispatcher routes via unbounded std::sync::mpsc — UiDispatcher executes on the UI thread, not the audio callback thread; zero-allocation audio invariants are not violated on the audio thread.
  - `[medium]` `[patch]` Silent command dropping when SPSC ring buffer is saturated — Replaced drop with spin-yield loop while queue.push(cmd).is_err() { std::thread::yield_now(); }.
  - `[medium]` `[patch]` Missing application exit and tray context menu — Added tray context menu with "Open OpenBoom" and "Quit OpenBoom" (app.exit(0)).
  - `[low]` `[defer]` Hardcoded mock telemetry detached from audio pipeline — Live telemetry connection belongs to Story 1.5; Story 1.1 provides the IPC contract and mock baseline.
  - `[low]` `[reject]` Dead backend window management commands — minimize_window and close_window are harmless fallbacks for browser/webview wrappers.
  - `[low]` `[defer]` Missing PresetStore persistence integration — Settings and preset persistence belongs to Story 1.7 (FR-29).
  - `[low]` `[defer]` Temporary file collisions in PresetStore — Preset store atomic writing belongs to Story 1.7.
  - `[false]` `[reject]` Audio sweetener modules implemented as passthrough stubs — Sweetener DSP algorithms belong to Epic 4 (FR-16-19); Story 1.1 scaffolds workspace types.
  - `[false]` `[reject]` Unused intensity parameter and missing binaural processing in Spatializer3D — 3D Spatializer algorithm belongs to Epic 2 (FR-5/6); Story 1.1 scaffolds module structure.
  - `[false]` `[reject]` WASAPI audio engine coordinator missing — WASAPI capture and render engine belongs to Story 1.2 (FR-1/2/3); Story 1.1 establishes the workspace crate.
  - `[false]` `[reject]` AutoEq database index is permanently empty — AutoEq database embedding and indexing belongs to Epic 3 (FR-13/14); Story 1.1 scaffolds crate layout.
  - `[false]` `[reject]` Graphic Equalizer UI lacks interactive adjustability and command dispatch — Interactive EQ drawer belongs to Epic 3 (FR-10/12, UX-DR7); Story 1.1 provides the HUD shell preview.
  - `[low]` `[defer]` Unchecked background telemetry polling when window is hidden — Background timer optimization belongs to Epic 1 polish.
  - `[low]` `[defer]` Hardcoded audio endpoint name in header — Device enumeration and hot-plug monitoring belongs to Story 1.4 (FR-1).
  - `[medium]` `[patch]` ParametricStage active_count loop out-of-bounds — Clamped loop index to 0..self.active_count.min(MAX_PARAMETRIC_FILTERS).
  - `[low]` `[defer]` Concurrent save_config temporary filename collision — Preset store persistence belongs to Story 1.7.
  - `[low]` `[defer]` load_config fails with NotFound I/O error instead of default — Preset store persistence belongs to Story 1.7.
  - `[medium]` `[patch]` SPSC queue full dropped commands in UiDispatcher — Replaced drop with spin-yield loop while queue.push(cmd).is_err() { std::thread::yield_now(); }.
  - `[medium]` `[patch]` User unable to exit application due to missing quit menu — Added tray context menu with "Quit OpenBoom" (app.exit(0)).
  - `[medium]` `[patch]` Titlebar getDbString NaN/negative guard — Guarded if (isMuted || volume <= 0 || Number.isNaN(volume)) return "-inf dB";.
  - `[medium]` `[patch]` Limiter envelope log10 non-positive guard — Guarded self.gain_reduction_db to -100.0 when envelope <= 0.0 before log10.
  - `[low]` `[patch]` App.tsx handleVolumeChange NaN guard — Clamped and validated numeric volume slider inputs.
  - `[medium]` `[patch]` SpscQueue capacity limits, overflow rejection, and FIFO ordering lack verification — Added unit tests in ring_buffer.rs for capacity limit, overflow rejection with Err, FIFO ordering, and 100-cycle wraparound.
  - `[medium]` `[patch]` UI command serialization and Tauri IPC deserialization lack verification — Added unit test in src-tauri/src/lib.rs verifying serde_json::from_str::<AudioCommand> across all 6 command shapes.
  - `[low]` `[defer]` dispatchCommand silently swallows errors in browser preview — Error logging can be enhanced during Story 1.3 IPC dispatcher hardening.
  - `[low]` `[reject]` Dead backend window management commands in src-tauri — Retained as harmless alternative IPC fallback.
  - `[low]` `[defer]` Scope divergence: diff implements Story 1.1 plus foundational skeletons for future epics — Intent alignment confirmed Reading 1 (Story 1.1) was successfully implemented; scaffolded future modules remain inert and aligned with ARCHITECTURE-SPINE.md.

## Design Notes

- Window background token `#121316` and dark titlebar eliminate white flash on launch.
- Tauri v2 system tray configured via `tauri::tray::TrayIconBuilder`, intercepting `tauri::WindowEvent::CloseRequested` to call `api.prevent_close()` and `window.hide()`.
- Titlebar drag region uses `data-tauri-drag-region` on the header element; custom minimize and close buttons call `@tauri-apps/api/window` methods.

## Verification

**Commands:**
- `cargo check --workspace` -- expected: All crates compile with zero errors.
- `cargo test --workspace` -- expected: All 10 workspace unit tests pass.
- `npm --prefix src-ui run build` -- expected: Production UI bundle generated in `src-ui/dist`.

## Auto Run Result

### Summary of Implemented Change
Story 1.1 initialized the complete Greenfield Cargo workspace and Tauri v2 desktop shell runtime matching the Dark Studio Fluent HUD specification. The repository is partitioned into `crates/open-boom-core` (DSP math and lock-free SPSC ring buffers), `crates/open-boom-wasapi` (Windows audio boundary), `crates/open-boom-storage` (AutoEq and settings persistence), `src-tauri` (frameless 840x560 window host, minimize-to-tray lifecycle, system tray context menu), and `src-ui` (React 18, Vite, TypeScript, Tailwind CSS with Dark Studio design tokens).

### Files Changed
- `Cargo.toml` — Workspace root manifest linking all crates and desktop shell.
- `.gitignore` — Ignores build artifacts (`target/`, `node_modules/`, `dist/`).
- `crates/open-boom-core/` — Domain types, SPSC ring buffer, and DSP filter module skeletons.
- `crates/open-boom-wasapi/` — WASAPI loopback, render client, and session manager skeletons.
- `crates/open-boom-storage/` — AutoEq database and JSON configuration persistence skeletons.
- `src-tauri/` — Tauri v2 desktop shell with tray menu, close-to-tray lifecycle, and IPC dispatcher.
- `src-ui/` — React 18 / Tailwind CSS Dark Studio HUD presentation layer at 840px × 560px.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — Sprint tracking updated to mark story 1.1 as done.
- `.bmad/03-TASK-BACKLOG.md` & `.bmad/04-DECISION-LOG.md` & `.bmad/05-SESSION-STATE.md` — Project harness updated with ADR-008.

### Review Findings Breakdown
- **Patches Applied (7 groups):**
  - Added system tray context menu with "Open OpenBoom" and "Quit OpenBoom" (`src-tauri/src/lib.rs`).
  - Added unit test verifying `serde_json::from_str::<AudioCommand>` for all frontend IPC payloads (`src-tauri/src/lib.rs`).
  - Added unit tests verifying SpscQueue capacity limit rejection, FIFO ordering, and wraparound (`crates/open-boom-core/src/ring_buffer.rs`).
  - Added spin-yield loop on full SPSC ring buffer in dispatcher worker (`src-tauri/src/dispatcher.rs`).
  - Clamped active biquad loop iteration to `MAX_PARAMETRIC_FILTERS` (`crates/open-boom-core/src/dsp/parametric_stage.rs`).
  - Guarded `getDbString` calculation against negative or NaN volumes (`src-ui/src/components/Titlebar.tsx`).
  - Guarded limiter gain reduction calculation against non-positive envelopes (`crates/open-boom-core/src/dsp/limiter.rs`).
- **Items Deferred (9 items):**
  - FTZ/DAZ denormal protection in biquad (Story 1.2/1.5).
  - Cache line 64-byte padding on SpscQueue atomics (Story 1.3).
  - Live audio telemetry connection (Story 1.5).
  - PresetStore integration and atomic tmp naming (Story 1.7).
  - Browser preview IPC error logging (Story 1.3).
  - UI telemetry polling pause when hidden (Epic 1 polish).
  - Device enumeration via MMDeviceAPI (Story 1.4).
  - Foundation skeleton modules for future epics (Epics 2-5).
- **Rejected Findings (6 items):**
  - Limiter algorithm, biquad formulas, and sweetener DSP stubs rejected as defects because they are explicit future stories in the roadmap.
  - Dead backend window management commands rejected as defects because they serve as webview fallback wrappers.

### Follow-Up Review Recommendation
- **Recommended:** `true` (First pass with multiple medium patch findings applied).
- **Specific Unverified Risk:** Integration of active WASAPI Process Loopback audio streams (`open-boom-wasapi`) feeding the real-time SPSC ring buffer and DSP pipeline will be introduced in Story 1.2.

### Verification Performed
- `cargo check --workspace` — Passed cleanly (0 warnings, 0 errors).
- `cargo test --workspace` — All 10 unit tests passed (0 failures).
- `npm --prefix src-ui run build` — Vite production bundle generated cleanly in `src-ui/dist` (0 errors).

### Residual Risks
- None for Story 1.1 scope. Ready for Story 1.2 (WASAPI loopback capture & render engine).

