//! Second-order IIR Biquad filter (Direct Form II Transposed topology).
//!
//! Implements Robert Bristow-Johnson (RBJ) Audio EQ Cookbook second-order peaking biquad
//! filters, pole stability validation, linear coefficient interpolation (lerp), and
//! Direct Form II Transposed sample processing.

use std::f32::consts::PI;

/// Normalized biquad filter coefficients (where a0 = 1.0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiquadCoefficients {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl Default for BiquadCoefficients {
    fn default() -> Self {
        Self::identity()
    }
}

impl BiquadCoefficients {
    /// Passthrough identity filter with unity gain ($b_0 = 1.0$, all other coefficients $0.0$).
    pub const fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    /// Computes peaking EQ biquad coefficients according to Robert Bristow-Johnson
    /// Audio EQ Cookbook formulas.
    ///
    /// - $A = 10^{\text{gain\_db} / 40}$
    /// - $\omega_0 = 2\pi f_0 / f_s$
    /// - $\alpha = \sin(\omega_0) / (2 Q)$
    /// - $b_0 = 1 + \alpha A$, $b_1 = -2 \cos(\omega_0)$, $b_2 = 1 - \alpha A$
    /// - $a_0 = 1 + \alpha / A$, $a_1 = -2 \cos(\omega_0)$, $a_2 = 1 - \alpha / A$
    ///
    /// Normalized with $a_0 = 1.0$.
    pub fn peaking_eq(sample_rate: f32, freq: f32, q: f32, gain_db: f32) -> Self {
        let safe_sr = if sample_rate.is_finite() && sample_rate >= 8000.0 && sample_rate <= 384_000.0 {
            sample_rate
        } else {
            48_000.0
        };

        let safe_q = if q.is_finite() && q > 0.05 {
            q
        } else {
            4.31
        };

        let safe_gain = if gain_db.is_nan() {
            0.0
        } else {
            gain_db.clamp(-12.0, 12.0)
        };

        let max_freq = safe_sr * 0.499;
        let safe_freq = if freq.is_finite() {
            freq.clamp(1.0, max_freq)
        } else {
            1_000.0f32.min(max_freq)
        };

        let w0 = 2.0 * PI * safe_freq / safe_sr;
        let (sin_w0, cos_w0) = w0.sin_cos();
        let alpha = sin_w0 / (2.0 * safe_q);

        if safe_gain.abs() < 1e-6 {
            let a0 = 1.0 + alpha;
            let inv_a0 = 1.0 / a0;
            let a1 = (-2.0 * cos_w0) * inv_a0;
            let a2 = (1.0 - alpha) * inv_a0;
            return Self {
                b0: 1.0,
                b1: a1,
                b2: a2,
                a1,
                a2,
            };
        }

        let a_linear = 10.0f32.powf(safe_gain / 40.0);
        let b0 = 1.0 + alpha * a_linear;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a_linear;
        let a0 = 1.0 + alpha / a_linear;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a_linear;

        let inv_a0 = 1.0 / a0;
        Self {
            b0: b0 * inv_a0,
            b1: b1 * inv_a0,
            b2: b2 * inv_a0,
            a1: a1 * inv_a0,
            a2: a2 * inv_a0,
        }
    }

    /// Checks if the filter poles remain strictly within the unit circle ($|z| < 1.0$)
    /// using the Schur-Cohn / Jury stability conditions for $z^2 + a_1 z + a_2 = 0$:
    /// 1. $|a_2| < 1.0$
    /// 2. $|a_1| < 1.0 + a_2$
    #[inline]
    pub fn is_stable(&self) -> bool {
        self.a2.abs() < 1.0 && self.a1.abs() < (1.0 + self.a2)
    }

    /// Calculates the maximum pole modulus (distance from complex origin).
    /// Returns a value strictly $< 1.0$ for stable filters.
    pub fn max_pole_modulus(&self) -> f32 {
        let disc = self.a1 * self.a1 - 4.0 * self.a2;
        if disc <= 0.0 {
            self.a2.abs().sqrt()
        } else {
            let s = disc.sqrt();
            let p1 = ((-self.a1 + s) * 0.5).abs();
            let p2 = ((-self.a1 - s) * 0.5).abs();
            p1.max(p2)
        }
    }

