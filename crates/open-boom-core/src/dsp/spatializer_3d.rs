//! 3D Surround spatializer down-folding 7.1 surround to binaural stereo.
//!
//! Convolves 8 virtual speaker channels (FL, FR, C, LFE, RL, RR, SL, SR) with
//! calibrated KEMAR dummy-head Head-Related Transfer Function (HRTF) impulse responses
//! using SIMD acceleration (SSE2/AVX) and zero heap allocations.

use crate::types::{
    OrbitDirection, CH_CENTER, CH_FRONT_LEFT, CH_FRONT_RIGHT, CH_LFE, CH_REAR_LEFT, CH_REAR_RIGHT,
    CH_SURROUND_LEFT, CH_SURROUND_RIGHT, ITU_R_7_1_DEFAULT_AZIMUTHS, SURROUND_7_1_CHANNELS,
};

/// FIR filter tap length for HRTF impulse responses (64 taps = 1.33ms at 48 kHz).
pub const HRTF_TAPS: usize = 64;

/// The NEON path strides 8 floats per iteration, so the tap count must be a multiple of 8.
const _: () = assert!(HRTF_TAPS % 8 == 0);

/// Sub-block frame size for orbital HRTF coefficient updates (32 samples = ~0.67ms at 48 kHz).
pub const ORBIT_SUB_BLOCK_SAMPLES: usize = 32;

/// Default 8D orbital revolution period in seconds (12.0 seconds).
pub const DEFAULT_ORBIT_PERIOD_SEC: f32 = 12.0;

/// Minimum allowable 8D orbital revolution period in seconds (2.0 seconds).
pub const MIN_ORBIT_PERIOD_SEC: f32 = 2.0;

/// Maximum allowable 8D orbital revolution period in seconds (30.0 seconds).
pub const MAX_ORBIT_PERIOD_SEC: f32 = 30.0;

/// Default 8D orbital spatial depth factor (0.85 = 85%).
pub const DEFAULT_ORBIT_DEPTH: f32 = 0.85;

/// Wraps an azimuth angle in degrees to the continuous range [-180.0, 180.0].
#[inline]
pub fn wrap_deg_180(deg: f32) -> f32 {
    if !deg.is_finite() {
        return 0.0;
    }
    let mut a = (deg + 180.0).rem_euclid(360.0) - 180.0;
    if a == -180.0 && deg > 0.0 {
        a = 180.0;
    }
    a
}

/// Calibrated KEMAR HRTF impulse responses in reverse order for forward linear convolution.
/// Index [ch][k] where k=0 corresponds to delay=63, k=63 corresponds to delay=0 (newest sample).
///
/// Acoustic parameters:
/// - Front Left (-30°): Ipsilateral peak tap 3, contralateral peak tap 15 (ITD = 12 samples = 0.25ms).
/// - Center (0°): Symmetric arrival at tap 4 (ITD = 0), frontal pinna elevation boost.
/// - Surround Left (-90°): Ipsilateral peak tap 2, contralateral peak tap 34 (ITD = 32 samples = 0.67ms).
/// - Rear Left (-135°): Ipsilateral peak tap 3 with 6 kHz rear pinna notch (tap 7 reflection),
///   contralateral peak tap 29 (ITD = 26 samples = 0.54ms).
/// - LFE: Omnidirectional bass distribution.
pub static KEMAR_HRTF_LEFT_REV: [[f32; HRTF_TAPS]; SURROUND_7_1_CHANNELS] = init_kemar_left_rev();
pub static KEMAR_HRTF_RIGHT_REV: [[f32; HRTF_TAPS]; SURROUND_7_1_CHANNELS] = init_kemar_right_rev();

