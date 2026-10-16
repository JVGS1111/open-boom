//! CoreAudio audio engine coordinator implementing the hexagonal
//! [`AudioHostBackend`] boundary for macOS.

use std::sync::Arc;
use std::sync::Mutex;

use open_boom_core::dsp::pipeline::AudioProcessor;
use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::traits::AudioHostBackend;
use open_boom_core::types::{
    AudioCommand, AudioDeviceInfo, EngineError, EngineResult, TelemetrySnapshot,
    LIMITER_LOOKAHEAD_SAMPLES,
};

use crate::capture::{CaptureConfig, CoreAudioCapture};
use crate::device::CoreAudioDeviceManager;
use crate::hal::AudioHal;
use crate::render::{CoreAudioRender, RenderConfig};
use crate::stream_pump::{CoreAudioStreamPump, StreamPumpConfig};
use crate::BridgeMode;

/// Default capacity for the SPSC command ring buffer.
pub const DEFAULT_COMMAND_QUEUE_CAPACITY: usize = 256;

/// High-level CoreAudio engine coordinator managing capture, DSP, and render.
pub struct CoreAudioCoordinator {
    capture: CoreAudioCapture,
    render: CoreAudioRender,
    processor: AudioProcessor,
    command_queue: Option<Arc<SpscQueue<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>>>,
    device_manager: CoreAudioDeviceManager,
    is_running: bool,
    uses_virtual_bridge: bool,
    bridge_force_mode: Option<BridgeMode>,
    pump_telemetry: Option<Arc<Mutex<TelemetrySnapshot>>>,
    /// Pump configuration requested through [`Self::create_stream_pump`]; the pump itself
    /// is owned by, and tied to the lifecycle of, this coordinator.
    pump_config: Option<StreamPumpConfig>,
    pump: Option<CoreAudioStreamPump>,
}

impl Default for CoreAudioCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreAudioCoordinator {
    /// Creates a coordinator with default capture/render configuration.
    pub fn new() -> Self {
        Self::with_configs(CaptureConfig::default(), RenderConfig::default())
    }

    /// Creates a coordinator with explicit capture and render configurations.
    pub fn with_configs(capture_config: CaptureConfig, render_config: RenderConfig) -> Self {
        Self::build(capture_config, render_config, None)
    }

    /// Creates a coordinator with an attached SPSC command queue.
    pub fn with_command_queue(
        capture_config: CaptureConfig,
        render_config: RenderConfig,
        command_queue: Arc<SpscQueue<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>>,
    ) -> Self {
        Self::build(capture_config, render_config, Some(command_queue))
    }

    /// Creates a coordinator over an explicitly supplied HAL (headless tests).
    pub fn with_hal(
        hal: Arc<dyn AudioHal>,
        capture_config: CaptureConfig,
        render_config: RenderConfig,
    ) -> Self {
        let device_manager = CoreAudioDeviceManager::with_hal(hal.clone());
        Self {
            capture: CoreAudioCapture::new(hal.clone(), capture_config),
            render: CoreAudioRender::new(hal, render_config),
            processor: AudioProcessor::new(),
            command_queue: None,
            device_manager,
            is_running: false,
            uses_virtual_bridge: false,
            bridge_force_mode: None,
            pump_telemetry: None,
            pump_config: None,
            pump: None,
        }
    }

    /// Creates a forced-synthetic coordinator with a deterministic virtual sink (tests).
    pub fn with_forced_synthetic_and_virtual(force_synthetic: bool, include_virtual: bool) -> Self {
        let device_manager = CoreAudioDeviceManager::with_forced_synthetic_and_virtual(
            force_synthetic,
            include_virtual,
        );
        let hal = device_manager.hal();
        Self {
            capture: CoreAudioCapture::new(hal.clone(), CaptureConfig::default()),
            render: CoreAudioRender::new(hal, RenderConfig::default()),
            processor: AudioProcessor::new(),
            command_queue: None,
            device_manager,
            is_running: false,
            uses_virtual_bridge: include_virtual,
            bridge_force_mode: None,
            pump_telemetry: None,
            pump_config: None,
            pump: None,
        }
    }

    fn build(
        capture_config: CaptureConfig,
        render_config: RenderConfig,
        command_queue: Option<Arc<SpscQueue<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>>>,
    ) -> Self {
        // Either side requesting synthetic capture/render selects the synthetic-capable HAL.
        let force_synthetic = capture_config.force_synthetic || render_config.force_synthetic;
        let allow_synthetic =
            capture_config.allow_synthetic_fallback || render_config.allow_synthetic_fallback;
        let device_manager = if force_synthetic {
            CoreAudioDeviceManager::with_forced_synthetic(true)
        } else if allow_synthetic {
            CoreAudioDeviceManager::with_synthetic_fallback(true)
        } else {
            CoreAudioDeviceManager::new()
        };
        let hal = device_manager.hal();

        Self {
            capture: CoreAudioCapture::new(hal.clone(), capture_config),
            render: CoreAudioRender::new(hal, render_config),
            processor: AudioProcessor::new(),
            command_queue,
            device_manager,
            is_running: false,
            uses_virtual_bridge: false,
            bridge_force_mode: None,
            pump_telemetry: None,
            pump_config: None,
            pump: None,
        }
    }

