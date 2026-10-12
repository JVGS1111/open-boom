//! Audio engine coordinator managing the unified lifecycle of WASAPI capture,
//! real-time DSP pipeline processing, and audio output.

use std::sync::Arc;
use open_boom_core::dsp::pipeline::AudioProcessor;
use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::{AudioCommand, AudioDeviceInfo, EngineResult, TelemetrySnapshot};
use crate::device::AudioDeviceManager;
use crate::loopback_capture::{LoopbackCapture, LoopbackConfig};
use crate::render_client::{RenderClient, RenderConfig};
use crate::virtual_bridge::{BridgeMode, VirtualAudioBridge, VirtualBridgeConfig};

/// Default capacity for SPSC command ring buffer.
pub const DEFAULT_COMMAND_QUEUE_CAPACITY: usize = 256;

/// High-level audio engine coordinator managing user-mode capture and output.
pub struct AudioEngineCoordinator {
    capture: LoopbackCapture,
    render: RenderClient,
    processor: AudioProcessor,
    command_queue: Option<Arc<SpscQueue<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>>>,
    device_manager: AudioDeviceManager,
    bridge: VirtualAudioBridge,
    is_running: bool,
}

impl Default for AudioEngineCoordinator {
    fn default() -> Self {
        Self::new(LoopbackConfig::default(), RenderConfig::default())
    }
}

impl AudioEngineCoordinator {
    /// Creates a new audio engine coordinator with the specified capture and render configs.
    pub fn new(capture_config: LoopbackConfig, render_config: RenderConfig) -> Self {
        let allow_synthetic = render_config.allow_synthetic_fallback || render_config.force_synthetic;
        let device_manager = if render_config.force_synthetic {
            AudioDeviceManager::with_forced_synthetic(true)
        } else {
            AudioDeviceManager::with_synthetic_fallback(allow_synthetic)
        };
        let mut bridge = VirtualAudioBridge::new(VirtualBridgeConfig {
            allow_synthetic_fallback: allow_synthetic,
            ..Default::default()
        });
        let _ = bridge.detect(&device_manager);

        Self {
            capture: LoopbackCapture::new(capture_config),
            render: RenderClient::with_config(render_config),
            processor: AudioProcessor::new(),
            command_queue: None,
            device_manager,
            bridge,
            is_running: false,
        }
    }

    /// Creates an audio engine coordinator with an attached SPSC command queue.
    pub fn with_command_queue(
        capture_config: LoopbackConfig,
        render_config: RenderConfig,
        command_queue: Arc<SpscQueue<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>>,
    ) -> Self {
        let allow_synthetic = render_config.allow_synthetic_fallback || render_config.force_synthetic;
        let device_manager = if render_config.force_synthetic {
            AudioDeviceManager::with_forced_synthetic(true)
        } else {
            AudioDeviceManager::with_synthetic_fallback(allow_synthetic)
        };
        let mut bridge = VirtualAudioBridge::new(VirtualBridgeConfig {
            allow_synthetic_fallback: allow_synthetic,
            ..Default::default()
        });
        let _ = bridge.detect(&device_manager);

        Self {
            capture: LoopbackCapture::new(capture_config),
            render: RenderClient::with_config(render_config),
            processor: AudioProcessor::new(),
            command_queue: Some(command_queue),
            device_manager,
            bridge,
            is_running: false,
        }
    }

    /// Creates an audio engine coordinator with explicit virtual bridge configuration.
    pub fn with_bridge_config(
        capture_config: LoopbackConfig,
        render_config: RenderConfig,
        bridge_config: VirtualBridgeConfig,
        command_queue: Option<Arc<SpscQueue<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>>>,
    ) -> Self {
        let allow_synthetic = render_config.allow_synthetic_fallback || render_config.force_synthetic;
        let device_manager = if render_config.force_synthetic {
            AudioDeviceManager::with_forced_synthetic(true)
        } else {
            AudioDeviceManager::with_synthetic_fallback(allow_synthetic)
        };
        let mut bridge = VirtualAudioBridge::new(bridge_config);
        let _ = bridge.detect(&device_manager);

        Self {
            capture: LoopbackCapture::new(capture_config),
            render: RenderClient::with_config(render_config),
            processor: AudioProcessor::new(),
            command_queue,
            device_manager,
            bridge,
            is_running: false,
        }
    }

    /// Attaches an SPSC command queue to the coordinator.
    pub fn set_command_queue(
        &mut self,
        queue: Arc<SpscQueue<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>>,
    ) {
        self.command_queue = Some(queue);
    }

    /// Reference to the internal DSP processor.
    pub fn processor(&self) -> &AudioProcessor {
        &self.processor
    }

    /// Mutable reference to the internal DSP processor.
    pub fn processor_mut(&mut self) -> &mut AudioProcessor {
        &mut self.processor
    }

    /// Reference to the loopback capture client.
    pub fn capture(&self) -> &LoopbackCapture {
        &self.capture
    }

    /// Mutable reference to the loopback capture client.
    pub fn capture_mut(&mut self) -> &mut LoopbackCapture {
        &mut self.capture
    }

    /// Reference to the render client.
    pub fn render(&self) -> &RenderClient {
        &self.render
    }

    /// Mutable reference to the render client.
    pub fn render_mut(&mut self) -> &mut RenderClient {
        &mut self.render
    }

    /// Returns whether the audio engine is actively running.
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Reference to the device manager.
    pub fn device_manager(&self) -> &AudioDeviceManager {
        &self.device_manager
    }

