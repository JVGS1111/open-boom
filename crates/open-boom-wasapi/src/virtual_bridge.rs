//! Virtual Audio Device Bridge managing dual-mode capture detection, pairing,
//! and automatic fallback between Mode A (Virtual Sink) and Mode B (Driverless Loopback).

use open_boom_core::types::{AudioDeviceInfo, EngineResult};
use crate::device::AudioDeviceManager;
use std::time::Instant;

/// Active audio bridge operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeMode {
    /// Mode A: Ingests from a PortCls/SYSVAD virtual audio sink (e.g. boomvad, VB-Audio Cable, VAC).
    /// Eliminates acoustic slapback echo and unlocks multi-channel (up to 7.1) ingestion.
    VirtualSinkModeA,
    /// Mode B: Driverless WASAPI Process Loopback fallback with process exclusion.
    LoopbackFallbackModeB,
}

/// Configuration options for the virtual audio bridge.
#[derive(Debug, Clone)]
pub struct VirtualBridgeConfig {
    /// Forced mode override (for testing or explicit user configuration).
    pub force_mode: Option<BridgeMode>,
    /// Whether synthetic fallback is permitted in headless/CI environments.
    pub allow_synthetic_fallback: bool,
    /// Simulated virtual device endpoint ID for unit testing.
    pub simulated_virtual_endpoint: Option<String>,
}

impl Default for VirtualBridgeConfig {
    fn default() -> Self {
        Self {
            force_mode: None,
            allow_synthetic_fallback: true,
            simulated_virtual_endpoint: None,
        }
    }
}

/// Virtual audio bridge managing dual-mode capture detection and pairing.
#[derive(Debug)]
pub struct VirtualAudioBridge {
    config: VirtualBridgeConfig,
    active_mode: BridgeMode,
    virtual_device: Option<AudioDeviceInfo>,
    is_active: bool,
    last_switch_time: Option<Instant>,
}

impl Default for VirtualAudioBridge {
    fn default() -> Self {
        Self::new(VirtualBridgeConfig::default())
    }
}

impl VirtualAudioBridge {
    /// Creates a new virtual audio bridge with the specified configuration.
    pub fn new(config: VirtualBridgeConfig) -> Self {
        let initial_mode = config.force_mode.unwrap_or(BridgeMode::LoopbackFallbackModeB);
        Self {
            config,
            active_mode: initial_mode,
            virtual_device: None,
            is_active: false,
            last_switch_time: None,
        }
    }

    /// Detects whether a compatible virtual audio device is present on the system.
    ///
    /// If found (Mode A), configures virtual sink capture with zero acoustic echo.
    /// If absent, automatically falls back to standalone WASAPI loopback (Mode B) without errors.
    pub fn detect(&mut self, device_manager: &AudioDeviceManager) -> EngineResult<BridgeMode> {
        let start_time = Instant::now();

        if let Some(forced) = self.config.force_mode {
            if forced == BridgeMode::VirtualSinkModeA {
                if let Some(ref sim_id) = self.config.simulated_virtual_endpoint {
                    self.virtual_device = Some(AudioDeviceInfo::new(
                        sim_id.clone(),
                        "Simulated Virtual Audio Cable",
                        false,
                    ));
                    self.active_mode = BridgeMode::VirtualSinkModeA;
                } else if let Ok(Some(real_dev)) = device_manager.find_virtual_device() {
                    self.virtual_device = Some(real_dev);
                    self.active_mode = BridgeMode::VirtualSinkModeA;
                } else if self.config.allow_synthetic_fallback {
                    self.virtual_device = Some(AudioDeviceManager::synthetic_virtual_device());
                    self.active_mode = BridgeMode::VirtualSinkModeA;
                } else {
                    self.virtual_device = None;
                    self.active_mode = BridgeMode::LoopbackFallbackModeB;
                }
            } else {
                self.virtual_device = None;
                self.active_mode = BridgeMode::LoopbackFallbackModeB;
            }
            self.last_switch_time = Some(start_time);
            return Ok(self.active_mode);
        }

        if let Some(ref sim_id) = self.config.simulated_virtual_endpoint {
            let sim_dev = AudioDeviceInfo::new(
                sim_id.clone(),
                "Simulated Virtual Audio Cable",
                false,
            );
            self.virtual_device = Some(sim_dev);
            self.active_mode = BridgeMode::VirtualSinkModeA;
            self.last_switch_time = Some(start_time);
            return Ok(BridgeMode::VirtualSinkModeA);
        }

        match device_manager.find_virtual_device() {
            Ok(Some(dev)) => {
                self.virtual_device = Some(dev);
                self.active_mode = BridgeMode::VirtualSinkModeA;
                self.last_switch_time = Some(start_time);
                Ok(BridgeMode::VirtualSinkModeA)
            }
            Ok(None) => {
                self.virtual_device = None;
                self.active_mode = BridgeMode::LoopbackFallbackModeB;
                self.last_switch_time = Some(start_time);
                Ok(BridgeMode::LoopbackFallbackModeB)
            }
            Err(err) => {
                if self.config.allow_synthetic_fallback {
                    self.virtual_device = None;
                    self.active_mode = BridgeMode::LoopbackFallbackModeB;
                    self.last_switch_time = Some(start_time);
                    Ok(BridgeMode::LoopbackFallbackModeB)
                } else {
                    Err(err)
                }
            }
        }
    }

    /// Updates the forced bridge operating mode (`None` for Auto, `Some(VirtualSinkModeA)`, or `Some(LoopbackFallbackModeB)`)
    /// and re-runs detection immediately.
    pub fn set_force_mode(
        &mut self,
        force_mode: Option<BridgeMode>,
        device_manager: &AudioDeviceManager,
    ) -> EngineResult<BridgeMode> {
        self.config.force_mode = force_mode;
        self.virtual_device = None;
        self.detect(device_manager)
    }

    /// Returns the currently configured forced mode override, if any.
    pub fn force_mode(&self) -> Option<BridgeMode> {
        self.config.force_mode
    }

    /// Returns the currently active bridge operating mode (Mode A vs Mode B).
    pub fn active_mode(&self) -> BridgeMode {
        self.active_mode
    }

    /// Returns whether the engine is actively capturing through a virtual audio device bridge (Mode A).
    pub fn is_using_virtual_bridge(&self) -> bool {
        self.active_mode == BridgeMode::VirtualSinkModeA
    }

    /// Returns a reference to the active virtual audio endpoint, if paired.
    pub fn virtual_device(&self) -> Option<&AudioDeviceInfo> {
        self.virtual_device.as_ref()
    }

    /// Explicitly switches the bridge to driverless loopback fallback (Mode B).
    pub fn switch_to_loopback_fallback(&mut self) -> BridgeMode {
        self.active_mode = BridgeMode::LoopbackFallbackModeB;
        self.virtual_device = None;
        self.last_switch_time = Some(Instant::now());
        BridgeMode::LoopbackFallbackModeB
    }

    /// Handles runtime virtual device invalidation (`AUDCLNT_E_DEVICE_INVALIDATED`).
    ///
    /// Transitions from Mode A to driverless loopback fallback (Mode B) within 500ms.
    pub fn handle_device_invalidated(&mut self) -> BridgeMode {
        self.switch_to_loopback_fallback()
    }

    /// Starts the virtual audio bridge lifecycle.
    pub fn start(&mut self) -> EngineResult<()> {
        self.is_active = true;
        Ok(())
    }

    /// Stops the virtual audio bridge.
    pub fn stop(&mut self) -> EngineResult<()> {
        self.is_active = false;
        Ok(())
    }

    /// Returns whether the virtual audio bridge is active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}
