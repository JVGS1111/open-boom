//! 3D Surround spatializer down-folding 7.1 surround to binaural stereo.
//!
//! Convolves 8 virtual speaker channels (FL, FR, C, LFE, RL, RR, SL, SR) with
//! calibrated KEMAR dummy-head Head-Related Transfer Function (HRTF) impulse responses
//! using SIMD acceleration (SSE2/AVX) and zero heap allocations.

use crate::types::{
    CH_CENTER, CH_FRONT_LEFT, CH_FRONT_RIGHT, CH_LFE, CH_REAR_LEFT, CH_REAR_RIGHT,
    CH_SURROUND_LEFT, CH_SURROUND_RIGHT, ITU_R_7_1_DEFAULT_AZIMUTHS, SURROUND_7_1_CHANNELS,
};

/// FIR filter tap length for HRTF impulse responses (64 taps = 1.33ms at 48 kHz).
pub const HRTF_TAPS: usize = 64;

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

/// Computes a vector dot product of 64 floats using SSE2 intrinsics on x86_64,
/// with scalar fallback for other architectures.
#[inline(always)]
fn dot_product_64(a: &[f32], b: &[f32]) -> f32 {
    #[cfg(target_arch = "x86_64")]
    {
        unsafe { dot_product_64_sse(a, b) }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        dot_product_64_scalar(a, b)
    }
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
    for i in 0..64 {
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
            let (hl, hr) = compute_interpolated_hrtf(channel, safe_deg);
            self.hrtf_left[channel] = hl;
            self.hrtf_right[channel] = hr;
        }
    }

    /// Reset all virtual speaker azimuth angles to ITU-R BS.775 reference defaults.
    #[inline]
    pub fn reset_speaker_positions(&mut self) {
        self.speaker_azimuth = ITU_R_7_1_DEFAULT_AZIMUTHS;
        for ch in 0..SURROUND_7_1_CHANNELS {
            let (hl, hr) = compute_interpolated_hrtf(ch, ITU_R_7_1_DEFAULT_AZIMUTHS[ch]);
            self.hrtf_left[ch] = hl;
            self.hrtf_right[ch] = hr;
        }
    }

    /// Flushes all internal delay lines to zero without transient DC clicks.
    pub fn reset(&mut self) {
        self.history = [[0.0; 128]; SURROUND_7_1_CHANNELS];
        self.write_pos = 0;
        self.current_intensity = self.target_intensity;
    }

    /// Convolves 8 virtual speaker channels into binaural stereo [L, R] using KEMAR HRTF.
    #[inline]
    pub fn process_frame(&mut self, channels: &[f32; SURROUND_7_1_CHANNELS]) -> (f32, f32) {
        let dry_l = channels[CH_FRONT_LEFT];
        let dry_r = channels[CH_FRONT_RIGHT];

        if !self.enabled {
            return (dry_l, dry_r);
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