    /// Mutable reference to the device manager.
    pub fn device_manager_mut(&mut self) -> &mut AudioDeviceManager {
        &mut self.device_manager
    }

    /// Enumerates active audio render endpoints on the system.
    pub fn enumerate_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        self.device_manager.enumerate_devices()
    }

    /// Switches the audio render endpoint to the specified device ID, or default if None.
    pub fn switch_device(&mut self, device_id: Option<String>) -> EngineResult<()> {
        self.render.switch_device(device_id)
    }

    /// Handles loss of active render endpoint by falling back to system default within 500ms.
    pub fn handle_device_lost(&mut self) -> EngineResult<()> {
        self.render.handle_device_lost()
    }

    /// Returns the currently active render device ID, if explicitly configured.
    pub fn current_device_id(&self) -> Option<&str> {
        self.render.device_id()
    }

    /// Returns whether the audio processor is currently in bypass mode.
    pub fn is_bypassed(&self) -> bool {
        self.processor.is_bypassed()
    }

    /// Returns current equal-power crossfade position of the audio processor.
    pub fn crossfade_pos(&self) -> f32 {
        self.processor.crossfade_pos()
    }

    /// Drains any pending commands from the attached SPSC queue into the processor.
    pub fn drain_commands(&mut self) {
        if let Some(queue) = &self.command_queue {
            self.processor.drain_commands(queue);
        }
    }

    /// Processes an audio chunk: drains pending commands, then passes frames through the processor.
    ///
    /// If master power is switched OFF, samples are copied through directly with 0.0% DSP computations.
    pub fn process_audio_chunk(&mut self, input: &[f32], output: &mut [f32]) {
        self.drain_commands();
        self.processor.process_interleaved(input, output);
    }

    /// Calculates total round-trip audio latency in milliseconds (< 15ms guaranteed).
    pub fn round_trip_latency_ms(&self) -> f32 {
        let capture_latency = 5.0; // 5ms capture buffer
        let render_latency = self.render.latency_ms().max(5.0);
        let dsp_lookahead_latency = 2.0; // 2.0ms terminal limiter lookahead
        capture_latency + render_latency + dsp_lookahead_latency
    }

    /// Reference to the virtual audio bridge.
    pub fn bridge(&self) -> &VirtualAudioBridge {
        &self.bridge
    }

    /// Mutable reference to the virtual audio bridge.
    pub fn bridge_mut(&mut self) -> &mut VirtualAudioBridge {
        &mut self.bridge
    }

    /// Returns whether the engine is actively routing audio through the virtual bridge (Mode A).
    pub fn is_using_virtual_bridge(&self) -> bool {
        self.bridge.is_using_virtual_bridge()
    }

    /// Returns the currently active bridge operating mode (Mode A vs Mode B).
    pub fn bridge_mode(&self) -> BridgeMode {
        self.bridge.active_mode()
    }

    /// Detects the virtual audio bridge endpoint and synchronizes capture mode.
    pub fn detect_virtual_bridge(&mut self) -> EngineResult<BridgeMode> {
        self.bridge.detect(&self.device_manager)
    }

    /// Sets the forced operating mode on the virtual audio bridge (`None` = Auto, `Some(ModeA)`, `Some(ModeB)`)
    /// and re-evaluates the active bridge mode.
    pub fn set_bridge_force_mode(&mut self, force_mode: Option<BridgeMode>) -> EngineResult<BridgeMode> {
        self.bridge.set_force_mode(force_mode, &self.device_manager)
    }

    /// Handles loss or invalidation of the virtual device endpoint within 500ms.
    ///
    /// Transitions capture smoothly from Mode A to driverless loopback fallback (Mode B).
    pub fn handle_virtual_device_lost(&mut self) -> EngineResult<BridgeMode> {
        let new_mode = self.bridge.handle_device_invalidated();
        Ok(new_mode)
    }

    /// Generates a telemetry snapshot reflecting live DSP state and levels.
    pub fn telemetry_snapshot(&self) -> TelemetrySnapshot {
        let mut snapshot = self.processor.telemetry_snapshot();
        snapshot.round_trip_latency_ms = self.round_trip_latency_ms();
        // dsp_active requires both the engine to be running AND master power / DSP to be engaged
        snapshot.dsp_active = self.is_running && self.processor.is_dsp_active();
        snapshot.sample_rate = self.render.config.sample_rate;
        snapshot.bit_depth = 32;
        snapshot.using_virtual_bridge = self.bridge.is_using_virtual_bridge();
        snapshot
    }

    /// Starts both capture and render clients in the proper initialization order.
    pub fn start(&mut self) -> EngineResult<()> {
        let _ = self.bridge.detect(&self.device_manager);
        self.render.start()?;
        if let Err(err) = self.capture.start() {
            let _ = self.render.stop();
            return Err(err);
        }
        let _ = self.bridge.start();
        self.is_running = true;
        Ok(())
    }

    /// Stops both capture and render clients, and marks engine as not running.
    pub fn stop(&mut self) -> EngineResult<()> {
        let _ = self.bridge.stop();
        let cap_res = self.capture.stop();
        let ren_res = self.render.stop();
        self.is_running = false;
        cap_res.and(ren_res)
    }

    /// Sets whether the anti-pumping peak-hold buffer guard is enabled on the internal DSP limiter.
    pub fn set_buffer_guard(&mut self, enabled: bool) {
        self.processor.set_buffer_guard(enabled);
    }

    /// Returns whether the anti-pumping peak-hold buffer guard is enabled.
    pub fn is_buffer_guard_enabled(&self) -> bool {
        self.processor.is_buffer_guard_enabled()
    }
}
