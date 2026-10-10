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
pub use ring_buffer::SpscQueue;
pub use types::{
    AudioCommand, EngineError, EngineResult, TelemetrySnapshot, GRAPHIC_EQ_BANDS,
    INTERNAL_SAMPLE_RATE, INTERNAL_SAMPLE_RATE_F32, ISO_31_BAND_FREQUENCIES,
    LIMITER_CEILING_DBFS, LIMITER_LOOKAHEAD_SAMPLES, MAX_PARAMETRIC_FILTERS, STEREO_CHANNELS,
    SURROUND_7_1_CHANNELS,
};
