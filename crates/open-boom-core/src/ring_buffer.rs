//! Lock-free Single-Producer Single-Consumer (SPSC) ring buffer for POD audio commands.

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A lock-free SPSC ring buffer with fixed capacity.
/// Zero dynamic allocations once initialized.
pub struct SpscQueue<T, const CAP: usize> {
    buffer: Box<[UnsafeCell<Option<T>>]>,
    head: AtomicUsize,
    tail: AtomicUsize,
}

// Safety: T is Send, and the ring buffer guarantees single-producer single-consumer access.
unsafe impl<T: Send, const CAP: usize> Sync for SpscQueue<T, CAP> {}
unsafe impl<T: Send, const CAP: usize> Send for SpscQueue<T, CAP> {}

impl<T: Copy + Send, const CAP: usize> SpscQueue<T, CAP> {
    /// Creates a new SPSC ring buffer with the specified fixed capacity.
    /// Capacity must be a power of two.
    pub fn new() -> Arc<Self> {
        assert!(CAP > 0 && (CAP & (CAP - 1)) == 0, "Capacity must be a power of two");
        let mut storage = Vec::with_capacity(CAP);
        for _ in 0..CAP {
            storage.push(UnsafeCell::new(None));
        }

        Arc::new(Self {
            buffer: storage.into_boxed_slice(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        })
    }

    /// Pushes an item into the queue. Called exclusively by the producer thread.
    /// Returns Ok(()) on success, or Err(item) if the queue is full.
    pub fn push(&self, item: T) -> Result<(), T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head.wrapping_sub(tail) >= CAP {
            return Err(item); // Full
        }

        let idx = head & (CAP - 1);
        unsafe {
            *self.buffer[idx].get() = Some(item);
        }

        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    /// Pops an item from the queue. Called exclusively by the consumer thread (audio real-time loop).
    /// Returns Some(item) if available, or None if the queue is empty.
    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail == head {
            return None; // Empty
        }

        let idx = tail & (CAP - 1);
        let item = unsafe { (*self.buffer[idx].get()).take() };

        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        item
    }

    /// Returns true if the queue is currently empty.
    pub fn is_empty(&self) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        tail == head
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AudioCommand;

    #[test]
    fn test_spsc_push_pop() {
        let queue = SpscQueue::<AudioCommand, 16>::new();
        assert!(queue.is_empty());

        let cmd = AudioCommand::SetMasterPower(true);
        assert!(queue.push(cmd).is_ok());
        assert!(!queue.is_empty());

        let popped = queue.pop();
        assert_eq!(popped, Some(cmd));
        assert!(queue.is_empty());
    }

    #[test]
    fn test_spsc_capacity_limits_and_overflow_rejection() {
        const CAP: usize = 4;
        let queue = SpscQueue::<u32, CAP>::new();

        // Fill to exact capacity
        for i in 0..CAP {
            assert!(queue.push(i as u32).is_ok(), "Pushing within capacity should succeed");
        }

        // Pushing beyond capacity must return Err(item)
        let overflow_item = 999;
        assert_eq!(
            queue.push(overflow_item),
            Err(overflow_item),
            "Pushing when full must reject and return the item"
        );

        // Pop one item, making room for one more
        assert_eq!(queue.pop(), Some(0));
        assert!(queue.push(100).is_ok(), "Pushing after pop should succeed");
        assert_eq!(queue.push(101), Err(101), "Pushing when full again must fail");
    }

    #[test]
    fn test_spsc_fifo_ordering() {
        const CAP: usize = 8;
        let queue = SpscQueue::<usize, CAP>::new();

        for i in 0..CAP {
            assert!(queue.push(i).is_ok());
        }

        for i in 0..CAP {
            assert_eq!(queue.pop(), Some(i), "Items must be popped in strict FIFO order");
        }
        assert!(queue.is_empty());
    }

    #[test]
    fn test_spsc_wraparound_cycles() {
        const CAP: usize = 4;
        let queue = SpscQueue::<usize, CAP>::new();

        // Run 100 cycles to verify head and tail wrap around without corruption
        for cycle in 0..100 {
            let base = cycle * CAP;
            for i in 0..CAP {
                assert!(queue.push(base + i).is_ok());
            }
            for i in 0..CAP {
                assert_eq!(queue.pop(), Some(base + i));
            }
            assert!(queue.is_empty());
        }
    }
}