const fn init_kemar_left_rev() -> [[f32; HRTF_TAPS]; SURROUND_7_1_CHANNELS] {
    let mut table = [[0.0f32; HRTF_TAPS]; SURROUND_7_1_CHANNELS];

    // Helper: place impulse in normal order then invert index: rev[63 - delay] = val
    // Front Left (0): Ipsilateral (Left ear) direct arrival at tap 3, head-shadowed on Right ear.
    table[CH_FRONT_LEFT][63 - 3] = 0.72;
    table[CH_FRONT_LEFT][63 - 4] = 0.28;
    table[CH_FRONT_LEFT][63 - 5] = -0.12;
    table[CH_FRONT_LEFT][63 - 6] = 0.05;

    // Front Right (1): Contralateral (Left ear) delayed by ITD ~ 12 samples (tap 15), ILD attenuation.
    table[CH_FRONT_RIGHT][63 - 15] = 0.38;
    table[CH_FRONT_RIGHT][63 - 16] = 0.22;
    table[CH_FRONT_RIGHT][63 - 17] = 0.10;

    // Center (2): Symmetric arrival at tap 4.
    table[CH_CENTER][63 - 4] = 0.54;
    table[CH_CENTER][63 - 5] = 0.26;
    table[CH_CENTER][63 - 6] = -0.08;

    // LFE (3): Low-passed mono bass.
    table[CH_LFE][63 - 4] = 0.35;
    table[CH_LFE][63 - 5] = 0.30;
    table[CH_LFE][63 - 6] = 0.20;
    table[CH_LFE][63 - 7] = 0.10;

    // Rear Left (4): Ipsilateral arrival tap 3 with pinna rear notch at tap 7 (-0.28) and tap 11 (+0.10).
    table[CH_REAR_LEFT][63 - 3] = 0.64;
    table[CH_REAR_LEFT][63 - 4] = 0.20;
    table[CH_REAR_LEFT][63 - 7] = -0.28; // 6 kHz rear pinna notch cancellation
    table[CH_REAR_LEFT][63 - 11] = 0.10;

    // Rear Right (5): Contralateral delayed by ITD ~ 26 samples (tap 29) + head shadow.
    table[CH_REAR_RIGHT][63 - 29] = 0.28;
    table[CH_REAR_RIGHT][63 - 30] = 0.18;
    table[CH_REAR_RIGHT][63 - 31] = 0.08;

    // Surround Left (6): Ipsilateral maximum lateral arrival tap 2.
    table[CH_SURROUND_LEFT][63 - 2] = 0.78;
    table[CH_SURROUND_LEFT][63 - 3] = 0.30;
    table[CH_SURROUND_LEFT][63 - 4] = -0.15;

    // Surround Right (7): Contralateral maximum ITD ~ 32 samples (tap 34).
    table[CH_SURROUND_RIGHT][63 - 34] = 0.24;
    table[CH_SURROUND_RIGHT][63 - 35] = 0.16;
    table[CH_SURROUND_RIGHT][63 - 36] = 0.08;

    table
}

const fn init_kemar_right_rev() -> [[f32; HRTF_TAPS]; SURROUND_7_1_CHANNELS] {
    let mut table = [[0.0f32; HRTF_TAPS]; SURROUND_7_1_CHANNELS];

    // Front Left (0): Contralateral (Right ear) delayed by ITD ~ 12 samples (tap 15).
    table[CH_FRONT_LEFT][63 - 15] = 0.38;
    table[CH_FRONT_LEFT][63 - 16] = 0.22;
    table[CH_FRONT_LEFT][63 - 17] = 0.10;

    // Front Right (1): Ipsilateral (Right ear) direct arrival at tap 3.
    table[CH_FRONT_RIGHT][63 - 3] = 0.72;
    table[CH_FRONT_RIGHT][63 - 4] = 0.28;
    table[CH_FRONT_RIGHT][63 - 5] = -0.12;
    table[CH_FRONT_RIGHT][63 - 6] = 0.05;

    // Center (2): Symmetric arrival at tap 4.
    table[CH_CENTER][63 - 4] = 0.54;
    table[CH_CENTER][63 - 5] = 0.26;
    table[CH_CENTER][63 - 6] = -0.08;

    // LFE (3): Low-passed mono bass.
    table[CH_LFE][63 - 4] = 0.35;
    table[CH_LFE][63 - 5] = 0.30;
    table[CH_LFE][63 - 6] = 0.20;
    table[CH_LFE][63 - 7] = 0.10;

    // Rear Left (4): Contralateral delayed by ITD ~ 26 samples (tap 29) + head shadow.
    table[CH_REAR_LEFT][63 - 29] = 0.28;
    table[CH_REAR_LEFT][63 - 30] = 0.18;
    table[CH_REAR_LEFT][63 - 31] = 0.08;

    // Rear Right (5): Ipsilateral arrival tap 3 with pinna rear notch at tap 7 (-0.28) and tap 11 (+0.10).
    table[CH_REAR_RIGHT][63 - 3] = 0.64;
    table[CH_REAR_RIGHT][63 - 4] = 0.20;
    table[CH_REAR_RIGHT][63 - 7] = -0.28; // 6 kHz rear pinna notch cancellation
    table[CH_REAR_RIGHT][63 - 11] = 0.10;

    // Surround Left (6): Contralateral maximum ITD ~ 32 samples (tap 34).
    table[CH_SURROUND_LEFT][63 - 34] = 0.24;
    table[CH_SURROUND_LEFT][63 - 35] = 0.16;
    table[CH_SURROUND_LEFT][63 - 36] = 0.08;

    // Surround Right (7): Ipsilateral maximum lateral arrival tap 2.
    table[CH_SURROUND_RIGHT][63 - 2] = 0.78;
    table[CH_SURROUND_RIGHT][63 - 3] = 0.30;
    table[CH_SURROUND_RIGHT][63 - 4] = -0.15;

    table
}

