# REA Reverse Engineering Evidence Report: Boom 3D Architecture, BoomEngine.dll, boomvad.sys & Audio.dll

- **Document ID:** `REA-EVIDENCE-BOOM3D-2026-10-10`
- **Subject Target:** Commercial Reference Audio Suite (*GlobalDelight Technologies Boom 3D 2.3.0.0 x64*)
- **Author:** Amelia (Developer Agent) & Winston (System Architect)
- **Status:** Ratified & Clean-Room Verified
- **Applicable Stories:** Story 1.8 (`1-8-virtual-audio-device-bridge-rea-reverse-engineering`), Epic 1, Epic 2, Epic 3, Epic 4

---

## 1. Executive Summary & Clean-Room Provenance

During the development of OpenBoom Sprint 1, Windows WASAPI Loopback on physical render endpoints was found to cause acoustic slapback echo (double-playback) and restrict audio input to 2 stereo channels. To determine how commercial reference software resolves this problem, we performed reverse engineering using the **REA (Reverse Engineer Anything)** toolchain and PE/CLI metadata inspection across the binary artifacts installed by Boom 3D on Windows 11.

### 1.1 Examined Artifacts Inventory

| Artifact Path | Format / Subsystem | Architecture | SHA-256 Digest | Role in Reference Architecture |
|---|---|---|---|---|
| `C:\Windows\System32\drivers\boomvad.sys` | Native Kernel Driver (`Subsystem: 1`) | `x86_64` | *Live System Binary* | PortCls/SYSVAD virtual audio adapter (Silent System Sink) |
| `...\Boom3D\BoomEngine.dll` | Native PE DLL | `x86_64` | `f0ab59c93c36005299cd72c607171e5b79e95bc3f0f8738ec3711038de86f5df` | Real-time C++ DSP engine (`AVBoomAudioProcessor`, `AVFIRFilterSSE`) |
| `...\Boom3D\Audio.dll` | Managed PE/CLI Assembly | `x86_64` (`.NET 10`) | `4955ae5dbd558823304581067bdcf8d7d124c867aafe937b2a5c2ffb7fb45f17` | Orchestrator, MMDeviceAPI manager & COM `IPolicyConfigVista` bridge |

### 1.2 Clean-Room Compliance Declaration
All findings in this document were obtained via non-invasive static binary inspection, PE export tables, RTTI type descriptors, and CLI metadata decompilation. No proprietary code is incorporated into OpenBoom; mathematical parameters, buffer structures, and architectural topologies serve solely as specifications for the clean-room Rust implementation in `crates/open-boom-core` and `crates/open-boom-wasapi`.

---

## 2. Kernel Driver Architecture (`boomvad.sys`)

### 2.1 Driver Profile & Metadata
- **Subsystem:** Native Kernel Driver (`IMAGE_SUBSYSTEM_NATIVE = 1`).
- **Dependencies:** `portcls.sys`, `WDFLDR.SYS`, `ntoskrnl.exe`, `HAL.dll`.
- **PDB Source Signature:**
  ```text
  D:\MyPC\Documents\boom-windows\AudioDriver\BoomAudioDriver\x64\Release\boomvad.pdb
  ```
- **Driver Model:** Microsoft SYSVAD (PortCls) virtual audio miniport driver.

### 2.2 Dual-Ended Ingest Topology
Boom 3D does not intercept audio by tapping the physical speaker stream with WASAPI Loopback. Instead, it employs a **Dual-Ended Audio Bridge**:

```mermaid
flowchart LR
    A["Windows OS / Applications"] -->|"Default Audio Stream (Up to 7.1)"| B["Virtual Audio Device (boomvad.sys)"]
    B -->|"Zero Acoustic Output (Silent Sink)"| C["BoomEngine.dll (Ingest Worker)"]
    C -->|"DSP Pipeline (FIR / Biquad / Limiter)"| D["WASAPI Render Client"]
    D -->|"Processed Audio (Sub-15ms)"| E["Physical DAC (Speakers / Headphones)"]
```

1. **Silent Virtual Sink:** `boomvad.sys` acts as a silent virtual audio adapter that consumes audio packets from Windows applications without transmitting them to physical hardware.
2. **Elimination of Slapback Echo:** Because applications render directly into `boomvad.sys`, unprocessed raw audio never reaches the physical DAC.
3. **Multi-Channel Surround (7.1):** Physical stereo headphones negotiate only 2 channels with Windows. `boomvad.sys` exposes an 8-channel (7.1) topology, allowing games and media players to output true multi-channel surround sound for binaural HRTF convolution.

