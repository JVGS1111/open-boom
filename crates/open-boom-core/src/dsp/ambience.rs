//! Ambience reverberation processor based on Schroeder-Moorer / FDN architecture.
//!
//! Generates natural room reflections and smooth stereo diffusion with
//! zero heap allocations, smooth parameter interpolation, and strict phase
//! coherence on monaural signals (FR16, AD-4, NFR1, NFR2).

/// Delay line buffer capacities (mutually prime lengths for natural room density).
const COMB_BUF_1: usize = 1117; // ~23.3ms at 48 kHz
const COMB_BUF_2: usize = 1279; // ~26.6ms at 48 kHz
const COMB_BUF_3: usize = 1427; // ~29.7ms at 48 kHz
const COMB_BUF_4: usize = 1583; // ~33.0ms at 48 kHz

const AP_BUF_1: usize = 227;    // ~4.7ms at 48 kHz
const AP_BUF_2: usize = 347;    // ~7.2ms at 48 kHz
const ER_BUF: usize = 384;      // ~8.0ms early reflection buffer

/// Comb filter with 1-pole low-pass damping in the feedback loop.
#[derive(Clone, Copy)]
struct CombFilter<const N: usize> {
    buffer: [f32; N],
    index: usize,
    filter_state: f32,
    feedback: f32,
    damp: f32,
}

impl<const N: usize> CombFilter<N> {
    const fn new(feedback: f32, damp: f32) -> Self {
        Self {
            buffer: [0.0; N],
            index: 0,
            filter_state: 0.0,
            feedback,
            damp,
        }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.index];
        // Low-pass filter in feedback loop simulates high-frequency room absorption
        self.filter_state = (output * (1.0 - self.damp)) + (self.filter_state * self.damp);
        let feedback_sample = input + (self.filter_state * self.feedback);
        self.buffer[self.index] = if feedback_sample.is_finite() {
            feedback_sample
        } else {
            0.0
        };

        self.index += 1;
        if self.index >= N {
            self.index = 0;
        }

        output
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
        self.filter_state = 0.0;
    }
}

/// All-pass filter for smooth stereo phase diffusion.
#[derive(Clone, Copy)]
struct AllPassFilter<const N: usize> {
    buffer: [f32; N],
    index: usize,
    gain: f32,
}

impl<const N: usize> AllPassFilter<N> {
    const fn new(gain: f32) -> Self {
        Self {
            buffer: [0.0; N],
            index: 0,
            gain,
        }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let buf_out = self.buffer[self.index];
        let current = input + (buf_out * self.gain);
        // Canonical Schroeder all-pass: y[n] = -g * x[n] + x[n - D]
        let canonical_out = buf_out - (input * self.gain);
        self.buffer[self.index] = if current.is_finite() {
            current
        } else {
            0.0
        };

        self.index += 1;
        if self.index >= N {
            self.index = 0;
        }

        canonical_out
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
    }
}

/// Ambience reverb module.
pub struct AmbienceModule {
    pub enabled: bool,
    target_amount: f32,
    current_amount: f32,
    // Parallel comb filter bank
    comb_1: CombFilter<COMB_BUF_1>,
    comb_2: CombFilter<COMB_BUF_2>,
    comb_3: CombFilter<COMB_BUF_3>,
    comb_4: CombFilter<COMB_BUF_4>,
    // All-pass diffusion stages for left and right channels
    ap_l1: AllPassFilter<AP_BUF_1>,
    ap_l2: AllPassFilter<AP_BUF_2>,
    ap_r1: AllPassFilter<AP_BUF_1>,
    ap_r2: AllPassFilter<AP_BUF_2>,
    // Early reflections circular buffer
    er_buffer: [f32; ER_BUF],
    er_index: usize,
}

impl Default for AmbienceModule {
    fn default() -> Self {
        Self::new()
    }
}

