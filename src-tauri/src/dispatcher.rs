//! Single-producer UI Command Dispatcher feeding the lock-free SPSC queue.

use open_boom_core::ring_buffer::SpscQueue;
use open_boom_core::types::AudioCommand;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

pub const COMMAND_QUEUE_CAPACITY: usize = 256;

/// Dispatcher bridging asynchronous Tauri commands to the real-time audio thread.
#[derive(Clone)]
pub struct UiDispatcher {
    sender: Sender<AudioCommand>,
}

impl UiDispatcher {
    pub fn new(queue: Arc<SpscQueue<AudioCommand, COMMAND_QUEUE_CAPACITY>>) -> Self {
        let (sender, receiver): (Sender<AudioCommand>, Receiver<AudioCommand>) = channel();

        // Worker thread that receives commands from async Tauri handlers
        // and pushes them into the single-producer lock-free ring buffer.
        thread::spawn(move || {
            while let Ok(cmd) = receiver.recv() {
                // If the ring buffer is temporarily full, yield and retry until pushed
                while queue.push(cmd).is_err() {
                    std::thread::yield_now();
                }
            }
        });

        Self { sender }
    }

    #[allow(clippy::result_large_err)]
    pub fn send(&self, cmd: AudioCommand) -> Result<(), std::sync::mpsc::SendError<AudioCommand>> {
        self.sender.send(cmd)
    }
}