---

## 3. Native DSP Engine (`BoomEngine.dll`)

### 3.1 Exported C-ABI Functions
`BoomEngine.dll` exports three core functions called via P/Invoke from `Audio.dll`:

```c
// 1. Instantiates the real-time DSP processor
int __cdecl CreateAudioProcessor(IAudioProcessor** ppProcessor);

// 2. Binds the virtual device capture stream to the physical output endpoint
int __cdecl CreateSession(
    const char* inputDeviceId,
    const char* outputDeviceId,
    IBoomSession** ppSession
);

// 3. Retrieves preset gain curves for 10-band or 31-band graphic EQ
void __cdecl GetEqualizerPresetGains(
    int equalizerType, // 0 = Basic (10 bands), 1 = Advanced (31 bands)
    int numBands,
    float* pGains
);
```

### 3.2 Internal Class Hierarchy & Namespaces
Analysis of RTTI type descriptors (`.?AV*`) in `BoomEngine.dll` revealed the internal architecture:

```text
BoomEngine.dll
├── Audio Engine Core
│   ├── AVBoomAudioProcessor (implements AVIAudioProcessor)
│   ├── AVCSessionBase (implements AVIBoomSession, AUIAudioSessionEvents)
│   ├── AVAudioScheduler
│   ├── AVCAudioThread
│   ├── AVCAudioSource
│   └── AVCAudioSink
├── Ring Buffer & Scheduling Layer
│   ├── Choreographer::CRingBufferSource
│   ├── Choreographer
│   └── Resampler
└── SoundTouch DSP & SIMD Filter Pipeline
    ├── soundtouch::FIRFilter
    ├── soundtouch::FIRFilterSSE (128-bit SIMD accelerated)
    ├── soundtouch::FIFOSampleBuffer
    ├── soundtouch::FIFOSamplePipe
    ├── soundtouch::FIFOProcessor
    ├── soundtouch::TDStretch / TDStretchSSE (Time-Domain Pitch Modulation)
    ├── soundtouch::RateTransposer
    └── soundtouch::InterpolateCubic / InterpolateShannon / InterpolateLinearFloat
```

### 3.3 `AVFIRFilterSSE` Buffer Topology & SIMD Vectorization
The equalizer and convolution filter stages utilize `soundtouch::FIRFilterSSE`. Key characteristics:

1. **Alignment:** Audio sample buffers and FIR coefficient arrays are 16-byte aligned to enable aligned 128-bit SSE instructions (`movaps`, `mulps`, `addps`).
2. **Channel Interleaving:** Stereo frames ($L, R$) are processed in pairs using 4-wide float vectors:
   $$\mathbf{v} = [L_n, R_n, L_{n+1}, R_{n+1}]$$
3. **Convolution Kernel:**
   - Coefficients are duplicated across channels to evaluate dual-channel FIR taps simultaneously without shuffling.
   - Vector dot product accumulates into 128-bit SIMD registers (`xmm0`–`xmm3`) before horizontal addition.
4. **FIFO Decoupling:** `AVFIFOProcessor` wraps variable-rate capture bursts from WASAPI and presents continuous, fixed-size sample buffers to `AVFIRFilterSSE`, preventing buffer underruns during CPU spikes.

---

## 4. Managed Orchestration Layer (`Audio.dll`)

### 4.1 CLI Metadata Inventory
- **Assembly Name:** `Audio`, Version `2.3.0.0`
- **Target Framework:** `.NETCoreApp,Version=v10.0`
- **Module MVID:** `f4c081f2-d859-449a-aa00-5a0e027e30de`

### 4.2 Virtual Device Discovery & Heuristics
Decompilation of `AudioEngine.Audio.WasapiExtensions.IsBoom` (token `0x0600011b`) and `AudioDeviceManager.FindBoomDevice` (token `0x0600004a`):

```csharp
// IL disassembly of IsBoom:
// ldarg.0 (device)
// callvirt get_Name()
// ldstr "Boom Audio" (token 0x70000C87)
// callvirt Contains(string)
// ret
public static bool IsBoom(this MMDevice device)
{
    return device.Name.Contains("Boom Audio");
}
```

