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

/// Mandatory true peak limiter ceiling in dBFS (-0.2 dBFS).
pub const LIMITER_CEILING_DBFS: f32 = -0.2;

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
    /// Atomic 31-band graphic EQ snapshot and preamp attenuation offset.
    LoadGraphicEqSnapshot {
        gains_db: [f32; GRAPHIC_EQ_BANDS],
        preamp_offset_db: f32,
    },
    /// Configure one of the audio sweetener modules.
    SetSweetener {
        module_id: u8, // 0 = Ambience, 1 = Fidelity, 2 = Night Mode, 3 = Spatial Widener
        enabled: bool,
        amount: f32,
    },
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