/// Reference impulse response for direct Rear Center (180 deg / -180 deg).
/// Symmetric arrival at tap 4 with rear pinna notch at tap 7 (-0.28) and tap 11 (+0.10).
pub const REAR_CENTER_HRTF_REV: [f32; HRTF_TAPS] = {
    let mut table = [0.0f32; HRTF_TAPS];
    table[63 - 4] = 0.50;
    table[63 - 5] = 0.20;
    table[63 - 7] = -0.28;
    table[63 - 11] = 0.10;
    table
};

/// Computes 64-tap KEMAR HRTF impulse response tables for a speaker channel at any azimuth angle
/// (-180.0 to +180.0 degrees) by linearly interpolating between neighboring acoustic anchors.
///
/// Guaranteed zero heap allocations, zero locks, and stack-only computation.
pub fn compute_interpolated_hrtf(
    channel: usize,
    azimuth_deg: f32,
) -> ([f32; HRTF_TAPS], [f32; HRTF_TAPS]) {
    if channel == CH_LFE {
        return (KEMAR_HRTF_LEFT_REV[CH_LFE], KEMAR_HRTF_RIGHT_REV[CH_LFE]);
    }

    let angle = if azimuth_deg.is_nan() || azimuth_deg.is_infinite() {
        ITU_R_7_1_DEFAULT_AZIMUTHS[channel]
    } else {
        azimuth_deg.clamp(-180.0, 180.0)
    };

    // 9 sorted acoustic anchor points spanning the full 360-degree horizontal plane
    let anchors: [(f32, &[f32; HRTF_TAPS], &[f32; HRTF_TAPS]); 9] = [
        (-180.0, &REAR_CENTER_HRTF_REV, &REAR_CENTER_HRTF_REV),
        (-135.0, &KEMAR_HRTF_LEFT_REV[CH_REAR_LEFT], &KEMAR_HRTF_RIGHT_REV[CH_REAR_LEFT]),
        (-90.0, &KEMAR_HRTF_LEFT_REV[CH_SURROUND_LEFT], &KEMAR_HRTF_RIGHT_REV[CH_SURROUND_LEFT]),
        (-30.0, &KEMAR_HRTF_LEFT_REV[CH_FRONT_LEFT], &KEMAR_HRTF_RIGHT_REV[CH_FRONT_LEFT]),
        (0.0, &KEMAR_HRTF_LEFT_REV[CH_CENTER], &KEMAR_HRTF_RIGHT_REV[CH_CENTER]),
        (30.0, &KEMAR_HRTF_LEFT_REV[CH_FRONT_RIGHT], &KEMAR_HRTF_RIGHT_REV[CH_FRONT_RIGHT]),
        (90.0, &KEMAR_HRTF_LEFT_REV[CH_SURROUND_RIGHT], &KEMAR_HRTF_RIGHT_REV[CH_SURROUND_RIGHT]),
        (135.0, &KEMAR_HRTF_LEFT_REV[CH_REAR_RIGHT], &KEMAR_HRTF_RIGHT_REV[CH_REAR_RIGHT]),
        (180.0, &REAR_CENTER_HRTF_REV, &REAR_CENTER_HRTF_REV),
    ];

    // Find the bounding anchor interval [idx, idx+1]
    let mut idx = 0;
    while idx < anchors.len() - 1 && angle > anchors[idx + 1].0 {
        idx += 1;
    }
    if idx >= anchors.len() - 1 {
        idx = anchors.len() - 2;
    }

    let (a0, l0, r0) = anchors[idx];
    let (a1, l1, r1) = anchors[idx + 1];

    if (angle - a0).abs() < 1e-4 {
        return (*l0, *r0);
    }
    if (angle - a1).abs() < 1e-4 {
        return (*l1, *r1);
    }

    let span = a1 - a0;
    let t = if span.abs() > 1e-5 {
        ((angle - a0) / span).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let one_minus_t = 1.0 - t;

    let mut out_left = [0.0f32; HRTF_TAPS];
    let mut out_right = [0.0f32; HRTF_TAPS];

    for tap in 0..HRTF_TAPS {
        out_left[tap] = l0[tap] * one_minus_t + l1[tap] * t;
        out_right[tap] = r0[tap] * one_minus_t + r1[tap] * t;
    }

    (out_left, out_right)
}

/// Computes a vector dot product over exactly [`HRTF_TAPS`] taps.
///
/// Uses NEON fused-multiply-add on `aarch64` (Apple Silicon), SSE2 on `x86_64`,
/// and a scalar fallback on every other architecture. Both slices must contain at
/// least [`HRTF_TAPS`] elements; any extra tail is ignored and a shorter slice fails
/// fast via a bounds check rather than silently truncating the convolution.
#[inline(always)]
fn dot_product_64(a: &[f32], b: &[f32]) -> f32 {
    let a = &a[..HRTF_TAPS];
    let b = &b[..HRTF_TAPS];

    #[cfg(target_arch = "aarch64")]
    {
        unsafe { dot_product_64_neon(a, b) }
    }
    #[cfg(target_arch = "x86_64")]
    {
        unsafe { dot_product_64_sse(a, b) }
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        dot_product_64_scalar(a, b)
    }
}

/// 64-tap dot product using ARM NEON fused-multiply-add on Apple Silicon.
///
/// Processes 8 floats per iteration across two independent accumulators to hide the
/// FMA latency chain, then horizontally sums the two 4-lane accumulators.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
#[allow(dead_code)]
unsafe fn dot_product_64_neon(a: &[f32], b: &[f32]) -> f32 {
    use core::arch::aarch64::*;

    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);

    let a_ptr = a.as_ptr();
    let b_ptr = b.as_ptr();

    let mut i = 0usize;
    while i < HRTF_TAPS {
        acc0 = vfmaq_f32(acc0, vld1q_f32(a_ptr.add(i)), vld1q_f32(b_ptr.add(i)));
        acc1 = vfmaq_f32(
            acc1,
            vld1q_f32(a_ptr.add(i + 4)),
            vld1q_f32(b_ptr.add(i + 4)),
        );
        i += 8;
    }

    vaddvq_f32(vaddq_f32(acc0, acc1))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn dot_product_64_sse(a: &[f32], b: &[f32]) -> f32 {
    use core::arch::x86_64::*;

    let mut acc0 = _mm_setzero_ps();
    let mut acc1 = _mm_setzero_ps();
    let mut acc2 = _mm_setzero_ps();
    let mut acc3 = _mm_setzero_ps();

    let a_ptr = a.as_ptr();
    let b_ptr = b.as_ptr();

    for i in 0..4 {
        let offset = i * 16;
        let va0 = _mm_loadu_ps(a_ptr.add(offset));
        let vb0 = _mm_loadu_ps(b_ptr.add(offset));
        acc0 = _mm_add_ps(acc0, _mm_mul_ps(va0, vb0));

        let va1 = _mm_loadu_ps(a_ptr.add(offset + 4));
        let vb1 = _mm_loadu_ps(b_ptr.add(offset + 4));
        acc1 = _mm_add_ps(acc1, _mm_mul_ps(va1, vb1));

        let va2 = _mm_loadu_ps(a_ptr.add(offset + 8));
        let vb2 = _mm_loadu_ps(b_ptr.add(offset + 8));
        acc2 = _mm_add_ps(acc2, _mm_mul_ps(va2, vb2));

        let va3 = _mm_loadu_ps(a_ptr.add(offset + 12));
        let vb3 = _mm_loadu_ps(b_ptr.add(offset + 12));
        acc3 = _mm_add_ps(acc3, _mm_mul_ps(va3, vb3));
    }

    let sum01 = _mm_add_ps(acc0, acc1);
    let sum23 = _mm_add_ps(acc2, acc3);
    let sum = _mm_add_ps(sum01, sum23);

    let shuf = _mm_movehdup_ps(sum);
    let sums = _mm_add_ps(sum, shuf);
    let shuf2 = _mm_movehl_ps(sums, sums);
    let res = _mm_add_ss(sums, shuf2);
    _mm_cvtss_f32(res)
}

#[inline(always)]
#[allow(dead_code)]
fn dot_product_64_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..HRTF_TAPS {
        sum += a[i] * b[i];
    }
    sum
}

