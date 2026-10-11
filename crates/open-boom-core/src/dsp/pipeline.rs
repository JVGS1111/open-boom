//! Real-time audio DSP pipeline processor and lock-free command dispatcher.
//!
//! Enforces zero heap allocations, zero locks, and bit-perfect zero-latency
//! passthrough when master power is switched OFF.

use crate::dsp::ambience::AmbienceModule;
use crate::dsp::fidelity::FidelityModule;
use crate::dsp::graphic_eq_stage::GraphicEqStage;
use crate::dsp::limiter::MasterLimiter;
use crate::dsp::night_mode::NightModeModule;
use crate::dsp::parametric_stage::ParametricStage;
use crate::dsp::spatial::SpatialWidenerModule;
use crate::dsp::spatializer_3d::Spatializer3D;
use crate::ring_buffer::SpscQueue;
use crate::types::{AudioCommand, TelemetrySnapshot, SURROUND_7_1_CHANNELS};

/// Real-time audio processing pipeline and command handler.
pub struct AudioProcessor {
    master_power: bool,
    master_volume: f32,
    master_boost: f32,
    is_muted: bool,
    is_bypassed: bool,
    crossfade_pos: f32,
    pub spatializer: Spatializer3D,
    pub parametric_eq: ParametricStage,
    pub graphic_eq: GraphicEqStage,
    pub ambience: AmbienceModule,
    pub fidelity: FidelityModule,
    pub night_mode: NightModeModule,
    pub spatial: SpatialWidenerModule,
    limiter: MasterLimiter,
    peak_l: f32,
    peak_r: f32,
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioProcessor {
    /// Creates a new audio processor with master power enabled and nominal unity gain.
    pub fn new() -> Self {
        Self {
            master_power: true,
            master_volume: 1.0,
            master_boost: 1.0,
            is_muted: false,
            is_bypassed: false,
            crossfade_pos: 0.0,
            spatializer: Spatializer3D::new(),
            parametric_eq: ParametricStage::new(),
            graphic_eq: GraphicEqStage::new(),
            ambience: AmbienceModule::new(),
            fidelity: FidelityModule::new(),
            night_mode: NightModeModule::new(),
            spatial: SpatialWidenerModule::new(),
            limiter: MasterLimiter::new(),
            peak_l: 0.0,
            peak_r: 0.0,
        }
    }

    /// Returns whether the master power switch is currently enabled.
    #[inline]
    pub fn master_power(&self) -> bool {
        self.master_power
    }

    /// Sets the master power state directly.
    #[inline]
    pub fn set_master_power(&mut self, enabled: bool) {
        self.master_power = enabled;
    }

    /// Returns whether DSP effects are actively engaged (power ON and not bypassed).
    #[inline]
    pub fn is_dsp_active(&self) -> bool {
        self.master_power && !self.is_bypassed
    }

    /// Returns current master volume (0.0 to 1.0).
    #[inline]
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Returns current software boost factor (1.0 to 1.5).
    #[inline]
    pub fn master_boost(&self) -> f32 {
        self.master_boost
    }

    /// Returns whether the processor is muted.
    #[inline]
    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    /// Returns whether the processor is currently in A/B bypass mode.
    #[inline]
    pub fn is_bypassed(&self) -> bool {
        self.is_bypassed
    }

    /// Returns current equal-power crossfade position (0.0 = fully processed, 1.0 = fully bypassed).
    #[inline]
    pub fn crossfade_pos(&self) -> f32 {
        self.crossfade_pos
    }

    /// Sets the operational sample rate and updates all internal DSP stages.
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        let sr = sample_rate as f32;
        self.parametric_eq.set_sample_rate(sr);
        self.graphic_eq.set_sample_rate(sr);
        self.fidelity.set_sample_rate(sr);
        self.night_mode.set_sample_rate(sr);
        self.spatial.set_sample_rate(sr);
    }

