//! High-fidelity sample rate converter using cubic Hermite interpolation.
//!
//! Converts arbitrary sample rates (44.1k, 88.2k, 96k, 192k) to the internal 48 kHz
//! float32 pipeline format with cross-chunk state preservation, zero dynamic allocations,
//! and C1 derivative continuity across buffer boundaries.

use open_boom_core::types::INTERNAL_SAMPLE_RATE;

/// Maximum number of audio channels supported by the resampler.
pub const MAX_RESAMPLER_CHANNELS: usize = 8;

/// Resampler converting arbitrary source sample rates into internal 48 kHz float32 streams.
pub struct Resampler {
    source_rate: u32,
    target_rate: u32,
    channels: usize,
    ratio: f64,
    phase: f64,
    /// Cross-chunk sample history per channel: [ch][0..3] where index 3 is sample at t = -1
    history: [[f32; 4]; MAX_RESAMPLER_CHANNELS],
    has_history: bool,
}

impl Resampler {
    /// Creates a new stereo resampler targeting the standard 48 kHz internal pipeline.
    pub fn new(source_rate: u32) -> Self {
        Self::with_target_rate(source_rate, INTERNAL_SAMPLE_RATE, 2)
    }

    /// Creates a resampler with a specified channel count (up to 8 channels).
    pub fn with_channels(source_rate: u32, channels: usize) -> Self {
        Self::with_target_rate(source_rate, INTERNAL_SAMPLE_RATE, channels)
    }

    /// Creates a resampler with arbitrary source and target rates and channel count.
    pub fn with_target_rate(source_rate: u32, target_rate: u32, channels: usize) -> Self {
        let safe_channels = channels.clamp(1, MAX_RESAMPLER_CHANNELS);
        let ratio = source_rate as f64 / target_rate as f64;
        Self {
            source_rate,
            target_rate,
            channels: safe_channels,
            ratio,
            phase: 0.0,
            history: [[0.0; 4]; MAX_RESAMPLER_CHANNELS],
            has_history: false,
        }
    }

    #[inline(always)]
    pub fn source_rate(&self) -> u32 {
        self.source_rate
    }

    #[inline(always)]
    pub fn target_rate(&self) -> u32 {
        self.target_rate
    }

    #[inline(always)]
    pub fn channels(&self) -> usize {
        self.channels
    }

    #[inline(always)]
    pub fn ratio(&self) -> f64 {
        self.ratio
    }

    /// Returns true if the source rate matches the target rate (48 kHz identity).
    #[inline(always)]
    pub fn is_identity(&self) -> bool {
        self.source_rate == self.target_rate
    }

    /// Calculates the maximum number of output frames that could be generated for a given
    /// input frame count based on the current phase and conversion ratio.
    pub fn required_output_frames(&self, input_frames: usize) -> usize {
        if self.is_identity() {
            input_frames
        } else {
            let span = (input_frames as f64 - self.phase).max(0.0);
            (span / self.ratio).ceil() as usize + 2
        }
    }

    /// Resets resampler phase and inter-chunk history state.
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.has_history = false;
        for ch in 0..MAX_RESAMPLER_CHANNELS {
            self.history[ch] = [0.0; 4];
        }
    }

    /// Resamples an interleaved audio slice into an interleaved output slice.
    ///
    /// Returns the number of output audio *frames* written (each frame has `channels` samples).
    /// Guaranteed zero heap allocations in the active processing loop.
    pub fn process_interleaved(&mut self, input: &[f32], output: &mut [f32]) -> usize {
        let input_frames = input.len() / self.channels;
        let max_output_frames = output.len() / self.channels;

        if input_frames == 0 || max_output_frames == 0 {
            return 0;
        }

        // Identity passthrough optimization (zero resampling overhead)
        if self.is_identity() {
            let frames_to_copy = input_frames.min(max_output_frames);
            let samples_to_copy = frames_to_copy * self.channels;
            output[..samples_to_copy].copy_from_slice(&input[..samples_to_copy]);
            return frames_to_copy;
        }

        let mut out_frame = 0;
        let mut pos = self.phase;
        let eps = 1e-7;

        while (pos + eps) < input_frames as f64 && out_frame < max_output_frames {
            let i = pos.floor() as isize;
            let t = (pos - pos.floor()) as f32;

            for ch in 0..self.channels {
                let p0 = self.get_sample(input, input_frames, i - 1, ch);
                let p1 = self.get_sample(input, input_frames, i, ch);
                let p2 = self.get_sample(input, input_frames, i + 1, ch);
                let p3 = self.get_sample(input, input_frames, i + 2, ch);

                let interpolated = hermite_interpolate(p0, p1, p2, p3, t);
                output[out_frame * self.channels + ch] = interpolated.clamp(-1.0, 1.0);
            }

            out_frame += 1;
            pos += self.ratio;
        }

        // Preserve inter-chunk state across buffer boundaries
        let rem = pos - input_frames as f64;
        self.phase = if rem.abs() < eps { 0.0 } else { rem };

        let n = input_frames;
        for ch in 0..self.channels {
            self.history[ch][0] = self.get_sample(input, n, n as isize - 4, ch);
            self.history[ch][1] = self.get_sample(input, n, n as isize - 3, ch);
            self.history[ch][2] = self.get_sample(input, n, n as isize - 2, ch);
            self.history[ch][3] = self.get_sample(input, n, n as isize - 1, ch);
        }
        self.has_history = true;

        out_frame
    }

    /// Alias for `process_interleaved` matching standard audio pipeline convention.
    #[inline(always)]
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> usize {
        self.process_interleaved(input, output)
    }

    #[inline(always)]
    fn get_sample(&self, input: &[f32], input_frames: usize, frame_idx: isize, ch: usize) -> f32 {
        if frame_idx < 0 {
            if self.has_history {
                match frame_idx {
                    -1 => self.history[ch][3],
                    -2 => self.history[ch][2],
                    -3 => self.history[ch][1],
                    _ => self.history[ch][0],
                }
            } else {
                // Initial chunk padding before any history is available
                input[ch]
            }
        } else if frame_idx < input_frames as isize {
            input[frame_idx as usize * self.channels + ch]
        } else {
            // Smooth linear boundary extrapolation when sampling beyond current chunk tail
            let last_idx = input_frames.saturating_sub(1);
            let s_last = input[last_idx * self.channels + ch];
            if input_frames >= 2 {
                let s_prev = input[(last_idx - 1) * self.channels + ch];
                let delta = s_last - s_prev;
                let over = (frame_idx - last_idx as isize) as f32;
                (s_last + delta * over).clamp(-1.0, 1.0)
            } else {
                s_last
            }
        }
    }
}

/// 4-point cubic Hermite (Catmull-Rom) interpolation with C1 continuity.
#[inline(always)]
fn hermite_interpolate(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let c0 = p1;
    let c1 = 0.5 * (p2 - p0);
    let c2 = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
    let c3 = 0.5 * (p3 - p0) + 1.5 * (p1 - p2);
    ((c3 * t + c2) * t + c1) * t + c0
}