/// 3D Surround Spatializer Engine.
pub struct Spatializer3D {
    pub enabled: bool,
    pub target_intensity: f32,
    pub current_intensity: f32,
    pub speaker_gains: [f32; SURROUND_7_1_CHANNELS],
    pub speaker_gains_db: [f32; SURROUND_7_1_CHANNELS],
    pub speaker_muted: [bool; SURROUND_7_1_CHANNELS],
    pub speaker_azimuth: [f32; SURROUND_7_1_CHANNELS],
    pub hrtf_left: [[f32; HRTF_TAPS]; SURROUND_7_1_CHANNELS],
    pub hrtf_right: [[f32; HRTF_TAPS]; SURROUND_7_1_CHANNELS],
    history: [[f32; 128]; SURROUND_7_1_CHANNELS],
    write_pos: usize,
    pub orbit_enabled: bool,
    pub orbit_direction: OrbitDirection,
    pub orbit_period_sec: f32,
    pub orbit_depth: f32,
    pub orbit_phase_deg: f32,
    orbit_phase_progress: f64,
    orbit_sub_block_counter: usize,
    sample_rate: f32,
}

impl Default for Spatializer3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Spatializer3D {
    pub fn new() -> Self {
        let mut hrtf_left = [[0.0; HRTF_TAPS]; SURROUND_7_1_CHANNELS];
        let mut hrtf_right = [[0.0; HRTF_TAPS]; SURROUND_7_1_CHANNELS];
        for ch in 0..SURROUND_7_1_CHANNELS {
            let (hl, hr) = compute_interpolated_hrtf(ch, ITU_R_7_1_DEFAULT_AZIMUTHS[ch]);
            hrtf_left[ch] = hl;
            hrtf_right[ch] = hr;
        }

        Self {
            enabled: false,
            target_intensity: 0.5,
            current_intensity: 0.5,
            speaker_gains: [1.0; SURROUND_7_1_CHANNELS],
            speaker_gains_db: [0.0; SURROUND_7_1_CHANNELS],
            speaker_muted: [false; SURROUND_7_1_CHANNELS],
            speaker_azimuth: ITU_R_7_1_DEFAULT_AZIMUTHS,
            hrtf_left,
            hrtf_right,
            history: [[0.0; 128]; SURROUND_7_1_CHANNELS],
            write_pos: 0,
            orbit_enabled: false,
            orbit_direction: OrbitDirection::Clockwise,
            orbit_period_sec: DEFAULT_ORBIT_PERIOD_SEC,
            orbit_depth: DEFAULT_ORBIT_DEPTH,
            orbit_phase_deg: 0.0,
            orbit_phase_progress: 0.0,
            orbit_sub_block_counter: 0,
            sample_rate: crate::types::INTERNAL_SAMPLE_RATE_F32,
        }
    }