    /// Drains and processes all pending commands from an SPSC lock-free queue.
    ///
    /// Must be invoked by the real-time audio thread before processing each buffer.
    /// Operates with zero heap allocations and zero lock contention.
    #[inline]
    pub fn drain_commands<const CAP: usize>(
        &mut self,
        command_queue: &SpscQueue<AudioCommand, CAP>,
    ) {
        while let Some(command) = command_queue.pop() {
            self.handle_command(command);
        }
    }

    /// Dispatches a single POD audio command.
    #[inline]
    pub fn handle_command(&mut self, command: AudioCommand) {
        match command {
            AudioCommand::SetMasterPower(power) => {
                self.master_power = power;
            }
            AudioCommand::SetMasterVolume { volume, boost } => {
                let safe_vol = if volume.is_nan() || volume.is_infinite() {
                    1.0
                } else {
                    volume.clamp(0.0, 1.0)
                };
                let safe_boost = if boost.is_nan() || boost.is_infinite() {
                    1.0
                } else {
                    boost.clamp(1.0, 1.5)
                };
                self.master_volume = safe_vol;
                self.master_boost = safe_boost;
            }
            AudioCommand::SetMute(muted) => {
                self.is_muted = muted;
            }
            AudioCommand::SetBypass(bypassed) => {
                self.is_bypassed = bypassed;
            }
            AudioCommand::ResetFilters => {
                self.limiter.reset();
                self.spatializer.reset();
                self.parametric_eq.reset();
                self.graphic_eq.reset();
                self.ambience.reset();
                self.fidelity.reset();
                self.night_mode.reset();
                self.spatial.reset();
            }
            AudioCommand::SetSurround3D { enabled, intensity } => {
                self.spatializer.enabled = enabled;
                self.spatializer.set_intensity(intensity);
            }
            AudioCommand::SetSpeakerMute { channel, muted } => {
                self.spatializer.set_speaker_mute(channel as usize, muted);
            }
            AudioCommand::SetSpeakerGain { channel, gain_db } => {
                self.spatializer.set_speaker_gain_db(channel as usize, gain_db);
            }
            AudioCommand::SetSpeakerAzimuth { channel, azimuth_deg } => {
                self.spatializer.set_speaker_azimuth(channel as usize, azimuth_deg);
            }
            AudioCommand::ResetSpeakerPositions => {
                self.spatializer.reset_speaker_positions();
            }
            AudioCommand::LoadGraphicEqSnapshot {
                gains_db,
                preamp_offset_db,
            } => {
                self.graphic_eq.load_snapshot(gains_db, preamp_offset_db);
            }
            AudioCommand::SetEqBandGain { band_idx, gain_db } => {
                self.graphic_eq.set_band_gain(band_idx, gain_db);
            }
            AudioCommand::SetEqGains { gains_db } => {
                self.graphic_eq.set_gains(&gains_db);
            }
            AudioCommand::SetEqBypass { bypassed } => {
                self.graphic_eq.set_bypass(bypassed);
            }
            AudioCommand::SetAutoEqProfile {
                filters,
                filter_count,
                preamp_offset_db,
            } => {
                let count = filter_count.min(crate::types::MAX_PARAMETRIC_FILTERS);
                self.parametric_eq.load_profile(&filters[..count], preamp_offset_db);
                self.parametric_eq.set_enabled(true);
            }
            AudioCommand::SetAutoEqBypass { bypassed } => {
                self.parametric_eq.set_bypass(bypassed);
            }
            AudioCommand::SetSweetener {
                module_id,
                enabled,
                amount,
            } => match module_id {
                crate::types::SWEETENER_AMBIENCE => {
                    self.ambience.enabled = enabled;
                    self.ambience.set_amount(amount);
                }
                crate::types::SWEETENER_FIDELITY => {
                    self.fidelity.enabled = enabled;
                    self.fidelity.set_amount(amount);
                }
                crate::types::SWEETENER_NIGHT_MODE => {
                    self.night_mode.enabled = enabled;
                    self.night_mode.set_amount(amount);
                }
                crate::types::SWEETENER_SPATIAL => {
                    self.spatial.enabled = enabled;
                    self.spatial.set_amount(amount);
                }
                _ => {}
            },
            AudioCommand::SetBufferGuard(enabled) => {
                self.limiter.set_buffer_guard(enabled);
            }
        }
    }

