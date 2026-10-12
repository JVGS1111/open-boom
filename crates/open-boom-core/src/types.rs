//! Core domain types and Plain Old Data (POD) command contracts.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Standard internal pipeline sample rate in Hertz (48 kHz).
pub const INTERNAL_SAMPLE_RATE: u32 = 48_000;
pub const INTERNAL_SAMPLE_RATE_F32: f32 = 48_000.0;

/// Standard audio channels.
pub const STEREO_CHANNELS: usize = 2;
pub const SURROUND_7_1_CHANNELS: usize = 8;

/// Lookahead limiter delay in samples (2ms at 48 kHz = 96 samples).
pub const LIMITER_LOOKAHEAD_SAMPLES: usize = 96;

/// Equal-power A/B bypass crossfade duration in samples (5ms at 48 kHz = 240 samples).
pub const CROSSFADE_SAMPLES: usize = 240;

/// Mandatory true peak limiter ceiling in dBFS (-0.2 dBFS).
pub const LIMITER_CEILING_DBFS: f32 = -0.2;

/// Sweetener module IDs.
pub const SWEETENER_AMBIENCE: u8 = 0;
pub const SWEETENER_FIDELITY: u8 = 1;
pub const SWEETENER_NIGHT_MODE: u8 = 2;
pub const SWEETENER_SPATIAL: u8 = 3;

/// Maximum graphic EQ bands (ISO 1/3-octave standard).
pub const GRAPHIC_EQ_BANDS: usize = 31;

/// Maximum parametric biquad filters in AutoEq correction stage.
pub const MAX_PARAMETRIC_FILTERS: usize = 12;

/// ISO standard center frequencies for 31-band graphic equalizer (Hz).
pub const ISO_31_BAND_FREQUENCIES: [f32; GRAPHIC_EQ_BANDS] = [
    20.0, 25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0, 125.0, 160.0, 200.0, 250.0, 315.0, 400.0,
    500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0, 4000.0, 5000.0, 6300.0,
    8000.0, 10000.0, 12500.0, 16000.0, 20000.0,
];

/// Standard constant Q factor for ISO 1/3-octave equalizer filters.
pub const EQ_Q: f32 = 4.31;

/// Minimum allowable graphic EQ band gain in decibels (-12.0 dB).
pub const MIN_EQ_GAIN_DB: f32 = -12.0;

/// Maximum allowable graphic EQ band gain in decibels (+12.0 dB).
pub const MAX_EQ_GAIN_DB: f32 = 12.0;

/// Number of casual octave bands for 10-band graphic equalizer mode.
pub const CASUAL_EQ_10_BANDS: usize = 10;

/// ISO standard center frequencies for 10-band graphic equalizer (Hz).
pub const ISO_10_BAND_FREQUENCIES: [f32; CASUAL_EQ_10_BANDS] = [
    31.5, 63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

/// Mapping of 10-band octave sliders to their corresponding 31-band ISO indices.
pub const ISO_10_TO_31_BAND_INDICES: [usize; CASUAL_EQ_10_BANDS] = [
    2, 5, 8, 11, 14, 17, 20, 23, 26, 29,
];

/// Sub-buffer parameter smoothing block size in samples (32 samples = ~0.67ms at 48 kHz).
pub const EQ_SUB_BUFFER_SIZE: usize = 32;

/// Standard ITU-R BS.775 7.1 channel indices:
pub const CH_FRONT_LEFT: usize = 0;
pub const CH_FRONT_RIGHT: usize = 1;
pub const CH_CENTER: usize = 2;
pub const CH_LFE: usize = 3;
pub const CH_REAR_LEFT: usize = 4;
pub const CH_REAR_RIGHT: usize = 5;
pub const CH_SURROUND_LEFT: usize = 6;
pub const CH_SURROUND_RIGHT: usize = 7;

/// ITU-R BS.775 7.1 default reference azimuth angles (degrees).
pub const ITU_R_7_1_DEFAULT_AZIMUTHS: [f32; SURROUND_7_1_CHANNELS] = [
    -30.0,   // Front Left
    30.0,    // Front Right
    0.0,     // Center
    0.0,     // LFE (Subwoofer)
    -135.0,  // Rear Left
    135.0,   // Rear Right
    -90.0,   // Surround Left
    90.0,    // Surround Right
];

/// Motion direction for 8D orbital soundstage rotation.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OrbitDirection {
    Clockwise = 0,
    CounterClockwise = 1,
    PingPong = 2,
}

