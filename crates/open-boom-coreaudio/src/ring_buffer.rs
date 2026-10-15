//! Lock-free Single-Producer Single-Consumer Float32 sample ring.
//!
//! Bridges the CoreAudio HAL IOProc callbacks (which run on real-time CoreAudio
//! threads) and the OpenBoom stream pump. The ring operations themselves are
//! lock-free and allocation-free; the headless mock's per-call scratch buffers may
//! reallocate. The capture IOProc is the producer; the pump is the consumer. The
//! render ring has the pump as producer and the render IOProc as consumer.
//!
//! Use [`SampleRing::push_frames`] / [`SampleRing::pop_frames`] on the audio path:
//! they move whole interleaved frames only, so a partial write can never permanently
//! shift the L/R interleave, and they record how many samples were dropped when the
//! ring was full.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

/// A lock-free SPSC ring of interleaved Float32 samples.
///
/// Capacity must be a non-zero power of two so the modular index can be a cheap
/// bit mask. Sample payloads are stored as raw bits in [`AtomicU32`] slots, which
/// keeps the structure free of `unsafe` while preserving lock-free semantics.
pub struct SampleRing {
    buffer: Box<[AtomicU32]>,
    mask: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
    dropped_samples: AtomicUsize,
}

impl SampleRing {
    /// Creates a new sample ring with the given capacity (rounded up to a power of two).
    pub fn new(capacity: usize) -> Arc<Self> {
        let capacity = capacity.max(1).next_power_of_two();
        let buffer: Vec<AtomicU32> = (0..capacity).map(|_| AtomicU32::new(0)).collect();
        Arc::new(Self {
            buffer: buffer.into_boxed_slice(),
            mask: capacity - 1,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            dropped_samples: AtomicUsize::new(0),
        })
    }

    /// Total sample capacity of the ring.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Number of samples currently available to the consumer.
    pub fn available(&self) -> usize {
        self.head
            .load(Ordering::Acquire)
            .wrapping_sub(self.tail.load(Ordering::Acquire))
    }

    /// Number of samples the producer can still write.
    pub fn free_space(&self) -> usize {
        self.capacity().saturating_sub(self.available())
    }

    /// Writes up to `samples.len()` samples, returning how many were actually written.
    ///
    /// Called exclusively by the producer (capture IOProc or stream pump).
    pub fn push_samples(&self, samples: &[f32]) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let free = self.capacity().saturating_sub(head.wrapping_sub(tail));
        let count = samples.len().min(free);

        for (i, sample) in samples.iter().take(count).enumerate() {
            self.buffer[(head + i) & self.mask].store(sample.to_bits(), Ordering::Relaxed);
        }