    /// Processes an interleaved stereo audio buffer.
    ///
    /// If `master_power == false`, executes bit-perfect passthrough: copies input
    /// directly to output without any arithmetic or filtering, achieving 0.0% DSP CPU usage.
    #[inline]
    pub fn process_interleaved(&mut self, input: &[f32], output: &mut [f32]) {
        let len = input.len().min(output.len());
        if len == 0 {
            return;
        }
        let in_slice = &input[..len];
        let out_slice = &mut output[..len];

        // Bit-perfect zero-overhead passthrough when master power is switched OFF
        if !self.master_power {
            out_slice.copy_from_slice(in_slice);
            let mut max_l: f32 = 0.0;
            let mut max_r: f32 = 0.0;
            let frames = len / 2;
            for i in 0..frames {
                max_l = max_l.max(out_slice[i * 2].abs());
                max_r = max_r.max(out_slice[i * 2 + 1].abs());
            }
            if !len.is_multiple_of(2) {
                max_l = max_l.max(out_slice[len - 1].abs());
            }
            self.peak_l = max_l;
            self.peak_r = max_r;
            return;
        }

        // Mute overrides all output with silence
        if self.is_muted {
            out_slice.fill(0.0);
            self.peak_l = 0.0;
            self.peak_r = 0.0;
            return;
        }

        let gain = self.master_volume * self.master_boost;
        let mut max_l: f32 = 0.0;
        let mut max_r: f32 = 0.0;

        let frames = len / 2;
        let step = 1.0f32 / crate::types::CROSSFADE_SAMPLES as f32;

        for i in 0..frames {
            let dry_l = in_slice[i * 2];
            let dry_r = in_slice[i * 2 + 1];

            let (spat_l, spat_r) = if self.spatializer.enabled {
                self.spatializer.process_stereo_frame(dry_l, dry_r)
            } else {
                (dry_l, dry_r)
            };

            // Stage 3A: Dedicated Parametric AutoEq Correction Stage (AD-6)
            let (autoeq_l, autoeq_r) = self.parametric_eq.process_stereo(spat_l, spat_r);

            // Stage 3B: 31-Band ISO Graphic Equalizer Bank
            let (eq_l, eq_r) = self.graphic_eq.process_stereo(autoeq_l, autoeq_r);

            // Stage 4: Real-Time Sweeteners Rack & Dynamics Control (Epic 4)
            let (amb_l, amb_r) = self.ambience.process_stereo(eq_l, eq_r);
            let (fid_l, fid_r) = self.fidelity.process_stereo(amb_l, amb_r);
            let (nm_l, nm_r) = self.night_mode.process_stereo(fid_l, fid_r);
            let (sw_l, sw_r) = self.spatial.process_stereo(nm_l, nm_r);

            let proc_l = sw_l * gain;
            let proc_r = sw_r * gain;

            let alpha = self.crossfade_pos;
            let (mix_l, mix_r) = if alpha <= 0.0 {
                (proc_l, proc_r)
            } else if alpha >= 1.0 {
                (dry_l, dry_r)
            } else {
                let (g_b, g_a) = (alpha * std::f32::consts::FRAC_PI_2).sin_cos();
                (proc_l * g_a + dry_l * g_b, proc_r * g_a + dry_r * g_b)
            };

            // Advance crossfade position smoothly towards target
            if self.is_bypassed {
                if self.crossfade_pos < 1.0 {
                    self.crossfade_pos = (self.crossfade_pos + step).min(1.0);
                    if (1.0 - self.crossfade_pos).abs() < 1e-6 {
                        self.crossfade_pos = 1.0;
                    }
                }
            } else if self.crossfade_pos > 0.0 {
                self.crossfade_pos = (self.crossfade_pos - step).max(0.0);
                if self.crossfade_pos < 1e-6 {
                    self.crossfade_pos = 0.0;
                }
            }

            let (out_l, out_r) = self.limiter.process_stereo(mix_l, mix_r);

            out_slice[i * 2] = out_l;
            out_slice[i * 2 + 1] = out_r;

            max_l = max_l.max(out_l.abs());
            max_r = max_r.max(out_r.abs());
        }

        // Handle odd trailing sample directly without advancing stereo limiter circular buffer
        if !len.is_multiple_of(2) {
            let last_idx = len - 1;
            let dry_val = in_slice[last_idx];
            let spat_val = if self.spatializer.enabled {
                let (sl, _) = self.spatializer.process_stereo_frame(dry_val, dry_val);
                sl
            } else {
                dry_val
            };
            let autoeq_val = self.parametric_eq.process_mono_left(spat_val);
            let eq_val = self.graphic_eq.process_mono_left(autoeq_val);
            let (amb_val, _) = self.ambience.process_stereo(eq_val, eq_val);
            let (fid_val, _) = self.fidelity.process_stereo(amb_val, amb_val);
            let (nm_val, _) = self.night_mode.process_stereo(fid_val, fid_val);
            let (sw_val, _) = self.spatial.process_stereo(nm_val, nm_val);
            let proc_val = sw_val * gain;
            let alpha = self.crossfade_pos;
            let mix_val = if alpha <= 0.0 {
                proc_val
            } else if alpha >= 1.0 {
                dry_val
            } else {
                let (g_b, g_a) = (alpha * std::f32::consts::FRAC_PI_2).sin_cos();
                proc_val * g_a + dry_val * g_b
            };
            let clamped = if mix_val.is_finite() {
                mix_val.clamp(-self.limiter.ceiling_linear(), self.limiter.ceiling_linear())
            } else {
                0.0
            };
            out_slice[last_idx] = clamped;
            max_l = max_l.max(clamped.abs());
        }

        self.peak_l = max_l;
        self.peak_r = max_r;
    }