    /// Reference to the capture client.
    pub fn capture(&self) -> &CoreAudioCapture {
        &self.capture
    }

    /// Mutable reference to the capture client.
    pub fn capture_mut(&mut self) -> &mut CoreAudioCapture {
        &mut self.capture
    }

    /// Reference to the render client.
    pub fn render(&self) -> &CoreAudioRender {
        &self.render
    }

    /// Mutable reference to the render client.
    pub fn render_mut(&mut self) -> &mut CoreAudioRender {
        &mut self.render
    }

    /// Reference to the internal DSP processor.
    pub fn processor(&self) -> &AudioProcessor {
        &self.processor
    }

    /// Mutable reference to the internal DSP processor.
    pub fn processor_mut(&mut self) -> &mut AudioProcessor {
        &mut self.processor
    }

    /// Reference to the device manager.
    pub fn device_manager(&self) -> &CoreAudioDeviceManager {
        &self.device_manager
    }

    /// Mutable reference to the device manager.
    pub fn device_manager_mut(&mut self) -> &mut CoreAudioDeviceManager {
        &mut self.device_manager
    }

    /// Sets whether the anti-pumping peak-hold buffer guard is enabled on the internal DSP limiter.
    pub fn set_buffer_guard(&mut self, enabled: bool) {
        self.processor.set_buffer_guard(enabled);
    }

    /// Returns whether the anti-pumping peak-hold buffer guard is enabled.
    pub fn is_buffer_guard_enabled(&self) -> bool {
        self.processor.is_buffer_guard_enabled()
    }

    /// Detects the virtual audio bridge endpoint and synchronizes capture mode.
    pub fn detect_virtual_bridge(&mut self) -> EngineResult<BridgeMode> {
        if let Some(forced) = self.bridge_force_mode {
            return Ok(forced);
        }
        let has_virtual = self
            .device_manager
            .find_virtual_device()
            .ok()
            .flatten()
            .is_some();
        self.uses_virtual_bridge = has_virtual;
        if has_virtual {
            Ok(BridgeMode::VirtualSinkModeA)
        } else {
            Ok(BridgeMode::LoopbackFallbackModeB)
        }
    }

    /// Sets the forced operating mode on the virtual audio bridge (`None` = Auto, `Some(ModeA)`, `Some(ModeB)`)
    /// and re-evaluates the active bridge mode.
    pub fn set_bridge_force_mode(
        &mut self,
        force_mode: Option<BridgeMode>,
    ) -> EngineResult<BridgeMode> {
        self.bridge_force_mode = force_mode;
        match force_mode {
            Some(mode) => {
                self.uses_virtual_bridge = mode == BridgeMode::VirtualSinkModeA;
                Ok(mode)
            }
            None => self.detect_virtual_bridge(),
        }
    }

    /// Returns the currently configured forced operating mode on the virtual audio bridge, if any.
    pub fn bridge_force_mode(&self) -> Option<BridgeMode> {
        self.bridge_force_mode
    }

    /// Returns whether the virtual bridge is currently in use.
    pub fn is_using_virtual_bridge(&self) -> bool {
        self.uses_virtual_bridge
    }

    /// Returns the currently pinned render device id, or `None` when using the host default.
    pub fn current_device_id(&self) -> Option<String> {
        self.render.device_id().map(|id| id.to_string())
    }

    /// Handles loss or invalidation of the virtual device endpoint within 500ms.
    pub fn handle_virtual_device_lost(&mut self) -> EngineResult<BridgeMode> {
        self.uses_virtual_bridge = false;
        Ok(BridgeMode::LoopbackFallbackModeB)
    }

    /// Reference to the attached SPSC command queue, if any.
    pub fn command_queue(&self) -> Option<Arc<SpscQueue<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>>> {
        self.command_queue.clone()
    }

    /// Attaches an SPSC command queue.
    pub fn set_command_queue(
        &mut self,
        queue: Arc<SpscQueue<AudioCommand, DEFAULT_COMMAND_QUEUE_CAPACITY>>,
    ) {
        self.command_queue = Some(queue);
    }

    /// Returns whether the engine is currently running.
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Returns whether a stream pump has been configured (via [`Self::create_stream_pump`]).
    pub fn is_pump_configured(&self) -> bool {
        self.pump_config.is_some()
    }

