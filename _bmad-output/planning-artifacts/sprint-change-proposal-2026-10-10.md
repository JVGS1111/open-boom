# Sprint Change Proposal: Pivot from Pure Driverless WASAPI Loopback to Virtual Audio Device Architecture (PortCls / SYSVAD) & REA Reverse Engineering Integration

- **Proposal Date:** 2026-10-10
- **Project:** OpenBoom (`open-boom`)
- **Author:** Amelia (Developer Agent) & Winston (System Architect)
- **Status:** Approved (Ratified by Joaov)
- **Change Scope Classification:** Moderate-to-Major (Architecture Alignment & Backlog Refinement)

---

## 1. Issue Summary

### 1.1 Problem Statement
During the implementation and testing of Sprint 1 (specifically Story 1.2 *Driverless WASAPI Audio Ingest Engine* and preparation for Story 1.4 *Master Volume & Device Selection*), a fundamental architectural limitation of Windows audio was identified:

**Pure WASAPI Loopback on a physical render endpoint cannot intercept and process system-wide audio without causing acoustic duplication (slapback echo).**

When an application renders audio to the physical playback device (e.g., *Realtek High Definition Audio*), the physical hardware plays that audio immediately. While WASAPI Loopback (`AUDCLNT_STREAMFLAGS_LOOPBACK` or `AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS`) can tap that audio buffer in user space, feeding that stream through OpenBoom's DSP pipeline (spatializer, EQ, limiter) and rendering the processed signal back to the same physical device results in:
1. **Acoustic Duplication:** The listener hears both the raw unprocessed signal from the originating application and the processed signal delayed by buffer latency (~10–15ms), causing severe comb filtering and echo.
2. **Channel Limitation:** Physical stereo endpoints (headphones) only negotiate a 2-channel format in Windows, preventing Windows audio sessions and games from outputting true 5.1/7.1 surround sound to be processed by the 3D HRTF spatializer.

### 1.2 Discovery Context & Reverse Engineering Evidence
To understand how commercial reference software (*Boom 3D for Windows*) overcomes this limitation, we inspected the installed binary packages of Boom 3D (`GlobalDelightTechnologies.Boom3D`) using PE analysis and REA (Reverse Engineer Anything) principles:

1. **Kernel Driver Architecture (`boomvad.sys`):**
   - File location: `C:\Windows\System32\drivers\boomvad.sys`
   - Subsystem: Native Kernel Driver (`Subsystem: 1`)
   - Dependencies: `portcls.sys`, `WDFLDR.SYS`, `ntoskrnl.exe`, `HAL.dll`
   - Build path signature: `\MyPC\Documents\boom-windows\AudioDriver\BoomAudioDriver\x64\Release\boomvad`
   - *Finding:* Boom 3D uses a Microsoft SYSVAD-derived PortCls virtual audio adapter. It registers a virtual audio playback endpoint in Windows.

2. **Native Audio Processing Engine (`BoomEngine.dll`):**
   - File location: `C:\Program Files\WindowsApps\GlobalDelightTechnologies.Boom3D_2.3.0.0_x64__b7pwcagxvespy\Boom3D\BoomEngine.dll`
   - Exported C++ functions:
     - `CreateAudioProcessor`
     - `CreateSession`
     - `GetEqualizerPresetGains`
   - Internal DSP classes identified:
     - `AVBoomAudioProcessor`
     - `AVFIRFilter` / `AVFIRFilterSSE`
     - `AVFIFOProcessor`
     - `AVCSessionBase`
     - `AUIAudioSessionEvents`

3. **Managed Orchestration Layer (`Audio.dll`):**
   - Key types identified:
     - `AudioDeviceManager`
     - `FindBoomDevice` / `IsBoom` / `IsBoomDeviceStereo`
     - `BoomVolumeControl` / `Boom_OnVolumeNotification`
     - `DefaultAudioRenderDeviceChangedEventArgs`

### 1.3 Architectural Insight
Boom 3D operates via a **Dual-Ended Audio Bridge**:
- **Ingestion:** Windows sets the Virtual Audio Device (`boomvad`) as the System Default Playback Endpoint. All applications render multi-channel (up to 7.1) audio into this silent virtual sink.
- **Processing:** The Boom 3D core service reads PCM buffers from the virtual device, runs the real-time DSP pipeline (`BoomEngine.dll` / `AVBoomAudioProcessor`), and writes the enhanced audio directly to the user's selected physical hardware endpoint (speakers/headphones).
- **Volume & Endpoint Control:** `BoomVolumeControl` intercepts Windows master volume changes and hardware hot-plugs without dropping audio.

---

## 2. Impact Analysis

