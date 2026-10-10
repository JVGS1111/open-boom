# Adversarial Architecture Review — OpenBoom Architecture Spine

- **Artifact Under Review:** `c:/Users/joaov/Documents/GitHub/open-boom/_bmad-output/planning-artifacts/architecture/architecture-open-boom-2026-10-10/ARCHITECTURE-SPINE.md`
- **Review Date:** 2026-10-10
- **Review Lens:** Adversarial Lens (bmad-review)
- **Verdict:** REVISE BEFORE SPRINT EXECUTION

---

## Adversary Attack Scenarios & Divergence Tests

### Scenario A: SPSC Command Queue & Serialization Clash
- **Unit 1 (`src-tauri` / IPC Bridge):** Adheres to diagram line 61 ("Commands (JSON / Envelopes)") and AD-4. Implements Tauri commands taking JSON messages from WebView2 across async worker threads and pushes serialized JSON strings / `serde_json::Value` envelopes into the command ring buffer.
- **Unit 2 (`crates/open-boom-core` / DSP Coordinator):** Adheres to AD-1 Rule 1 (strictly zero heap allocations under MMCSS). Implements a cache-line aligned ring buffer expecting flat `#[repr(C)]` POD structs.
- **Divergence:** Compiles incompatibly. Deserializing dynamic JSON inside the audio loop triggers heap allocations (`malloc`), priority inversions, and audio dropouts. Furthermore, concurrent Tauri async worker threads calling `push()` simultaneously on an SPSC queue violate the single-producer contract and corrupt memory pointers.

### Scenario B: WASAPI Process Loopback & Physical Passthrough Duplication
- **Unit 1 (`loopback_capture.rs`):** Implements AD-2 using `AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` with `PROCESS_LOOPBACK_MODE_EXCLUDE_PROCESS_TREE` on the default Windows render endpoint.
- **Unit 2 (`render_client.rs`):** Implements low-latency render output targeting the same default Windows render endpoint.
- **Divergence:** Windows Core Audio loopback capture is a passive tap. Applications (Spotify, Chrome) continue outputting directly to the physical DAC uninhibited. OpenBoom plays the captured and processed stream ~15ms later onto the same DAC. The user hears two overlapping signals (unprocessed dry + delayed wet), causing severe slapback echo and acoustic comb filtering.

### Scenario C: AutoEq Parametric Formulation vs 31-Band Graphic Topology
- **Unit 1 (`autoeq_db.rs` / `HeadphonePicker.tsx`):** Implements AD-6 and AD-9. Parses 4,000+ AutoEq profiles into arbitrary parametric biquad descriptors ($f_0$, $Q$, filter type, gain).
- **Unit 2 (`biquad.rs` / `Equalizer.tsx`):** Implements line 73 and FR-9 as a fixed 31-band graphic equalizer with fixed 1/3-octave ISO standard center frequencies and fixed Q values.
- **Divergence:** Parametric AutoEq profiles cannot be loaded into a fixed 31-band graphic EQ without losing calibration accuracy or causing shape mismatch. The spine fails to decouple the parametric calibration engine from the graphic equalizer.

---

## Canonical Findings

### Finding 1: Clashing Shared-Data Shapes Across Lock-Free Command Queue
- **location:** `ARCHITECTURE-SPINE.md:60-64`, `AD-1`, `AD-4`
- **trigger_condition:** Diagram specifies JSON/Envelopes across an SPSC queue, while AD-1 strictly forbids heap allocations on the consumer audio thread.
- **guard_snippet:** Define an explicit fixed-size, C-compatible POD enum in `crates/open-boom-core/src/types.rs` (`#[repr(C)] pub enum AudioCommand { SetBandGain { band: u8, gain_db: f32 }, SetSweetener { id: u8, intensity: f32 }, ... }`) and prohibit JSON string or dynamic envelope serialization over the audio ring buffer boundary.
- **potential_consequence:** Real-time audio thread invokes dynamic allocation during message deserialization, triggering thread descheduling, priority inversion, and audio crackling under MMCSS.

### Finding 2: Missing Passthrough Muting in Driverless WASAPI Process Loopback
- **location:** `ARCHITECTURE-SPINE.md:126-130` (`AD-2`), `291-300`
- **trigger_condition:** Process loopback capture on default endpoints does not intercept or silence direct hardware audio output from third-party applications.
- **guard_snippet:** Specify the isolation routing architecture: require either a dedicated virtual loopback endpoint or programmatic session muting via Core Audio `ISimpleAudioVolume` (verifying loopback taps pre-attenuation streams), preventing double-playback through the physical DAC.
- **potential_consequence:** End users experience double audio with a 10–15ms latency offset, making music and dialogue unlistenable due to severe slapback echo.

### Finding 3: Structural Incompatibility Between AutoEq Parametric Biquads and 31-Band Graphic EQ
- **location:** `ARCHITECTURE-SPINE.md:73`, `158-164` (`AD-6`), `314`, `349` (`FR-9`), `353` (`FR-13`)
- **trigger_condition:** Conflation of variable-parameter AutoEq curves (arbitrary frequencies and Q) with a fixed 31-band graphic biquad filter bank.
- **guard_snippet:** Architecturally separate the EQ processing block into two distinct stages: (1) `ParametricCorrectionStage` (up to 12 arbitrary second-order IIR biquads loaded from AutoEq) and (2) `GraphicEqStage` (fixed 31-band ISO center frequencies with fixed Q), exposing distinct command contracts for each.
- **potential_consequence:** AutoEq calibration curves are either quantized onto 31 fixed bands (distorting headphone frequency correction) or fail to compile against the fixed graphic EQ data structure.