impl Default for OrbitDirection {
    fn default() -> Self {
        Self::Clockwise
    }
}

/// Filter type for parametric biquad filters.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ParametricFilterType {
    Peaking = 0,
    LowShelf = 1,
    HighShelf = 2,
}

/// Plain Old Data representation of a parametric biquad filter.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParametricFilterPod {
    pub filter_type: ParametricFilterType,
    pub frequency_hz: f32,
    pub q: f32,
    pub gain_db: f32,
}

impl Default for ParametricFilterPod {
    fn default() -> Self {
        Self {
            filter_type: ParametricFilterType::Peaking,
            frequency_hz: 1000.0,
            q: 0.7071,
            gain_db: 0.0,
        }
    }
}

/// Flat, lock-free Plain Old Data (POD) commands transferred across the SPSC boundary.
/// Memory layout is fixed and contains zero heap pointers or dynamically sized types.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AudioCommand {
    /// Enable or disable master DSP pipeline processing (bit-perfect passthrough fallback).
    SetMasterPower(bool),
    /// Adjust linear master volume (0.0 to 1.0) and software boost factor (1.0 to 1.5).
    SetMasterVolume { volume: f32, boost: f32 },
    /// Instant mute toggle.
    SetMute(bool),
    /// Phase-continuous instant A/B bypass toggle.
    SetBypass(bool),
    /// 3D Surround spatializer intensity (0.0 to 1.0) and enable switch.
    SetSurround3D { enabled: bool, intensity: f32 },
    /// Configure 8D orbital soundstage rotation mode.
    SetOrbit8D {
        enabled: bool,
        direction: OrbitDirection,
        period_sec: f32,
        depth: f32,
    },
    /// Mute or unmute an individual virtual speaker channel (0..7).
    SetSpeakerMute { channel: u8, muted: bool },
    /// Adjust relative trim gain for an individual virtual speaker in dB (-12.0 to +12.0 dB).
    SetSpeakerGain { channel: u8, gain_db: f32 },
    /// Adjust azimuth angle for an individual virtual speaker in degrees (-180.0 to +180.0 deg).
    SetSpeakerAzimuth { channel: u8, azimuth_deg: f32 },
    /// Reset all virtual speaker azimuth angles to ITU-R BS.775 reference positions.
    ResetSpeakerPositions,
    /// Atomic 31-band graphic EQ snapshot and preamp attenuation offset.
    LoadGraphicEqSnapshot {
        gains_db: [f32; GRAPHIC_EQ_BANDS],
        preamp_offset_db: f32,
    },
    /// Adjust gain for an individual graphic EQ band (0..30) in dB (-12.0 to +12.0 dB).
    SetEqBandGain {
        band_idx: usize,
        gain_db: f32,
    },
    /// Set all 31 graphic EQ band gains atomically.
    SetEqGains {
        gains_db: [f32; GRAPHIC_EQ_BANDS],
    },
    /// Enable or disable bypass for the graphic EQ stage.
    SetEqBypass {
        bypassed: bool,
    },
    /// Atomic parametric AutoEq profile snapshot and auto-gain preamp attenuation offset.
    SetAutoEqProfile {
        filters: [ParametricFilterPod; MAX_PARAMETRIC_FILTERS],
        filter_count: usize,
        preamp_offset_db: f32,
    },
    /// Enable or disable bypass for the parametric AutoEq stage.
    SetAutoEqBypass {
        bypassed: bool,
    },
    /// Configure one of the audio sweetener modules.
    SetSweetener {
        module_id: u8, // 0 = Ambience, 1 = Fidelity, 2 = Night Mode, 3 = Spatial Widener
        enabled: bool,
        amount: f32,
    },
    /// Configure peak-hold buffer guard on the master limiter and audio pump.
    SetBufferGuard(bool),
    /// Reset or flush all internal filter delay lines.
    ResetFilters,
}