### 2.1 Epic Impact
| Epic | Planned Scope | Impact of Change | Action Required |
|---|---|---|---|
| **Epic 1: Fundação do Projeto & UI Shell** | Driverless WASAPI loopback capture | Moderate: Ingest pipeline must support Virtual Audio Device endpoint capture as primary mode, retaining loopback as fallback. | Update Story 1.2 and Story 1.4. Add Virtual Device management. |
| **Epic 2: 3D Spatializer & HRTF** | Downmixing 7.1 to 2.0 | **Highly Positive**: A virtual 7.1 audio sink allows Windows to deliver genuine 8-channel streams to OpenBoom. | Align HRTF matrix with 8-channel virtual endpoint layout. |
| **Epic 3: Equalizer Multibanda & AutoEq** | 31-band graphic + parametric EQ | Low-to-Moderate: Leverage reverse-engineered `AVFIRFilterSSE` and `GetEqualizerPresetGains` from `BoomEngine.dll`. | Incorporate decompiled biquad/FIR coefficient formulas. |
| **Epic 4: Dynamic Effects Rack** | 5 audio effects | Low: Implementations will be verified against `BoomEngine.dll` effect structures using REA MCP. | Document exact algorithm parameters from decompiled evidence. |
| **Epic 5: Per-App Volume Mixer** | Session-based attenuation | Low: Session management attaches to the virtual endpoint session mixer. | Verify session enumeration on the virtual device. |

### 2.2 Story Impact
- **Story 1.1 (`1-1-project-workspace-scaffolding-desktop-shell-window`):** Completed (`done`). No rework needed.
- **Story 1.2 (`1-2-driverless-wasapi-audio-ingest-output-engine`):** Completed (`done`). The core primitives—Hermite $C^1$ resampler, `FormatConverter`, and RAII `MmcssGuard`—are 100% reusable. Ingestion logic will be updated to prioritize capturing from the Virtual Audio Device endpoint.
- **Story 1.3 (`1-3-lock-free-pod-command-dispatcher-master-power-control`):** Completed (`done`). The lock-free SPSC POD command queue and bit-perfect bypass are untouched.
- **Story 1.4 (`1-4-master-volume-boost-control-device-selection`):** In Backlog (`backlog`). Scope will be updated to manage the Virtual Device (System Default) $\rightarrow$ Physical Output Device routing pairing and volume mirroring.
- **New Story 1.8 (`1-8-virtual-audio-device-bridge-rea-reverse-engineering`):** Add a dedicated story for virtual audio device lifecycle detection, driver communication bridge, and reverse engineering extraction via REA MCP.

### 2.3 Artifact Impact Matrix
- **PRD (`prd.md` & `addendum.md`):** Update system constraints and capture requirements to specify the Virtual Audio Device pipeline with driverless fallback.
- **Architecture (`01-ARCHITECTURE.md` & `ARCHITECTURE-SPINE.md`):** Update system topology, replace ADR-003 and ADR-005 with ADR-011, and refine the audio thread invariant `AD-2`.
- **UX Specifications (`DESIGN.md` & `EXPERIENCE.md`):** Add visual indicator for Virtual Device status in the device selector bar and driver installation onboarding prompt.
- **Decision Log (`04-DECISION-LOG.md`):** Record ADR-011 detailing the PortCls/SYSVAD virtual device paradigm.
- **Project Guidelines (`AGENTS.md`):** Formalize the active use of REA MCP and the `decompilator` repository for all DSP and driver analysis.

---

## 3. Recommended Approach

### 3.1 Dual-Mode Audio Pipeline Strategy
We recommend a **Dual-Mode Audio Architecture**:
1. **Mode A — Virtual Audio Device (Primary / Production Mode):**
   - OpenBoom integrates with a virtual audio endpoint (supporting existing virtual devices such as `boomvad`, Virtual Audio Cable, or an open-source PortCls companion driver).
   - The virtual device is set as Windows Default Playback Device.
   - Applications render clean, uncompressed multi-channel (up to 7.1) PCM audio directly into the virtual sink.
   - OpenBoom captures from the virtual device, executes the real-time DSP pipeline, and outputs to the selected physical DAC endpoint.
   - **Result:** Zero acoustic duplication, true multi-channel 7.1 spatialization, zero feedback loops, and 100% application compatibility.
2. **Mode B — Standalone Driverless Loopback (Fallback / Restricted Mode):**
   - Retained for environments where kernel drivers cannot be installed or are disabled by administrative policy.
   - Uses the existing `crates/open-boom-wasapi` loopback implementation.

### 3.2 REA MCP & Decompilator Knowledge Extraction Loop
- Use the global `rea` MCP server to inspect `BoomEngine.dll` (`CreateAudioProcessor`, `CreateSession`, `AVBoomAudioProcessor`, `AVFIRFilterSSE`) and `Audio.dll`.
- Reverse-engineer the exact ring-buffer FIFO mechanism (`AVFIFOProcessor`), filter coefficients, and HRTF convolution parameters to guide the Rust implementation in `crates/open-boom-core`.

---

## 4. Detailed Change Proposals

### 4.1 PRD Changes (`prd.md`)