    /// Returns whether the owned stream pump is currently running.
    pub fn pump_is_running(&self) -> bool {
        self.pump.as_ref().map(|pump| pump.is_running()).unwrap_or(false)
    }

    /// Returns `(capture_dropped_samples, render_dropped_samples)` for telemetry.
    pub fn dropped_samples(&self) -> (usize, usize) {
        (self.capture.dropped_samples(), self.render.dropped_samples())
    }

    /// Returns whether the owned stream pump detected a stalled capture stream.
    pub fn is_capture_stalled(&self) -> bool {
        self.pump
            .as_ref()
            .map(|pump| pump.is_capture_stalled())
            .unwrap_or(false)
    }

    /// Enumerates active render endpoints.
    pub fn enumerate_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        self.device_manager.enumerate_devices()
    }

    /// Switches the render endpoint, updating the OS default output device.
    ///
    /// If the rebind fails the render client is left stopped and the coordinator stops
    /// reporting itself as running.
    pub fn switch_device(&mut self, device_id: Option<String>) -> EngineResult<()> {
        let result = self.render.switch_device(device_id);
        if self.is_running && !self.render.is_rendering() {
            self.is_running = false;
        }
        result
    }

    /// Restores the system default output endpoint after device loss.
    pub fn handle_device_lost(&mut self) -> EngineResult<()> {
        // Consume the HAL edge we are handling so it is not re-processed later.
        self.device_manager.take_default_output_changed();
        let result = self.render.handle_device_lost();
        if self.is_running && !self.render.is_rendering() {
            self.is_running = false;
        }
        result
    }

    /// Checks the HAL's "default output changed" edge and restores the default endpoint
    /// if it fired. Returns whether a recovery was performed.
    pub fn poll_device_loss(&mut self) -> EngineResult<bool> {
        if self.device_manager.take_default_output_changed() {
            self.render.handle_device_lost()?;
            if self.is_running && !self.render.is_rendering() {
                self.is_running = false;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Consumes the HAL "default output changed" edge without acting on it.
    pub fn take_default_output_changed(&self) -> bool {
        self.device_manager.take_default_output_changed()
    }

    /// Drains any pending SPSC commands into the DSP processor.
    pub fn drain_commands(&mut self) {
        if let Some(queue) = &self.command_queue {
            self.processor.drain_commands(queue);
        }
    }

    /// Processes one interleaved chunk, preserving bit-perfect passthrough when master power is off.
    pub fn process_chunk(&mut self, input: &[f32], output: &mut [f32]) {
        self.process_chunk_impl(input, output);
    }

    fn process_chunk_impl(&mut self, input: &[f32], output: &mut [f32]) {
        self.drain_commands();
        self.processor.process_interleaved(input, output);
    }

    /// Terminal DSP lookahead in milliseconds, on the negotiated stream rate.
    fn dsp_lookahead_ms(&self) -> f32 {
        let rate = self.render.sample_rate().max(1) as f32;
        (LIMITER_LOOKAHEAD_SAMPLES as f32 / rate) * 1000.0
    }

    /// Round-trip latency derived from the configured capture/render buffers plus DSP lookahead.
    ///
    /// All three terms use the negotiated stream rate. This reflects the actual configured
    /// buffers; there is no universal sub-15 ms guarantee, though the documented default
    /// configuration (240-frame buffers at 48 kHz) yields 5 ms + 5 ms + 2 ms = 12 ms.
    pub fn round_trip_latency_ms(&self) -> f32 {
        self.capture.latency_ms() + self.render.latency_ms() + self.dsp_lookahead_ms()
    }

    /// Generates a telemetry snapshot reflecting live DSP state.
    pub fn telemetry_snapshot(&self) -> TelemetrySnapshot {
        let mut snapshot = match &self.pump_telemetry {
            Some(cell) => cell
                .lock()
                .map(|guard| *guard)
                .unwrap_or_else(|_| self.processor.telemetry_snapshot()),
            None => self.processor.telemetry_snapshot(),
        };
        snapshot.round_trip_latency_ms = self.round_trip_latency_ms();
        snapshot.dsp_active = self.is_running && self.processor.is_dsp_active();
        snapshot.sample_rate = self.render.sample_rate();
        snapshot.bit_depth = 32;
        snapshot.using_virtual_bridge = self.uses_virtual_bridge;
        snapshot
    }

    /// Starts the render and capture streams in a healthy order, then the configured pump.
    pub fn start(&mut self) -> EngineResult<()> {
        let capture_rate = self.capture.config().sample_rate;
        let render_rate = self.render.config().sample_rate;
        if capture_rate != render_rate {
            return Err(EngineError::UnsupportedFormat(format!(
                "capture/render sample-rate mismatch: {capture_rate} Hz vs {render_rate} Hz"
            )));
        }

        self.uses_virtual_bridge = match self.bridge_force_mode {
            Some(BridgeMode::VirtualSinkModeA) => true,
            Some(BridgeMode::LoopbackFallbackModeB) => false,
            None => self
                .device_manager
                .find_virtual_device()
                .ok()
                .flatten()
                .is_some(),
        };

        self.render.start()?;
        if let Err(err) = self.capture.start() {
            let _ = self.render.stop();
            return Err(err);
        }

        // Validate the negotiated rates actually agree before declaring the engine live,
        // then adopt the negotiated rate so DSP and telemetry reflect the real stream.
        let negotiated_capture = self.capture.sample_rate();
        let negotiated_render = self.render.sample_rate();
        if negotiated_capture != negotiated_render {
            let _ = self.capture.stop();
            let _ = self.render.stop();
            return Err(EngineError::UnsupportedFormat(format!(
                "negotiated sample-rate mismatch: capture {negotiated_capture} Hz vs render {negotiated_render} Hz"
            )));
        }
        self.processor.set_sample_rate(negotiated_render);

        if let Err(err) = self.spawn_pump() {
            let _ = self.capture.stop();
            let _ = self.render.stop();
            return Err(err);
        }

        self.is_running = true;
        Ok(())
    }

    /// Stops the owned pump, then the capture and render streams, reporting any failure.
    pub fn stop(&mut self) -> EngineResult<()> {
        if let Some(mut pump) = self.pump.take() {
            pump.stop();
        }

        let capture_result = self.capture.stop();
        let render_result = self.render.stop();
        self.is_running = false;

        match (capture_result, render_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(err)) => Err(err),
            (Err(capture_err), Err(render_err)) => Err(EngineError::DeviceInitialization(format!(
                "capture stop failed: {capture_err}; render stop failed: {render_err}"
            ))),
        }
    }

    /// Configures the single, coordinator-owned stream pump.
    ///
    /// Calling this more than once fails rather than starting a second pump that would
    /// compete with the first for the same SPSC rings and command queue.
    pub fn create_stream_pump(&mut self, config: StreamPumpConfig) -> EngineResult<()> {
        if self.pump.is_some() || self.pump_config.is_some() {
            return Err(EngineError::DeviceInitialization(
                "a CoreAudio stream pump is already configured".to_string(),
            ));
        }
        self.pump_config = Some(config);
        if self.is_running {
            self.spawn_pump()?;
        }
        Ok(())
    }

    /// Returns the pump telemetry cell or initializes it.
    pub fn pump_telemetry_cell(&mut self) -> Arc<Mutex<TelemetrySnapshot>> {
        self.pump_telemetry
            .get_or_insert_with(|| Arc::new(Mutex::new(TelemetrySnapshot::default())))
            .clone()
    }

    /// Spawns the configured pump if it is not already running.
    fn spawn_pump(&mut self) -> EngineResult<()> {
        let Some(config) = self.pump_config.clone() else {
            return Ok(());
        };
        if self.pump_is_running() {
            return Ok(());
        }

        let capture_ring = self.capture.ring();
        let render_ring = self.render.ring();
        let sample_rate = self.render.sample_rate();
        let telemetry = self.pump_telemetry_cell();
        let pump = CoreAudioStreamPump::start_with_rings(
            capture_ring,
            render_ring,
            self.command_queue.clone(),
            telemetry,
            sample_rate,
            config,
        )?;
        self.pump = Some(pump);
        Ok(())
    }
}

impl AudioHostBackend for CoreAudioCoordinator {
    fn start(&mut self) -> EngineResult<()> {
        CoreAudioCoordinator::start(self)
    }

    fn stop(&mut self) -> EngineResult<()> {
        CoreAudioCoordinator::stop(self)
    }

    fn is_running(&self) -> bool {
        CoreAudioCoordinator::is_running(self)
    }

    fn process_chunk(&mut self, input: &[f32], output: &mut [f32]) {
        self.process_chunk_impl(input, output);
    }

    fn enumerate_devices(&self) -> EngineResult<Vec<AudioDeviceInfo>> {
        CoreAudioCoordinator::enumerate_devices(self)
    }

    fn switch_device(&mut self, device_id: Option<String>) -> EngineResult<()> {
        CoreAudioCoordinator::switch_device(self, device_id)
    }

    fn round_trip_latency_ms(&self) -> f32 {
        CoreAudioCoordinator::round_trip_latency_ms(self)
    }

    fn current_device_id(&self) -> Option<String> {
        self.render.device_id().map(|id| id.to_string())
    }
}
