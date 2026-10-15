//! Real-time CoreAudio stream pump.
//!
//! A dedicated worker thread drains frames published by the capture IOProc, runs the
//! OpenBoom DSP pipeline, and enqueues the result for the render IOProc. The thread
//! owns its own [`AudioProcessor`] and never holds a mutex across the audio path: the
//! SPSC command queue and sample rings are lock-free, and the telemetry cell is only
//! locked for a brief publish after each processed block.
//!
//! The loop paces on the capture clock — it spins briefly, then yields, then parks in
//! short bounded waits until the capture IOProc has published a full block — rather than
//! sleeping for a fixed delay. Sustained idleness marks the capture stream as stalled.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use open_boom_core::dsp::pipeline::AudioProcessor;
use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{AudioCommand, EngineError, EngineResult, TelemetrySnapshot};

use crate::coordinator::DEFAULT_COMMAND_QUEUE_CAPACITY;
use crate::realtime::{RealtimeGuard, DEFAULT_AUDIO_PRIORITY};
use crate::ring_buffer::SampleRing;

/// Engine interleave used by the sample rings (stereo Float32).
const ENGINE_CHANNELS: usize = 2;

/// Iterations to spin before yielding the CPU while waiting for capture data.
const SPIN_BEFORE_YIELD: u32 = 1024;
/// Iterations to yield before parking in short bounded waits.
const SPIN_BEFORE_PARK: u32 = 2048;
/// How long capture may stay idle before the stream is flagged as stalled.
const STALL_THRESHOLD: Duration = Duration::from_millis(250);

/// Configuration for the CoreAudio stream pump.
#[derive(Debug, Clone)]
pub struct StreamPumpConfig {
    /// Frames processed per iteration.
    pub block_frames: usize,
}

impl Default for StreamPumpConfig {
    fn default() -> Self {
        Self {
            block_frames: crate::DEFAULT_BUFFER_FRAMES as usize,
        }
    }
}

/// Active stream pump handle.
pub struct CoreAudioStreamPump {
    is_running: Arc<AtomicBool>,
    capture_stalled: Arc<AtomicBool>,
    worker_handle: Option<JoinHandle<()>>,
}

impl CoreAudioStreamPump {
    /// Spawns the real-time pump worker thread.
    pub fn start(
        capture_ring: Arc<SampleRing>,
        render_ring: Arc<SampleRing>,
        command_queue: Option<Arc<SpscQueue<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>>>,
        telemetry: Arc<Mutex<TelemetrySnapshot>>,
        sample_rate: u32,
        config: StreamPumpConfig,
    ) -> EngineResult<Self> {
        let is_running = Arc::new(AtomicBool::new(true));
        let capture_stalled = Arc::new(AtomicBool::new(false));
        let running = is_running.clone();
        let stalled = capture_stalled.clone();
        let block_frames = config.block_frames.max(1);
        let sample_frames = block_frames * ENGINE_CHANNELS;
        let sample_rate = sample_rate.max(1);

        let handle = std::thread::Builder::new()
            .name("open-boom-coreaudio-pump".to_string())
            .spawn(move || {
                // Real-time scheduling applies to this audio thread; macOS typically
                // denies elevated priorities to unprivileged processes, in which case
                // the guard simply reports active = false.
                let _realtime = RealtimeGuard::apply(DEFAULT_AUDIO_PRIORITY);

                let mut processor = AudioProcessor::new();
                processor.set_sample_rate(sample_rate);

                let mut input = vec![0.0f32; sample_frames];
                let mut output = vec![0.0f32; sample_frames];

                while running.load(Ordering::Relaxed) {
                    // Pace on the capture clock with bounded backoff: spin, then yield,
                    // then park in 1 ms slices so a stopped capture cannot burn a core.
                    let wait_start = Instant::now();
                    let mut spins = 0u32;
                    while running.load(Ordering::Relaxed)
                        && capture_ring.available() < sample_frames
                    {
                        if spins < SPIN_BEFORE_YIELD {
                            std::hint::spin_loop();
                        } else if spins < SPIN_BEFORE_PARK {
                            std::thread::yield_now();
                        } else {
                            std::thread::park_timeout(Duration::from_millis(1));
                        }
                        spins = spins.saturating_add(1);
                        if wait_start.elapsed() >= STALL_THRESHOLD {
                            stalled.store(true, Ordering::Relaxed);
                        }
                    }
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }
                    stalled.store(false, Ordering::Relaxed);

                    if let Some(queue) = &command_queue {
                        processor.drain_commands(queue);
                    }

                    let read = capture_ring.pop_frames(&mut input, ENGINE_CHANNELS) * ENGINE_CHANNELS;
                    if read == 0 {
                        continue;
                    }

                    processor.process_interleaved(&input[..read], &mut output[..read]);
                    // Whole-frame push; a full render ring records its own drop count.
                    render_ring.push_frames(&output[..read], ENGINE_CHANNELS);

                    // Brief publish only; the coordinator owns the single latency source.
                    if let Ok(mut cell) = telemetry.lock() {
                        *cell = processor.telemetry_snapshot();
                    }
                }
            })
            .map_err(|err| {
                EngineError::DeviceInitialization(format!(
                    "failed to spawn CoreAudio stream pump thread: {err}"
                ))
            })?;

        Ok(Self {
            is_running,
            capture_stalled,
            worker_handle: Some(handle),
        })
    }

    /// Stops the pump and waits for the worker thread to exit.
    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }

    /// Returns whether the pump is actively running.
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    /// Returns whether capture has been idle long enough to be considered stalled.
    pub fn is_capture_stalled(&self) -> bool {
        self.capture_stalled.load(Ordering::Relaxed)
    }
}

impl Drop for CoreAudioStreamPump {
    fn drop(&mut self) {
        self.stop();
    }
}
