//! # OpenBoom WASAPI
//!
//! Windows OS audio boundary managing WASAPI Process Loopback capture,
//! MMCSS low-latency render client, cubic Hermite resampler, format conversion,
//! and MMDeviceAPI session manager.

pub mod coordinator;
pub mod device;
pub mod format_converter;
pub mod loopback_capture;
pub mod mmcss;
pub mod render_client;
pub mod resampler;
pub mod session_manager;
pub mod virtual_bridge;

pub use coordinator::{AudioEngineCoordinator, DEFAULT_COMMAND_QUEUE_CAPACITY};
pub use device::AudioDeviceManager;
pub use format_converter::{convert_samples_to_f32, SampleFormat};
pub use loopback_capture::{LoopbackCapture, LoopbackConfig, LoopbackMode};
pub use mmcss::{CallbackBudgetTracker, MmcssGuard};
pub use render_client::{RenderClient, RenderConfig};
pub use resampler::{Resampler, MAX_RESAMPLER_CHANNELS};
pub use session_manager::{ProcessAudioSession, SessionManager};
pub use virtual_bridge::{BridgeMode, VirtualAudioBridge, VirtualBridgeConfig};
