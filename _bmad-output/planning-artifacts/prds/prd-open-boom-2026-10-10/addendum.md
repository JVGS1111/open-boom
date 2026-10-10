# OpenBoom - PRD Addendum

This document captures technical implementation mechanisms, architecture rationale, rejected alternatives, and reverse-engineering insights that inform the OpenBoom requirements but belong outside the core PRD narrative.

---

## 1. Technical Audio Pipeline & Interception Rationale

### Options-Considered Matrix: Audio Interception on Windows

| Mechanism | Pros | Cons | Verdict |
| :--- | :--- | :--- | :--- |
| **Custom Kernel Driver (`boomvad.sys` clone)** | Total control over audio stream; transparent to all apps. | Requires Microsoft WHQL signature (~$500/yr + EV cert); BSOD risk; breaks on major OS updates. | **Rejected** (Violates zero-friction open-source goal). |
| **Windows Audio Processing Object (APO)** | Injected directly into Windows Audio Service; zero-latency (<1ms); no separate capture stream. | Endpoint configuration breaks easily with driver updates; complex COM/C++ registration; fragile GUI sync. | **Deferred to Phase 3 / Secondary Mode**. |
| **WASAPI Process Loopback (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS`)** | User-mode only; 100% crash-safe; zero feedback loops (captures processes without re-capturing OpenBoom output); native on Windows 10 build 20348+ & Windows 11. | Requires Windows 10 build 20348+ or Windows 11. | **Selected as Primary Pipeline**. |
| **Virtual Audio Cable Integration (VB-Cable / Open-source Bridge)** | Universal compatibility for legacy Windows 10 builds (1903–2004). | Requires user to install third-party virtual audio device software. | **Supported as Legacy Fallback**. |

---

## 2. Reverse-Engineering Insights from Boom 3D (Reference Artifacts)

Extracted from analysis of installed Boom 3D assets (`c:\Users\joaov\Documents\GitHub\decompilator` and REA MCP tools):

1. **DSP Execution Layer (`BoomEngine.dll` / `AudioEffectComponent.dll`):**
   - The reference application implements audio processing routines utilizing SSE SIMD instructions (`AVFIRFilterSSE`, `AVBoomAudioProcessor`).
   - Standard processing buffer: 32-bit floating point PCM at 44.1 kHz or 48 kHz.
   - Channel configuration: Virtual 7.1 layout folded down to stereo headphones via binaural impulse response matrices.
2. **Channel Matrix Geometry (7.1 Virtualization):**
   - Front Left (FL): -30° azimuth
   - Front Right (FR): +30° azimuth
   - Center (C): 0° azimuth
   - Low Frequency Effects (LFE): Subwoofer mono sum with low-pass filter (80–120 Hz)
   - Surround Left (SL): -90° azimuth
   - Surround Right (SR): +90° azimuth
   - Rear Left (RL): -150° azimuth
   - Rear Right (RR): +150° azimuth
3. **Harmonic Exciter ("Fidelity"):**
   - Reference adds controlled 2nd and 3rd order harmonic saturation combined with a high-frequency exciter shelf (+3 to +6 dB above 8 kHz).
4. **Night Mode (Dynamic Range Compression):**
   - Reference employs a dual-detector RMS/peak envelope follower with variable attack (1–5ms), release (50–200ms), and a logarithmic soft knee to prevent clipping while raising quiet dialogue.
5. **AutoEq Database Format:**
   - Bundles pre-computed parametric equalization biquad filter coefficients (Frequency, Q, Gain) matching the open-source AutoEq repository for over 4,000 headphone models.
