//! # OpenBoom CoreAudio
//!
//! macOS CoreAudio HAL audio host adapter (Epic 10, CAP-1) implementing the
//! platform-agnostic [`open_boom_core::traits::AudioHostBackend`] boundary.
//!
//! Responsibilities:
//! - Capture Float32 48 kHz stereo from a CoreAudio input/virtual device through an
//!   `AudioDeviceCreateIOProcID` callback that publishes frames into a lock-free ring.
//! - Render the DSP result back to the physical output endpoint through a render IOProc.
//! - Enumerate and switch CoreAudio HAL devices, including the OS default output.
//! - Apply real-time thread scheduling (`pthread_setschedparam(SCHED_RR)`) to the pump.
//! - Provide a deterministic, hardware-free mock HAL so the whole pipeline is testable
//!   in headless CI without physical or virtual audio hardware.
//!
//! The crate depends on `coreaudio-sys` only on macOS and carries zero `windows-rs`
//! dependencies on every target.

pub mod capture;
pub mod coordinator;
pub mod device;
pub mod hal;
pub mod realtime;
pub mod render;
pub mod ring_buffer;
pub mod stream_pump;

pub use capture::{CaptureConfig, CoreAudioCapture};
pub use coordinator::{CoreAudioCoordinator, DEFAULT_COMMAND_QUEUE_CAPACITY};
pub use device::CoreAudioDeviceManager;
pub use hal::{AudioHal, HalStream, HalStreamConfig, MockHal};
pub use realtime::{set_realtime_priority, RealtimeGuard, DEFAULT_AUDIO_PRIORITY};
pub use render::{CoreAudioRender, RenderConfig};
pub use ring_buffer::SampleRing;
pub use stream_pump::{CoreAudioStreamPump, StreamPumpConfig};

#[cfg(target_os = "macos")]
pub use hal::CoreAudioHal;

/// Default internal sample rate in Hertz (48 kHz Float32 pipeline).
pub const DEFAULT_STREAM_RATE: u32 = 48_000;

/// Default interleaved channel count for the OpenBoom stereo boundary.
pub const DEFAULT_STEREO_CHANNELS: u16 = 2;

/// Default buffer size in frames (240 frames = 5 ms at 48 kHz).
pub const DEFAULT_BUFFER_FRAMES: u32 = 240;

/// Default SPSC sample-ring capacity per direction (power of two).
pub const DEFAULT_RING_CAPACITY_SAMPLES: usize = 16_384;
