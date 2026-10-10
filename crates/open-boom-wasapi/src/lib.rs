//! # OpenBoom WASAPI
//!
//! Windows OS audio boundary managing WASAPI Process Loopback capture,
//! MMCSS low-latency render client, resampler, and MMDeviceAPI session manager.

pub mod loopback_capture;
pub mod render_client;
pub mod resampler;
pub mod session_manager;

pub use loopback_capture::{LoopbackCapture, LoopbackConfig};
pub use render_client::RenderClient;
pub use resampler::Resampler;
pub use session_manager::{ProcessAudioSession, SessionManager};