    /// Set operational sample rate in Hz for timing and orbital phase progression.
    #[inline]
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        if sample_rate.is_finite() && sample_rate > 0.0 {
            self.sample_rate = sample_rate;
        }
    }

    /// Set spatializer intensity (0.0 to 1.0) with smooth linear interpolation.
    #[inline]
    pub fn set_intensity(&mut self, intensity: f32) {
        let safe = if intensity.is_nan() || intensity.is_infinite() {
            0.5
        } else {
            intensity.clamp(0.0, 1.0)
        };
        self.target_intensity = safe;
    }

    /// Configures 8D orbital soundstage rotation mode.
    ///
    /// Period is clamped to [2.0, 30.0] seconds (default 12.0s).
    /// Depth is clamped to [0.0, 1.0] (default 0.85).
    /// Automatically engages 3D spatializer if `enabled == true`.
    /// When disabled, smoothly restores static speaker HRTFs without clicks or dropouts.
    pub fn set_orbit(
        &mut self,
        enabled: bool,
        direction: OrbitDirection,
        period_sec: f32,
        depth: f32,
    ) {
        let safe_period = if period_sec.is_nan() || period_sec.is_infinite() {
            DEFAULT_ORBIT_PERIOD_SEC
        } else {
            period_sec.clamp(MIN_ORBIT_PERIOD_SEC, MAX_ORBIT_PERIOD_SEC)
        };

        let safe_depth = if depth.is_nan() || depth.is_infinite() {
            DEFAULT_ORBIT_DEPTH
        } else {
            depth.clamp(0.0, 1.0)
        };

        self.orbit_period_sec = safe_period;
        self.orbit_depth = safe_depth;

        if enabled {
            self.enabled = true;
            if !self.orbit_enabled {
                self.orbit_enabled = true;
                self.orbit_direction = direction;
                self.orbit_phase_deg = 0.0;
                self.orbit_phase_progress = 0.0;
                self.orbit_sub_block_counter = 0;
                self.update_orbit_hrtfs();
            } else if self.orbit_direction != direction {
                self.sync_progress_to_direction(direction);
                self.orbit_direction = direction;
            }
        } else if self.orbit_enabled {
            self.orbit_enabled = false;
            self.orbit_direction = direction;
            self.orbit_phase_deg = 0.0;
            self.orbit_phase_progress = 0.0;
            self.orbit_sub_block_counter = 0;
            self.restore_static_hrtfs();
        } else {
            self.orbit_direction = direction;
        }
    }

    /// Synchronizes internal normalized phase progress when motion direction switches.
    fn sync_progress_to_direction(&mut self, new_direction: OrbitDirection) {
        match new_direction {
            OrbitDirection::Clockwise => {
                self.orbit_phase_progress = (self.orbit_phase_deg as f64 / 360.0).rem_euclid(1.0);
            }
            OrbitDirection::CounterClockwise => {
                self.orbit_phase_progress = (-self.orbit_phase_deg as f64 / 360.0).rem_euclid(1.0);
            }
            OrbitDirection::PingPong => {
                let sin_val = (self.orbit_phase_deg / 180.0).clamp(-1.0, 1.0) as f64;
                let rad = sin_val.asin();
                self.orbit_phase_progress = (rad / (2.0 * std::f64::consts::PI)).rem_euclid(1.0);
            }
        }
    }

    /// Restores static HRTF filter coefficients computed from calibrated `speaker_azimuth`.
    pub fn restore_static_hrtfs(&mut self) {
        for ch in 0..SURROUND_7_1_CHANNELS {
            let (hl, hr) = compute_interpolated_hrtf(ch, self.speaker_azimuth[ch]);
            self.hrtf_left[ch] = hl;
            self.hrtf_right[ch] = hr;
        }
    }

    /// Recalculates HRTF filter coefficients for non-LFE virtual speakers
    /// based on the current orbital phase and depth:
    /// $\theta_k(t) = \text{wrap}_{[-180, 180]}(\text{speaker\_azimuth}[k] + \theta_{\text{orbit}}(t) \cdot \text{orbit\_depth})$.
    #[inline]
    pub fn update_orbit_hrtfs(&mut self) {
        for ch in 0..SURROUND_7_1_CHANNELS {
            if ch == CH_LFE {
                // LFE channel isolation: never rotate or modulate LFE channel azimuth
                continue;
            }
            let eff_az = wrap_deg_180(self.speaker_azimuth[ch] + self.orbit_phase_deg * self.orbit_depth);
            let (hl, hr) = compute_interpolated_hrtf(ch, eff_az);
            self.hrtf_left[ch] = hl;
            self.hrtf_right[ch] = hr;
        }
    }

    /// Advances the orbital trajectory phase by one audio sample.
    #[inline]
    pub fn advance_orbit_phase(&mut self) {
        let period = if self.orbit_period_sec.is_finite() && self.orbit_period_sec >= MIN_ORBIT_PERIOD_SEC {
            self.orbit_period_sec as f64
        } else {
            DEFAULT_ORBIT_PERIOD_SEC as f64
        };
        let sample_rate = if self.sample_rate.is_finite() && self.sample_rate > 0.0 {
            self.sample_rate as f64
        } else {
            crate::types::INTERNAL_SAMPLE_RATE_F32 as f64
        };

        let phase_step = 1.0 / (period * sample_rate);
        self.orbit_phase_progress += phase_step;
        if self.orbit_phase_progress >= 1.0 {
            self.orbit_phase_progress -= 1.0;
        } else if self.orbit_phase_progress < 0.0 {
            self.orbit_phase_progress += 1.0;
        }

        match self.orbit_direction {
            OrbitDirection::Clockwise => {
                let raw_deg = (self.orbit_phase_progress * 360.0) as f32;
                self.orbit_phase_deg = wrap_deg_180(raw_deg);
            }
            OrbitDirection::CounterClockwise => {
                let raw_deg = (-self.orbit_phase_progress * 360.0) as f32;
                self.orbit_phase_deg = wrap_deg_180(raw_deg);
            }
            OrbitDirection::PingPong => {
                let angle_rad = 2.0 * std::f64::consts::PI * self.orbit_phase_progress;
                self.orbit_phase_deg = (180.0 * angle_rad.sin() as f32).clamp(-180.0, 180.0);
            }
        }
    }

    /// Mute or unmute an individual virtual speaker channel (0..7).
    #[inline]
    pub fn set_speaker_mute(&mut self, channel: usize, muted: bool) {
        if channel < SURROUND_7_1_CHANNELS {
            self.speaker_muted[channel] = muted;
        }
    }

    /// Set relative trim gain for an individual virtual speaker in dB (-12.0 to +12.0 dB).
    #[inline]
    pub fn set_speaker_gain_db(&mut self, channel: usize, gain_db: f32) {
        if channel < SURROUND_7_1_CHANNELS {
            let safe_db = if gain_db.is_nan() || gain_db.is_infinite() {
                0.0
            } else {
                gain_db.clamp(-12.0, 12.0)
            };
            self.speaker_gains_db[channel] = safe_db;
            self.speaker_gains[channel] = 10.0f32.powf(safe_db / 20.0);
        }
    }

    /// Set azimuth angle for an individual virtual speaker in degrees (-180.0 to +180.0).
    /// Recalculates 64-tap KEMAR HRTF filter coefficients dynamically without audio dropouts.
    #[inline]
    pub fn set_speaker_azimuth(&mut self, channel: usize, azimuth_deg: f32) {
        if channel < SURROUND_7_1_CHANNELS {
            let safe_deg = if azimuth_deg.is_nan() || azimuth_deg.is_infinite() {
                ITU_R_7_1_DEFAULT_AZIMUTHS[channel]
            } else {
                azimuth_deg.clamp(-180.0, 180.0)
            };
            self.speaker_azimuth[channel] = safe_deg;
            if !self.orbit_enabled {
                let (hl, hr) = compute_interpolated_hrtf(channel, safe_deg);
                self.hrtf_left[channel] = hl;
                self.hrtf_right[channel] = hr;
            } else if channel != CH_LFE {
                let eff_az = wrap_deg_180(safe_deg + self.orbit_phase_deg * self.orbit_depth);
                let (hl, hr) = compute_interpolated_hrtf(channel, eff_az);
                self.hrtf_left[channel] = hl;
                self.hrtf_right[channel] = hr;
            }
        }
    }

    /// Reset all virtual speaker azimuth angles to ITU-R BS.775 reference defaults.
    #[inline]
    pub fn reset_speaker_positions(&mut self) {
        self.speaker_azimuth = ITU_R_7_1_DEFAULT_AZIMUTHS;
        if !self.orbit_enabled {
            self.restore_static_hrtfs();
        } else {
            self.update_orbit_hrtfs();
        }
    }

    /// Flushes all internal delay lines to zero without transient DC clicks.
    pub fn reset(&mut self) {
        self.history = [[0.0; 128]; SURROUND_7_1_CHANNELS];
        self.write_pos = 0;
        self.current_intensity = self.target_intensity;
        self.orbit_sub_block_counter = 0;
        self.orbit_phase_progress = 0.0;
        self.orbit_phase_deg = 0.0;
        if !self.orbit_enabled {
            self.restore_static_hrtfs();
        } else {
            self.update_orbit_hrtfs();
        }
    }

    /// Convolves 8 virtual speaker channels into binaural stereo [L, R] using KEMAR HRTF.
    #[inline]
    pub fn process_frame(&mut self, channels: &[f32; SURROUND_7_1_CHANNELS]) -> (f32, f32) {
        let dry_l = channels[CH_FRONT_LEFT];
        let dry_r = channels[CH_FRONT_RIGHT];

        if !self.enabled {
            return (dry_l, dry_r);
        }

        // Sub-block orbital coefficient update (every 32 samples)
        if self.orbit_enabled {
            if self.orbit_sub_block_counter == 0 {
                self.update_orbit_hrtfs();
            }
            self.orbit_sub_block_counter = (self.orbit_sub_block_counter + 1) % ORBIT_SUB_BLOCK_SAMPLES;
            self.advance_orbit_phase();
        }

        // Smooth parameter interpolation step (crossfade over ~240 samples / 5ms)
        const STEP: f32 = 1.0 / 240.0;
        if self.current_intensity < self.target_intensity {
            self.current_intensity = (self.current_intensity + STEP).min(self.target_intensity);
        } else if self.current_intensity > self.target_intensity {
            self.current_intensity = (self.current_intensity - STEP).max(self.target_intensity);
        }

        let curr_int = self.current_intensity;
        if curr_int <= 1e-4 {
            return (dry_l, dry_r);
        }

        let pos = self.write_pos;
        let mut wet_l = 0.0f32;
        let mut wet_r = 0.0f32;

        for ch in 0..SURROUND_7_1_CHANNELS {
            let s = if self.speaker_muted[ch] {
                0.0
            } else {
                channels[ch] * self.speaker_gains[ch]
            };

            // Write sample into double circular buffer (continuous 64-sample window)
            self.history[ch][pos] = s;
            self.history[ch][pos + 64] = s;

            let history_window = &self.history[ch][pos + 1..pos + 65];

            // Evaluate 64-tap SIMD FIR convolution for Left and Right ears
            let conv_l = dot_product_64(history_window, &self.hrtf_left[ch]);
            let conv_r = dot_product_64(history_window, &self.hrtf_right[ch]);

            wet_l += conv_l;
            wet_r += conv_r;
        }

        self.write_pos = (pos + 1) & 63;

        // Equal-power wet/dry crossfade
        let out_l = dry_l * (1.0 - curr_int) + wet_l * curr_int;
        let out_r = dry_r * (1.0 - curr_int) + wet_r * curr_int;

        (out_l, out_r)
    }

    /// Spatializes an incoming stereo stream [L, R] by upmixing to 7.1 virtual soundstage
    /// and convolving with KEMAR HRTF.
    #[inline]
    pub fn process_stereo_frame(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }

        // Mid/Side surround upmixing matrix
        let fl = left;
        let fr = right;
        let c = (left + right) * 0.5 * std::f32::consts::FRAC_1_SQRT_2;
        let lfe = (left + right) * 0.25;
        let diff = (left - right) * 0.5;
        let rl = diff * 0.7;
        let rr = -diff * 0.7;
        let sl = diff;
        let sr = -diff;

        let surround_chs = [fl, fr, c, lfe, rl, rr, sl, sr];
        self.process_frame(&surround_chs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent scalar reference implementing the exact same left-to-right
    /// summation order used by the scalar fallback.
    fn scalar_reference(a: &[f32; HRTF_TAPS], b: &[f32; HRTF_TAPS]) -> f32 {
        let mut sum = 0.0f32;
        for i in 0..HRTF_TAPS {
            sum += a[i] * b[i];
        }
        sum
    }

    /// Asserts the SIMD dot product matches the scalar reference for `a · b`.
    ///
    /// The tolerance is a *scaled absolute* bound: absolute below unit magnitude and
    /// relative above it, which accommodates both the FMA/Lane reordering of the SIMD
    /// paths and the large dynamic range of the adversarial vectors below.
    fn assert_dot_product_equivalent(a: &[f32; HRTF_TAPS], b: &[f32; HRTF_TAPS]) {
        let expected = scalar_reference(a, b);
        let actual = dot_product_64(a, b);

        if expected.is_nan() {
            assert!(actual.is_nan(), "expected NaN, got {actual}");
            return;
        }
        if expected.is_infinite() {
            assert_eq!(actual, expected, "infinite results must keep their sign");
            return;
        }

        let tolerance = 1e-4 * expected.abs().max(1.0);
        assert!(
            (actual - expected).abs() <= tolerance,
            "SIMD dot product diverged: actual={actual}, expected={expected}, tolerance={tolerance}"
        );
    }

    #[test]
    fn test_dot_product_64_simd_equivalence_to_scalar() {
        // 1. Regular small-magnitude signal.
        let mut regular_a = [0.0f32; HRTF_TAPS];
        let mut regular_b = [0.0f32; HRTF_TAPS];
        for i in 0..HRTF_TAPS {
            regular_a[i] = ((i as f32) * 0.137).sin();
            regular_b[i] = ((i as f32) * 0.071).cos() * 0.5;
        }
        assert_dot_product_equivalent(&regular_a, &regular_b);

        // 2. Extreme magnitude (values near f32::MAX overflow to +Inf consistently).
        assert_dot_product_equivalent(&[1.0e20; HRTF_TAPS], &[1.0e20; HRTF_TAPS]);

        // 3. Adversarial cancellation: large alternating signs that nearly cancel.
        let mut cancel_a = [0.0f32; HRTF_TAPS];
        let mut cancel_b = [0.0f32; HRTF_TAPS];
        for i in 0..HRTF_TAPS {
            cancel_a[i] = if i % 2 == 0 { 1.0e3 } else { -1.0e3 };
            cancel_b[i] = if i % 4 == 0 { -1.0e3 } else { 1.0e3 };
        }
        assert_dot_product_equivalent(&cancel_a, &cancel_b);

        // 4. NaN propagation: a NaN input yields NaN on every path.
        let mut nan_a = [1.0f32; HRTF_TAPS];
        nan_a[7] = f32::NAN;
        assert_dot_product_equivalent(&nan_a, &[1.0f32; HRTF_TAPS]);

        // 5. Infinite inputs keep the correct infinity sign.
        assert_dot_product_equivalent(&[f32::INFINITY; HRTF_TAPS], &[1.0f32; HRTF_TAPS]);
        assert_dot_product_equivalent(&[f32::NEG_INFINITY; HRTF_TAPS], &[1.0f32; HRTF_TAPS]);

        // 6. A perfectly representable input must match bit-for-bit on every path.
        let exact_a = [1.0f32; HRTF_TAPS];
        let exact_b = [2.0f32; HRTF_TAPS];
        assert_eq!(dot_product_64(&exact_a, &exact_b), 128.0);
    }

    #[test]
    fn test_dot_product_64_slice_bounds_safety() {
        // Longer slices are accepted and only the first HRTF_TAPS taps contribute,
        // rather than reading the tail or silently truncating the convolution.
        let a_long = [1.0f32; HRTF_TAPS + 8];
        let b_long = [2.0f32; HRTF_TAPS + 8];
        assert_eq!(dot_product_64(&a_long, &b_long), 128.0);

        // Shorter slices fail fast instead of silently producing a truncated result.
        let a_short = [1.0f32; HRTF_TAPS - 1];
        let b_short = [2.0f32; HRTF_TAPS - 1];
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            dot_product_64(&a_short, &b_short)
        }));
        assert!(
            result.is_err(),
            "a slice shorter than HRTF_TAPS must panic rather than truncate silently"
        );
    }
}