/// Real-time engine telemetry snapshot readable by UI without locking.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TelemetrySnapshot {
    /// Master limiter gain reduction in decibels (<= 0.0 dB).
    pub limiter_gain_reduction_db: f32,
    /// Measured round-trip latency in milliseconds.
    pub round_trip_latency_ms: f32,
    /// Peak output level on Left channel in dBFS.
    pub peak_output_l_dbfs: f32,
    /// Peak output level on Right channel in dBFS.
    pub peak_output_r_dbfs: f32,
    /// Whether the limiter is actively attenuating peaks.
    pub limiter_active: bool,
    /// Whether master DSP is currently active.
    pub dsp_active: bool,
    /// Limiter headroom to -0.2 dBFS ceiling in decibels (>= 0.0 dB).
    pub limiter_headroom_db: f32,
    /// Active audio stream sample rate in Hertz (e.g. 48000).
    pub sample_rate: u32,
    /// Active audio stream bit depth (e.g. 32 for 32-bit float).
    pub bit_depth: u32,
    /// Whether capture is operating via a virtual audio device bridge (Mode A)
    /// instead of standalone WASAPI loopback fallback (Mode B).
    pub using_virtual_bridge: bool,
    /// 3D Surround spatializer intensity (0.0 to 1.0).
    pub surround_intensity: f32,
    /// Whether 3D Surround spatializer is currently active.
    pub surround_enabled: bool,
    /// AutoEq preamp attenuation offset in decibels (<= 0.0 dB).
    pub autoeq_preamp_db: f32,
    /// Whether AutoEq parametric stage is currently active.
    pub autoeq_active: bool,
    /// Current 8D orbital rotation angle in degrees (-180.0 to +180.0 deg).
    pub orbit_phase_deg: f32,
    /// Whether 8D orbital soundstage rotation mode is currently active.
    pub orbit_enabled: bool,
}

impl Default for TelemetrySnapshot {
    fn default() -> Self {
        Self {
            limiter_gain_reduction_db: 0.0,
            round_trip_latency_ms: 10.0,
            peak_output_l_dbfs: -100.0,
            peak_output_r_dbfs: -100.0,
            limiter_active: false,
            dsp_active: false,
            limiter_headroom_db: 20.0,
            sample_rate: INTERNAL_SAMPLE_RATE,
            bit_depth: 32,
            using_virtual_bridge: false,
            surround_intensity: 0.5,
            surround_enabled: false,
            autoeq_preamp_db: 0.0,
            autoeq_active: false,
            orbit_phase_deg: 0.0,
            orbit_enabled: false,
        }
    }
}

/// Information about an enumerated audio output render endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

impl AudioDeviceInfo {
    pub fn new(id: impl Into<String>, name: impl Into<String>, is_default: bool) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            is_default,
        }
    }
}

/// Information about an enumerated per-process audio session.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProcessAudioSession {
    pub process_id: u32,
    pub name: String,
    pub volume: f32,
    pub is_muted: bool,
    pub is_active: bool,
}

impl ProcessAudioSession {
    pub fn new(
        process_id: u32,
        name: impl Into<String>,
        volume: f32,
        is_muted: bool,
        is_active: bool,
    ) -> Self {
        Self {
            process_id,
            name: name.into(),
            volume,
            is_muted,
            is_active,
        }
    }
}

/// Persisted volume preference profile for a specific application executable.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AppVolumeProfile {
    pub process_name: String,
    pub volume: f32,
    pub is_muted: bool,
}

impl AppVolumeProfile {
    pub fn new(process_name: impl Into<String>, volume: f32, is_muted: bool) -> Self {
        Self {
            process_name: process_name.into(),
            volume: volume.clamp(0.0, 1.5),
            is_muted,
        }
    }
}

/// Engine errors conforming to the tagged Result algebra.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Audio device initialization failed: {0}")]
    DeviceInitialization(String),

    #[error("Audio format unsupported: {0}")]
    UnsupportedFormat(String),

    #[error("Buffer underrun or overrun: {0}")]
    BufferError(String),

    #[error("Command queue is full")]
    QueueFull,

    #[error("Internal DSP computation error: {0}")]
    DspError(String),
}

pub type EngineResult<T> = Result<T, EngineError>;
