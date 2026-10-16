---
id: SPEC-virtual-driver-onboarding
companions:
  - technical-driver-integration.md
sources:
  - .bmad/04-DECISION-LOG.md
  - .bmad/03-TASK-BACKLOG.md
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Virtual Audio Driver Onboarding, 7.1 Surround Automation, Gated UI & Anti-Trademark Attribution

## Why

To deliver pristine spatial audio without acoustic duplication (`BUG-MODE-B-01`), OpenBoom requires an isolated virtual audio sink. Historically coupled to Boom 3D's proprietary `boomvad.sys`, the application must transition to an autonomous, legally unassailable open-source distribution model. By providing a 1-click assisted onboarding flow for a WHQL-signed virtual driver, gating application controls until a driver is present, automating 8-channel 7.1 surround mask configuration, and strictly attributing third-party technology (`powered by VB-Cable`), OpenBoom eliminates user friction and trademark liability while guaranteeing driver integrity and system stability.

## Capabilities

- **CAP-1**
  - **intent:** System customizes the friendly name of the virtual endpoint in Windows Audio Engine to a clean branded format with explicit attribution (`OpenBoom Spatial Audio (powered by VB-Cable)`), while purging all legacy commercial references to "Boom 3D" and "Boom Audio" from the UI.
  - **success:** Windows Sound Control (`mmsys.cpl` and Windows Settings) displays `OpenBoom Spatial Audio (powered by VB-Cable)` as the playback endpoint name, and all application menus and status indicators display neutral terms ("Driver de Áudio Surround 7.1" / "Modo A: Virtual Sink") with 0 occurrences of "Boom" in customer-facing UI.

- **CAP-2**
  - **intent:** System automatically injects the 8-channel physical speaker mask (`0x63F`) into the virtual endpoint's property store without requiring manual configuration in the Windows Sound control panel.
  - **success:** Querying `PKEY_AudioEndpoint_PhysicalSpeakers` on the virtual endpoint returns `0x0000063F` (`KSAUDIO_SPEAKER_7POINT1_SURROUND`), and WASAPI stream initialization successfully negotiates 8 distinct audio channels (Float32 / PCM, 48 kHz).

- **CAP-3**
  - **intent:** System provides a 1-click assisted onboarding flow in the UI to download and silently install the WHQL-signed virtual audio driver when no compatible virtual device is detected.
  - **success:** User clicking "Instalar Dispositivo Surround 7.1" triggers background download, standard UAC elevation, silent driver installation via `-i -h`, and automatic endpoint activation within 15 seconds without external browser redirects.

- **CAP-4**
  - **intent:** On application startup, if no compatible virtual driver is detected, main audio controls are disabled (grayed out) and an Onboarding Modal automatically opens prompting 1-click driver installation.
  - **success:** Interactive controls (Master Switch, Volume Slider, Soundstage Dial, EQ Studio, Effects Rack) enter a visually disabled grayed-out state (`opacity-50 pointer-events-none`) and the Driver Onboarding Modal is displayed automatically on launch until a compatible driver is detected.

- **CAP-5**
  - **intent:** System encapsulates driver installation in robust try/catch error handling, displaying a clear error dialog and offering manual driver download links if automatic installation fails.
  - **success:** Simulated or real installer failure (e.g., UAC rejection, network timeout) catches the error, displays an error alert with descriptive diagnostic text, provides a clickable external manual download link to the driver website, and offers a "Tentar Novamente" (Retry) action.

## Constraints

- **WHQL Signature Integrity:** The driver package must maintain valid Microsoft WHQL digital signatures; no unsigned `.sys` files or `testsigning` modes that would compromise system integrity or stability are permitted.
- **Legal Attribution & Trademark Safety:** The renamed endpoint must explicitly retain `(powered by VB-Cable)` to avoid false white-labeling claims or trademark infringement.
- **Fail-Safe Physical DAC Routing:** System must guarantee immediate restoration of the user's physical audio device when Master Power is switched off, on application shutdown, or during uninstallation.
- **Direct Windows API Integration:** Endpoint renaming and speaker mask configuration must be executed via Windows Core Audio `IPropertyStore` and the Windows MMDevice registry keys without requiring third-party configuration tools.
- **Mandatory Live Host Smoke Verification:** Final verification must include a real test on the developer/user host machine: removing existing virtual audio drivers and verifying end-to-end 1-click installation and 7.1 activation.

## Non-goals

- Authoring, compiling, or maintaining an in-house kernel-mode audio driver (`.sys`) requiring an annual EV Code Signing certificate.
- Modifying, reverse-engineering, or repackaging the internal binary code of third-party drivers.
- Supporting legacy 32-bit (x86) Windows architectures.

## Success signal

A developer/user removes existing VB-Audio drivers from their PC, opens OpenBoom, observes all controls grayed out with the Driver Onboarding Modal active, clicks "Instalar com 1 Clique", approves UAC, and within 15 seconds observes the endpoint configured as `OpenBoom Spatial Audio (powered by VB-Cable)` with 7.1 channels (`0x63F`), modal dismisses, controls un-gray, and pristine spatial audio plays without echo. In case of UAC cancellation, an error modal shows the manual download link.