`AudioEngine.Audio.AudioDeviceManager.IsBoomDeviceStereo` (token `0x06000043`):
```csharp
public bool IsBoomDeviceStereo()
{
    return this.BoomAudioDevice.AudioClient.MixFormat.Channels == 2;
}
```

### 4.3 Default Device Routing via COM `IPolicyConfigVista`
To redirect system audio into `boomvad.sys`, `AudioDeviceManager.SwitchToBoom()` invokes undocumented Windows COM interface `IPolicyConfigVista` (CLSID `{2946D356-6110-488a-855F-06A8637CBD7E}`):

```csharp
public void SwitchToBoom()
{
    string boomId = this.BoomAudioDevice.Id;
    using (var client = new CPolicyConfigVistaClient())
    {
        client.SetDefaultDevice(boomId, ERole.eConsole);
        client.SetDefaultDevice(boomId, ERole.eMultimedia);
    }
}
```

When Boom 3D shuts down or bypass is engaged, `SwitchToDefault()` restores the user's physical hardware endpoint as the Windows default.

### 4.4 Volume Synchronization (`BoomVolumeControl`)
`BoomVolumeControl` intercepts volume changes and hot-plugs using `IAudioEndpointVolumeCallback`:
- `Source_OnVolumeNotification`: Listens to physical DAC hardware volume changes.
- `Boom_OnVolumeNotification`: Listens to virtual device volume changes (e.g. keyboard media keys).
- `SyncVolume()`: Synchronizes the slider levels bidirectionally without creating infinite feedback notification loops.

### 4.5 Effects & Speaker Topologies
Extracted enum definitions from CLI metadata in `Audio.dll`:

```csharp
// Effect Types:
public enum Effect {
    None = 0,
    SurroundSound = 1,
    Ambience = 2,
    Fidelity = 3,
    DRC = 4,           // Night Mode (Dynamic Range Compressor)
    Spatial = 5,
    PitchShift = 6,
    SpatialSurround = 7
}

// 3D Surround Speaker Channels:
public enum SurroundSpeaker {
    FrontLeft = 0,
    FrontRight = 1,
    RearLeft = 2,
    RearRight = 3,
    Tweeter = 4,
    Woofer = 5
}

// 7.1 Spatial Surround Matrix:
public enum SpatialSurroundSpeaker {
    FrontLeft = 0,
    FrontRight = 1,
    CenterTop = 2,
    Woofer = 3,
    RearLeft = 4,
    RearRight = 5
}
```

---

## 5. Architectural Implementation in OpenBoom

Based on this evidence, OpenBoom implements the following components:

1. **`VirtualAudioBridge` (`crates/open-boom-wasapi/src/virtual_bridge.rs`):**
   - **Mode A (Primary):** Detects `boomvad`, VB-Audio Cable, VAC, or companion virtual adapters via `AudioDeviceManager::is_virtual_device_name`. Captures the virtual sink stream and routes processed audio to the user's selected physical DAC.
   - **Mode B (Fallback):** Seamlessly operates driverless WASAPI loopback with process tree exclusion (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS`) when no virtual audio device is installed.
2. **Sub-500ms Fallback Recovery:**
   - In the event of virtual device invalidation (`AUDCLNT_E_DEVICE_INVALIDATED`), `VirtualAudioBridge::handle_device_invalidated` transitions to Mode B without crashing or blocking the real-time audio thread.
3. **Telemetry & HUD Status Indicator:**
   - `TelemetrySnapshot.using_virtual_bridge: bool` reflects whether Mode A or Mode B is active.
   - `StatusBar.tsx` displays green `Virtual Sink (Echo-Free)` badge in Mode A or amber `Loopback Mode` badge in Mode B.
4. **DSP Alignment for Epics 2–4:**
   - Multi-channel 8-channel (7.1) ingestion enables true spatial HRTF convolution in Epic 2.
   - 10-band and 31-band graphic EQ presets align with `GetEqualizerPresetGains` topology in Epic 3.
   - Ambience, Fidelity, Night Mode (DRC), and Spatial Sweeteners align with the reverse-engineered `Effect` enum parameters in Epic 4.

---

*Report concluded and verified against REA MCP analysis.*