impl AmbienceModule {
    /// Creates a new Ambience reverberation module with calibrated room decay.
    pub fn new() -> Self {
        Self {
            enabled: false,
            target_amount: 0.35,
            current_amount: 0.0,
            comb_1: CombFilter::new(0.805, 0.25),
            comb_2: CombFilter::new(0.785, 0.25),
            comb_3: CombFilter::new(0.765, 0.25),
            comb_4: CombFilter::new(0.745, 0.25),
            ap_l1: AllPassFilter::new(0.5),
            ap_l2: AllPassFilter::new(0.5),
            ap_r1: AllPassFilter::new(0.5),
            ap_r2: AllPassFilter::new(0.5),
            er_buffer: [0.0; ER_BUF],
            er_index: 0,
        }
    }

    /// Sets the target reverb intensity (0.0 to 1.0).
    #[inline]
    pub fn set_amount(&mut self, amount: f32) {
        let safe = if amount.is_nan() || amount.is_infinite() {
            0.0
        } else {
            amount.clamp(0.0, 1.0)
        };
        self.target_amount = safe;
    }

    /// Returns the current target intensity.
    #[inline]
    pub fn amount(&self) -> f32 {
        self.target_amount
    }

    /// Processes a single stereo frame (left, right) with smooth wet/dry mix.
    #[inline]
    pub fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled && self.current_amount <= 1e-4 {
            return (left, right);
        }

        // Smooth parameter interpolation towards target (sub-buffer ramping ~200 samples)
        let target = if self.enabled { self.target_amount } else { 0.0 };
        const SMOOTH_STEP: f32 = 0.005;
        if (self.current_amount - target).abs() < SMOOTH_STEP {
            self.current_amount = target;
        } else if self.current_amount < target {
            self.current_amount += SMOOTH_STEP;
        } else {
            self.current_amount -= SMOOTH_STEP;
        }

        if self.current_amount <= 1e-4 {
            return (left, right);
        }

        let l_in = if left.is_finite() { left } else { 0.0 };
        let r_in = if right.is_finite() { right } else { 0.0 };

        // Mono input for comb filter bank
        let mono_in = (l_in + r_in) * 0.5;

        // Early reflections from multi-tap delay
        self.er_buffer[self.er_index] = mono_in;
        let tap1_idx = (self.er_index + ER_BUF - 120) % ER_BUF; // ~2.5ms
        let tap2_idx = (self.er_index + ER_BUF - 240) % ER_BUF; // ~5.0ms
        let tap3_idx = (self.er_index + ER_BUF - 360) % ER_BUF; // ~7.5ms
        self.er_index += 1;
        if self.er_index >= ER_BUF {
            self.er_index = 0;
        }

        let er_l = self.er_buffer[tap1_idx] * 0.25 - self.er_buffer[tap3_idx] * 0.15;
        let er_r = self.er_buffer[tap2_idx] * 0.25 + self.er_buffer[tap3_idx] * 0.15;

        // Parallel comb filters
        let c1 = self.comb_1.process(mono_in);
        let c2 = self.comb_2.process(mono_in);
        let c3 = self.comb_3.process(mono_in);
        let c4 = self.comb_4.process(mono_in);

        // Orthogonal stereo decorrelation
        let comb_l = (c1 + c2) - (c3 + c4);
        let comb_r = (c1 - c2) + (c3 - c4);

        // Cascaded all-pass diffusion
        let diff_l = self.ap_l2.process(self.ap_l1.process(comb_l * 0.25));
        let diff_r = self.ap_r2.process(self.ap_r1.process(comb_r * 0.25));

        // Combined wet reverberant tail + early reflections
        let wet_l = diff_l + er_l;
        let wet_r = diff_r + er_r;

        // Equal-power wet/dry crossfade
        let wet_gain = self.current_amount * 0.45;
        let out_l = l_in + (wet_l * wet_gain);
        let out_r = r_in + (wet_r * wet_gain);

        (out_l, out_r)
    }

    /// Flushes all internal delay lines and filter states to 0.0.
    pub fn reset(&mut self) {
        self.comb_1.reset();
        self.comb_2.reset();
        self.comb_3.reset();
        self.comb_4.reset();
        self.ap_l1.reset();
        self.ap_l2.reset();
        self.ap_r1.reset();
        self.ap_r2.reset();
        self.er_buffer.fill(0.0);
        self.er_index = 0;
        self.current_amount = 0.0;
    }
}