```markdown
Section: 2.2 System Architecture & Audio Capture Requirements

OLD:
- Audio capture MUST be 100% driverless, utilizing Windows WASAPI Loopback (`AUDCLNT_STREAMFLAGS_LOOPBACK` / `AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS`) to eliminate kernel-mode driver dependencies.

NEW:
- Audio capture MUST implement a Dual-Mode Architecture:
  1. Primary Mode (Virtual Audio Device): Captures audio via a PortCls/SYSVAD virtual audio endpoint set as the system default playback device. Eliminates acoustic slapback echo and unlocks multi-channel (7.1) application audio rendering.
  2. Fallback Mode (Driverless Loopback): Captures audio via WASAPI Loopback with process exclusion for driverless or portable operation.
```

### 4.2 Architecture Changes (`ARCHITECTURE-SPINE.md` & `01-ARCHITECTURE.md`)

```markdown
Section: 1. System Topology & ADR Log

OLD (ADR-003 & ADR-005):
- Capture audio strictly through user-mode WASAPI Loopback to avoid kernel drivers.

NEW (ADR-011: Virtual Audio Device Pipeline & Clean-Room REA Alignment):
- Adopt the Dual-Ended Virtual Audio Device pipeline:
  [Applications] -> [Virtual Audio Device (Default Sink)] -> [OpenBoom AudioEngineCoordinator] -> [DSP Pipeline] -> [Physical DAC Render Endpoint].
- Use REA MCP to inspect Boom 3D binaries (`BoomEngine.dll`, `boomvad.sys`) to derive exact FIFO buffer topologies, SSE filter implementations, and volume notification interfaces without clean-room contamination.
```

### 4.3 Story 1.4 Specification Changes (`spec-1-4-...md`)

```markdown
Section: User Story & Acceptance Criteria

OLD:
- Story 1.4 controls volume boost and physical device selection on the direct WASAPI loopback stream.

NEW:
- Story 1.4 controls:
  1. Master Volume & 150% Boost with soft saturation.
  2. Dual Device Management: Enumerates and configures the Virtual Audio Device as the system default input sink while streaming processed output to the user's selected Physical Endpoint.
  3. Volume Synchronization: Synchronizes Windows hardware volume changes with the virtual device endpoint via `IAudioEndpointVolumeCallback`.
```

### 4.4 New Story 1.8 (`spec-1-8-virtual-audio-device-bridge-rea-reverse-engineering.md`)

```markdown
Story ID: 1-8-virtual-audio-device-bridge-rea-reverse-engineering
Title: Virtual Audio Device Bridge & REA Reverse Engineering Integration
Epic: Epic 1 (Fundação do Projeto & UI Shell)
Goal: Provide automated detection, pairing, and stream capture from the virtual audio device, alongside reverse-engineering scripts/evidence from `BoomEngine.dll` (`CreateAudioProcessor`, `AVBoomAudioProcessor`) using REA MCP.
Acceptance Criteria:
- AC-1.8.1: Detect whether a compatible virtual audio device (`boomvad` or equivalent virtual sink) is present on the system.
- AC-1.8.2: Implement `VirtualAudioBridge` in `crates/open-boom-wasapi` that captures the virtual endpoint render stream with zero acoustic feedback.
- AC-1.8.3: Provide REA analysis report in `.bmad/evidence/` documenting the exported interfaces of `BoomEngine.dll` (`CreateAudioProcessor`, `CreateSession`) and the `AVFIRFilterSSE` buffer structure.
- AC-1.8.4: Workspace tests verifying automatic fallback to driverless loopback if the virtual device is absent.
```

### 4.5 UI/UX Changes (`DESIGN.md` & `EXPERIENCE.md`)

```markdown
Section: Output Device Selector & Status Bar

OLD:
- Dropdown selector displaying only physical output devices.

NEW:
- Device Bar displays:
  - Active Output Device: Physical speaker/headphone selector.
  - Virtual Capture Status: Badge indicating "System Audio Captured via Virtual Sink" (Green) or "Fallback Loopback Mode" (Amber).
  - Setup Guide / Driver Prompt: Non-intrusive notification if no virtual audio endpoint is detected.
```

---

## 5. Implementation Handoff & Next Steps

### 5.1 Scope Classification
- **Classification:** Moderate (Backlog re-sequencing and adapter enhancement).
- **Justification:** Existing DSP engine, scaffolding, tests, and UI components remain 100% valid. The change enhances the ingestion adapter (`crates/open-boom-wasapi`) and device routing logic without disrupting the core pipeline.

### 5.2 Responsibility & Routing
1. **Developer Agent (Amelia):**
   - Update `crates/open-boom-wasapi` to incorporate virtual device endpoint capture and fallback logic.
   - Author Story 1.4 with dual-endpoint routing and volume synchronization.
   - Create Story 1.8 for the Virtual Audio Device bridge and REA analysis.
2. **System Architect (Winston):**
   - Record ADR-011 in `.bmad/04-DECISION-LOG.md`.
   - Update `ARCHITECTURE-SPINE.md` and `.bmad/01-ARCHITECTURE.md`.
3. **Product Manager (John):**
   - Update `prd.md` and `epics.md` to reflect the dual-mode ingestion model.
   - Update `sprint-status.yaml` and `.bmad/03-TASK-BACKLOG.md`.

---

*End of Proposal.*