        self.head.store(head.wrapping_add(count), Ordering::Release);
        count
    }

    /// Reads up to `out.len()` samples, returning how many were actually read.
    ///
    /// Unwritten tail entries of `out` are left untouched. Called exclusively by the
    /// consumer (stream pump or render IOProc).
    pub fn pop_samples(&self, out: &mut [f32]) -> usize {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let available = head.wrapping_sub(tail);
        let count = out.len().min(available);

        for (i, slot) in out.iter_mut().take(count).enumerate() {
            *slot = f32::from_bits(self.buffer[(tail + i) & self.mask].load(Ordering::Relaxed));
        }

        self.tail.store(tail.wrapping_add(count), Ordering::Release);
        count
    }

    /// Discards all buffered samples.
    pub fn clear(&self) {
        let head = self.head.load(Ordering::Acquire);
        self.tail.store(head, Ordering::Release);
        self.dropped_samples.store(0, Ordering::Relaxed);
    }

    /// Number of samples dropped because the ring was full since the last [`Self::clear`].
    pub fn dropped_samples(&self) -> usize {
        self.dropped_samples.load(Ordering::Relaxed)
    }

    /// Pushes whole interleaved frames, never splitting a frame across the head.
    ///
    /// Returns the number of frames written. Samples that do not fit (in whole frames)
    /// are counted in [`Self::dropped_samples`]. The sub-frame tail of `samples` is
    /// ignored, matching CoreAudio's frame-granular buffer sizes.
    pub fn push_frames(&self, samples: &[f32], channels: usize) -> usize {
        if channels == 0 {
            return 0;
        }
        let frames = samples.len() / channels;
        let writable = frames.min(self.free_space() / channels);
        self.dropped_samples
            .fetch_add((frames - writable) * channels, Ordering::Relaxed);
        self.push_samples(&samples[..writable * channels]) / channels
    }

    /// Pops whole interleaved frames, never splitting a frame across the tail.
    ///
    /// Returns the number of frames read; any unwritten tail of `out` is left untouched.
    pub fn pop_frames(&self, out: &mut [f32], channels: usize) -> usize {
        if channels == 0 {
            return 0;
        }
        let frames = out.len() / channels;
        let readable = frames.min(self.available() / channels);
        self.pop_samples(&mut out[..readable * channels]) / channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop_fifo_roundtrip() {
        let ring = SampleRing::new(8);
        assert_eq!(ring.available(), 0);

        let written = ring.push_samples(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(written, 4);
        assert_eq!(ring.available(), 4);

        let mut out = [0.0f32; 4];
        let read = ring.pop_samples(&mut out);
        assert_eq!(read, 4);
        assert_eq!(out, [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(ring.available(), 0);
    }

    #[test]
    fn test_overflow_drops_tail_and_preserves_order() {
        let ring = SampleRing::new(4);
        let written = ring.push_samples(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(written, 4, "overflow beyond capacity must be dropped");

        let mut out = [0.0f32; 8];
        let read = ring.pop_samples(&mut out);
        assert_eq!(read, 4);
        assert_eq!(&out[..4], &[1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_partial_read_leaves_tail_untouched() {
        let ring = SampleRing::new(8);
        ring.push_samples(&[1.0, 2.0]);

        let mut out = [9.0f32; 4];
        let read = ring.pop_samples(&mut out);
        assert_eq!(read, 2);
        assert_eq!(out, [1.0, 2.0, 9.0, 9.0], "unwritten tail must stay untouched");
    }

    #[test]
    fn test_wraparound_many_cycles() {
        let ring = SampleRing::new(4);
        for cycle in 0..1000 {
            let base = cycle as f32 * 4.0;
            assert_eq!(ring.push_samples(&[base, base + 1.0, base + 2.0, base + 3.0]), 4);
            let mut out = [0.0f32; 4];
            assert_eq!(ring.pop_samples(&mut out), 4);
            assert_eq!(out, [base, base + 1.0, base + 2.0, base + 3.0]);
        }
        assert_eq!(ring.available(), 0);
    }

    #[test]
    fn test_clear_discards_pending_samples() {
        let ring = SampleRing::new(8);
        ring.push_samples(&[1.0, 2.0, 3.0]);
        ring.clear();
        assert_eq!(ring.available(), 0);
        let mut out = [0.0f32; 4];
        assert_eq!(ring.pop_samples(&mut out), 0);
    }

    #[test]
    fn test_push_pop_frames_are_frame_aligned() {
        let ring = SampleRing::new(8);
        // 2 stereo frames.
        assert_eq!(ring.push_frames(&[1.0, -1.0, 2.0, -2.0], 2), 2);
        let mut out = [0.0f32; 4];
        assert_eq!(ring.pop_frames(&mut out, 2), 2);
        assert_eq!(out, [1.0, -1.0, 2.0, -2.0]);
    }

    #[test]
    fn test_push_frames_counts_whole_frame_drops() {
        let ring = SampleRing::new(4);
        // Capacity is 4 samples = 2 stereo frames; 3 frames offered => 1 frame dropped.
        assert_eq!(ring.push_frames(&[1.0, -1.0, 2.0, -2.0, 3.0, -3.0], 2), 2);
        assert_eq!(ring.dropped_samples(), 2);

        let mut out = [0.0f32; 4];
        assert_eq!(ring.pop_frames(&mut out, 2), 2);
        assert_eq!(out, [1.0, -1.0, 2.0, -2.0]);

        // clear resets the dropped counter.
        ring.clear();
        assert_eq!(ring.dropped_samples(), 0);
    }

    #[test]
    fn test_pop_frames_never_returns_a_partial_frame() {
        let ring = SampleRing::new(8);
        // Only 3 samples available (1.5 stereo frames).
        ring.push_samples(&[1.0, 2.0, 3.0]);
        let mut out = [0.0f32; 4];
        assert_eq!(ring.pop_frames(&mut out, 2), 1, "only one whole frame may be read");
        assert_eq!(&out[..2], &[1.0, 2.0]);
        assert_eq!(ring.available(), 1, "the partial frame stays buffered");
    }
}
