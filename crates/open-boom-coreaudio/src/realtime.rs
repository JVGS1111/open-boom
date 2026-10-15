//! Real-time thread scheduling for CoreAudio audio threads.
//!
//! macOS has no MMCSS equivalent; the supported user-space mechanism is the POSIX
//! real-time scheduler policy `SCHED_RR` configured through `pthread_setschedparam`.
//! Unprivileged processes typically receive `EPERM` for elevated real-time priorities,
//! so failure is reported with its `errno` and callers degrade gracefully to the
//! default scheduler instead of aborting.

use std::marker::PhantomData;

use open_boom_core::types::{EngineError, EngineResult};

/// Nominal real-time priority applied to the OpenBoom audio pump thread.
pub const DEFAULT_AUDIO_PRIORITY: i32 = 47;

/// Attempts to place the calling thread on the `SCHED_RR` real-time policy.
///
/// Returns `Ok(())` when the kernel accepted the policy change. On failure returns an
/// [`EngineError`] carrying the OS `errno`, so a privilege denial (`EPERM`) can be told
/// apart from an invalid argument (`EINVAL`).
#[cfg(target_os = "macos")]
pub fn set_realtime_priority(priority: i32) -> EngineResult<()> {
    let mut param: libc::sched_param = unsafe { std::mem::zeroed() };
    param.sched_priority = priority;
    let result =
        unsafe { libc::pthread_setschedparam(libc::pthread_self(), libc::SCHED_RR, &param) };
    if result == 0 {
        return Ok(());
    }
    let errno = std::io::Error::last_os_error();
    Err(EngineError::DeviceInitialization(format!(
        "pthread_setschedparam(SCHED_RR, priority {priority}) failed: {errno}"
    )))
}

/// Non-macOS platforms have no CoreAudio RT scheduler.
#[cfg(not(target_os = "macos"))]
pub fn set_realtime_priority(_priority: i32) -> EngineResult<()> {
    Err(EngineError::UnsupportedFormat(
        "no POSIX real-time scheduler on this platform".to_string(),
    ))
}

/// RAII guard that applies real-time scheduling to the current thread and restores the
/// previously captured scheduling policy on drop.
///
/// The guard is intentionally **not `Send`**: real-time scheduling is a property of the
/// thread it was applied on, so moving it to another thread would revert the wrong one.
/// `is_active()` reflects whether the kernel actually granted the real-time policy; it is
/// `false` off-macOS and also `false` on unprivileged macOS runs.
pub struct RealtimeGuard {
    active: bool,
    priority: i32,
    #[cfg(target_os = "macos")]
    previous: Option<libc::sched_param>,
    #[cfg(target_os = "macos")]
    previous_policy: i32,
    // `*const ()` is neither `Send` nor `Sync`, pinning the guard to its thread.
    _not_send: PhantomData<*const ()>,
}

impl RealtimeGuard {
    /// Applies real-time scheduling to the calling thread, falling back silently when the
    /// kernel denies the request.
    pub fn apply(priority: i32) -> Self {
        #[cfg(target_os = "macos")]
        {
            let mut previous: libc::sched_param = unsafe { std::mem::zeroed() };
            let mut previous_policy: i32 = libc::SCHED_OTHER;
            let captured = unsafe {
                libc::pthread_getschedparam(
                    libc::pthread_self(),
                    &mut previous_policy,
                    &mut previous,
                )
            } == 0;
            let active = set_realtime_priority(priority).is_ok();
            Self {
                active,
                priority,
                previous: captured.then_some(previous),
                previous_policy,
                _not_send: PhantomData,
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = set_realtime_priority(priority);
            Self {
                active: false,
                priority,
                _not_send: PhantomData,
            }
        }
    }

    /// Returns whether real-time scheduling is actually in effect on this thread.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Returns the requested real-time priority.
    pub fn priority(&self) -> i32 {
        self.priority
    }
}

impl Drop for RealtimeGuard {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        if self.active {
            if let Some(previous) = self.previous {
                unsafe {
                    let _ = libc::pthread_setschedparam(
                        libc::pthread_self(),
                        self.previous_policy,
                        &previous,
                    );
                }
            }
        }
    }
}