### Finding 4: Conflicting Ownership and State-Mutation Paths for Preamp Auto-Gain Offset
- **location:** `ARCHITECTURE-SPINE.md:158-164` (`AD-6`), `144-147` (`AD-4`), `312`
- **trigger_condition:** Preamp stage automatically computes and mutates digital attenuation internally, violating AD-4's rule that UI maintains presentation state as single source of truth.
- **guard_snippet:** Mandate that `preamp_offset_db` is computed deterministically in the UI/Control State Store upon preset/profile selection and transmitted to the audio engine as part of the preset command, keeping the audio DSP stage strictly stateless with respect to presentation values.
- **potential_consequence:** UI HUD displays incorrect preamp gain values; engine and UI state diverge, causing abrupt gain jumps when user moves the volume or preamp slider.

### Finding 5: Parameter Smoothing Divergence on Biquad Filter Coefficients
- **location:** `ARCHITECTURE-SPINE.md:144-147` (`AD-4`), `158-160` (`AD-6`)
- **trigger_condition:** 10ms linear sample-by-sample ramp applied to biquad filters causes either massive trigonometric CPU overruns or catastrophic filter instability.
- **guard_snippet:** Mandate sub-buffer block-based coefficient updates (e.g. recalculating RBJ coefficients once every 32/64 samples) combined with Direct Form II Transposed filter topology to preserve internal delay states and prevent coefficient interpolation blowups.
- **potential_consequence:** Filter coefficients interpolated linearly sample-by-sample move poles outside the unit circle, generating infinite impulse blowups and dangerous full-scale acoustic blasting.

### Finding 6: Incompatible Channel Count Assumptions Between Ingress and 3D Spatializer
- **location:** `ARCHITECTURE-SPINE.md:134-138` (`AD-3`), `237`, `312-315`
- **trigger_condition:** AD-3 asserts channel mixing occurs strictly at capture ingress and render egress, while Stage 2 downmixes 7.1 surround into 2-channel binaural stereo.
- **guard_snippet:** Update AD-3 to explicitly state the internal channel layout contract: capture ingress normalizes input to 7.1 (8 channels); Stage 2 (3D Spatializer) downmixes from 8 channels to 2 channels (Stereo); all downstream stages (EQ, Sweeteners, Limiter) process invariant 2-channel stereo.
- **potential_consequence:** DSP buffer stride mismatch causes out-of-bounds memory indexing, channel dropping, or panic in subsequent stereo filter units.

### Finding 7: Latency Discontinuity and DC Pop During Master Peak Limiter A/B Bypass
- **location:** `ARCHITECTURE-SPINE.md:150-155` (`AD-5`), `344` (`FR-4`)
- **trigger_condition:** Master limiter introduces an unavoidable 2ms (96-sample) lookahead delay, while FR-4 A/B Bypass toggles between processed and dry signals.
- **guard_snippet:** Specify that A/B Bypass routes audio around filter stages (Preamp, Spatializer, EQ, Sweeteners) but keeps the signal routed through the 2ms lookahead delay buffer and master limiter, using an equal-power 5ms cross-fade to eliminate clicks.
- **potential_consequence:** Engaging or disengaging A/B Bypass produces an audible time-slip glitch and transient pop due to an instantaneous 96-sample phase jump.

### Finding 8: Unsynchronized COM Apartment Initialization in Session Mixer
- **location:** `ARCHITECTURE-SPINE.md:176-180` (`AD-8`), `248`, `305`
- **trigger_condition:** Calling `IAudioSessionManager2` and `ISimpleAudioVolume` from arbitrary Tauri async tokio worker threads without COM MTA/STA initialization.
- **guard_snippet:** Designate a dedicated Session Mixer worker thread in `open-boom-wasapi` initialized with `CoInitializeEx(None, COINIT_MULTITHREADED)` that manages the COM lifetime, session enumeration, and event callbacks, communicating with Tauri via an internal MPSC channel.
- **potential_consequence:** COM API calls fail intermittently with `CO_E_NOTINITIALIZED` or RPC threading exceptions when invoked from Tauri thread-pool threads.

### Finding 9: Concurrency Race Conditions on SPSC Command Queue from Tauri IPC
- **location:** `ARCHITECTURE-SPINE.md:118-122` (`AD-1`), `234`, `259`
- **trigger_condition:** Multi-threaded Tauri async command handlers concurrently pushing commands into a lock-free Single-Producer Single-Consumer queue.
- **guard_snippet:** Implement a single UI Command Dispatcher thread or MPSC front-end buffer that serializes multi-threaded Tauri command invocations into the real-time SPSC ring buffer.
- **potential_consequence:** Simultaneous UI events (e.g. slider drag + tray mute) corrupt the un-synchronized SPSC queue head pointer, causing memory corruption or dropped commands.

### Finding 10: SPSC Queue Overflow and Partial State Corruption During Preset Loading
- **location:** `ARCHITECTURE-SPINE.md:144-147` (`AD-4`), `234`, `351` (`FR-11`), `363` (`FR-23`)
- **trigger_condition:** Loading a preset requires emitting >40 discrete parameter updates simultaneously, overflowing a fixed-capacity SPSC ring buffer.
- **guard_snippet:** Introduce atomic snapshot command messages, such as `LoadEqSnapshot { gains: [f32; 31], preamp_offset_db: f32 }`, allowing entire multi-parameter states to be committed to the audio engine in a single atomic queue push.
- **potential_consequence:** Ring buffer fills up mid-preset; trailing parameters are dropped, leaving the audio engine in a broken, half-configured hybrid state.