    /// Processes an 8-channel (7.1) frame [FL, FR, C, LFE, RL, RR, SL, SR] and down-folds
    /// to binaural stereo [L, R] through the full DSP chain.
    #[inline]
    pub fn process_surround_frame(&mut self, channels: &[f32; SURROUND_7_1_CHANNELS]) -> (f32, f32) {
        if !self.master_power {
            return (channels[0], channels[1]);
        }
        if self.is_muted {
            return (0.0, 0.0);
        }
        let (spat_l, spat_r) = self.spatializer.process_frame(channels);
        let (autoeq_l, autoeq_r) = self.parametric_eq.process_stereo(spat_l, spat_r);
        let (eq_l, eq_r) = self.graphic_eq.process_stereo(autoeq_l, autoeq_r);
        let (amb_l, amb_r) = self.ambience.process_stereo(eq_l, eq_r);
        let (fid_l, fid_r) = self.fidelity.process_stereo(amb_l, amb_r);
        let (nm_l, nm_r) = self.night_mode.process_stereo(fid_l, fid_r);
        let (sw_l, sw_r) = self.spatial.process_stereo(nm_l, nm_r);
        let gain = self.master_volume * self.master_boost;
        let proc_l = sw_l * gain;
        let proc_r = sw_r * gain;
        self.limiter.process_stereo(proc_l, proc_r)
    }

