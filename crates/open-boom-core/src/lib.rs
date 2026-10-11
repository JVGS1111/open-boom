//! # OpenBoom Core
//!
//! Real-time DSP engine coordinator, pure mathematical filter algorithms,
//! and lock-free SPSC queue boundaries.
//!
//! Operates with zero heap allocations on real-time processing paths and zero
//! proprietary kernel dependencies.

pub mod dsp;
pub mod ring_buffer;
pub mod types;

// Re-export common types
pub use dsp::biquad::{Biquad, BiquadCoefficients};
pub use dsp::graphic_eq_stage::GraphicEqStage;
pub use dsp::parametric_stage::ParametricStage;
pub use dsp::pipeline::AudioProcessor;
pub use ring_buffer::SpscQueue;
pub use types::{
    AudioCommand, AudioDeviceInfo, EngineError, EngineResult, ParametricFilterPod,
    ParametricFilterType, TelemetrySnapshot, EQ_Q, EQ_SUB_BUFFER_SIZE, GRAPHIC_EQ_BANDS,
    INTERNAL_SAMPLE_RATE, INTERNAL_SAMPLE_RATE_F32, ISO_31_BAND_FREQUENCIES,
    LIMITER_CEILING_DBFS, LIMITER_LOOKAHEAD_SAMPLES, MAX_EQ_GAIN_DB, MAX_PARAMETRIC_FILTERS,
    MIN_EQ_GAIN_DB, STEREO_CHANNELS, SURROUND_7_1_CHANNELS,
};