    /// Linearly interpolates (lerp) towards `target` coefficients by factor `t` in [0.0, 1.0].
    /// Guarantees that the resulting filter remains stable.
    pub fn lerp(&self, target: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let interpolated = Self {
            b0: self.b0 + (target.b0 - self.b0) * t,
            b1: self.b1 + (target.b1 - self.b1) * t,
            b2: self.b2 + (target.b2 - self.b2) * t,
            a1: self.a1 + (target.a1 - self.a1) * t,
            a2: self.a2 + (target.a2 - self.a2) * t,
        };

        if interpolated.is_stable() {
            interpolated
        } else {
            *target
        }
    }

    /// Computes the theoretical magnitude response in decibels (dB) at the specified frequency.
    pub fn frequency_response_db(&self, freq: f32, sample_rate: f32) -> f32 {
        let safe_sr = if sample_rate.is_finite() && sample_rate > 0.0 { sample_rate } else { 48_000.0 };
        let w = 2.0 * PI * freq / safe_sr;
        let (sin_w, cos_w) = w.sin_cos();
        let (sin_2w, cos_2w) = (2.0 * w).sin_cos();

        let num_re = self.b0 + self.b1 * cos_w + self.b2 * cos_2w;
        let num_im = -self.b1 * sin_w - self.b2 * sin_2w;
        let den_re = 1.0 + self.a1 * cos_w + self.a2 * cos_2w;
        let den_im = -self.a1 * sin_w - self.a2 * sin_2w;

        let num_mag_sq = num_re * num_re + num_im * num_im;
        let den_mag_sq = den_re * den_re + den_im * den_im;

        if den_mag_sq <= 1e-12 || num_mag_sq <= 1e-12 {
            -120.0
        } else {
            10.0 * (num_mag_sq / den_mag_sq).log10()
        }
    }
}

/// Single-channel biquad filter with Direct Form II Transposed topology.
#[derive(Debug, Clone, Copy, Default)]
pub struct Biquad {
    pub coeffs: BiquadCoefficients,
    s1: f32,
    s2: f32,
}

impl Biquad {
    /// Creates a new biquad filter with the specified coefficients and zeroed delay lines.
    pub fn new(coeffs: BiquadCoefficients) -> Self {
        Self {
            coeffs,
            s1: 0.0,
            s2: 0.0,
        }
    }

    /// Creates a new peaking EQ biquad filter.
    pub fn peaking_eq(sample_rate: f32, freq: f32, q: f32, gain_db: f32) -> Self {
        Self::new(BiquadCoefficients::peaking_eq(sample_rate, freq, q, gain_db))
    }

    /// Returns the internal DF-II Transposed state delay line values (s1, s2).
    #[inline]
    pub fn state(&self) -> (f32, f32) {
        (self.s1, self.s2)
    }

    /// Process a single audio sample using Direct Form II Transposed.
    ///
    /// $$y[n] = b_0 x[n] + s_1[n-1]$$
    /// $$s_1[n] = b_1 x[n] - a_1 y[n] + s_2[n-1]$$
    /// $$s_2[n] = b_2 x[n] - a_2 y[n]$$
    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let x = if input.is_finite() { input } else { 0.0 };
        let output = self.coeffs.b0 * x + self.s1;
        self.s1 = self.coeffs.b1 * x - self.coeffs.a1 * output + self.s2;
        self.s2 = self.coeffs.b2 * x - self.coeffs.a2 * output;

        // Anti-denormal flushing & NaN poison healing
        if !self.s1.is_finite() || self.s1.abs() < 1e-25 {
            self.s1 = 0.0;
        }
        if !self.s2.is_finite() || self.s2.abs() < 1e-25 {
            self.s2 = 0.0;
        }

        if output.is_finite() { output } else { 0.0 }
    }

    /// Reset filter memory delay lines to zero.
    #[inline]
    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
}