    /// Generates a real-time telemetry snapshot of engine state and levels.
    pub fn telemetry_snapshot(&self) -> TelemetrySnapshot {
        let peak_l_dbfs = if self.peak_l > 1e-5 {
            20.0 * self.peak_l.log10()
        } else {
            -100.0
        };

        let peak_r_dbfs = if self.peak_r > 1e-5 {
            20.0 * self.peak_r.log10()
        } else {
            -100.0
        };

        let limiter_gr = if self.master_power {
            self.limiter.gain_reduction_db()
        } else {
            0.0
        };

        let peak_out = self.peak_l.max(self.peak_r);
        let headroom_db = if peak_out < 1e-4 {
            20.0
        } else {
            let peak_dbfs = 20.0 * peak_out.log10();
            (-0.2 - peak_dbfs).clamp(0.0, 20.0)
        };

        TelemetrySnapshot {
            limiter_gain_reduction_db: limiter_gr,
            round_trip_latency_ms: 10.0,
            peak_output_l_dbfs: peak_l_dbfs,
            peak_output_r_dbfs: peak_r_dbfs,
            limiter_active: limiter_gr < -0.01,
            dsp_active: self.is_dsp_active(),
            limiter_headroom_db: headroom_db,
            sample_rate: crate::types::INTERNAL_SAMPLE_RATE,
            bit_depth: 32,
            using_virtual_bridge: false,
            surround_intensity: self.spatializer.target_intensity,
            surround_enabled: self.spatializer.enabled,
            autoeq_preamp_db: if self.parametric_eq.is_enabled() {
                self.parametric_eq.preamp_offset_db
            } else {
                0.0
            },
            autoeq_active: self.parametric_eq.is_enabled() && self.parametric_eq.active_count > 0,
        }
    }

    /// Adjust gain for an individual graphic EQ band (0..30) in dB (-12.0 to +12.0 dB).
    #[inline]
    pub fn set_eq_band_gain(&mut self, band_idx: usize, gain_db: f32) {
        self.graphic_eq.set_band_gain(band_idx, gain_db);
    }

    /// Set whether the graphic EQ stage is bypassed.
    #[inline]
    pub fn set_eq_bypass(&mut self, bypassed: bool) {
        self.graphic_eq.set_bypass(bypassed);
    }

    /// Returns whether the graphic EQ stage is bypassed.
    #[inline]
    pub fn is_eq_bypassed(&self) -> bool {
        self.graphic_eq.is_bypassed()
    }

    /// Load 31-band graphic EQ snapshot and preamp offset.
    #[inline]
    pub fn load_graphic_eq_snapshot(&mut self, gains_db: [f32; crate::types::GRAPHIC_EQ_BANDS], preamp_offset_db: f32) {
        self.graphic_eq.load_snapshot(gains_db, preamp_offset_db);
    }

    /// Load parametric AutoEq profile and auto preamp offset.
    #[inline]
    pub fn load_autoeq_profile(&mut self, filters: &[crate::types::ParametricFilterPod], preamp_offset_db: f32) {
        self.parametric_eq.load_profile(filters, preamp_offset_db);
        self.parametric_eq.set_enabled(true);
    }

    /// Set whether the parametric AutoEq stage is bypassed.
    #[inline]
    pub fn set_autoeq_bypass(&mut self, bypassed: bool) {
        self.parametric_eq.set_bypass(bypassed);
    }

    /// Returns whether the parametric AutoEq stage is bypassed.
    #[inline]
    pub fn is_autoeq_bypassed(&self) -> bool {
        self.parametric_eq.is_bypassed()
    }

    /// Sets whether the anti-pumping peak-hold buffer guard is enabled on the master limiter.
    #[inline]
    pub fn set_buffer_guard(&mut self, enabled: bool) {
        self.limiter.set_buffer_guard(enabled);
    }

    /// Returns whether the anti-pumping peak-hold buffer guard is enabled.
    #[inline]
    pub fn is_buffer_guard_enabled(&self) -> bool {
        self.limiter.is_buffer_guard_enabled()
    }
}
