//! Multimedia Class Scheduler Service (MMCSS) RAII thread prioritization.
//!
//! Registers the real-time audio thread under MMCSS "Pro Audio" to guarantee
//! prioritized CPU scheduling and prevent preemption jitter.

/// RAII guard managing MMCSS thread prioritization.
pub struct MmcssGuard {
    #[cfg(windows)]
    handle: Option<windows::Win32::Foundation::HANDLE>,
    task_name: String,
    task_index: u32,
    is_active: bool,
}

impl MmcssGuard {
    /// Attempts to register the calling thread under the specified MMCSS task name.
    ///
    /// If registration fails (e.g. running in headless CI or without MMCSS permissions),
    /// falls back gracefully to standard thread scheduling without panicking.
    pub fn register(task_name: &str) -> Self {
        #[cfg(windows)]
        {
            use windows::core::HSTRING;
            use windows::Win32::System::Threading::AvSetMmThreadCharacteristicsW;

            let h_name = HSTRING::from(task_name);
            let mut task_index = 0u32;
            let result = unsafe { AvSetMmThreadCharacteristicsW(&h_name, &mut task_index) };

            match result {
                Ok(handle) if !handle.is_invalid() => Self {
                    handle: Some(handle),
                    task_name: task_name.to_string(),
                    task_index,
                    is_active: true,
                },
                _ => Self {
                    handle: None,
                    task_name: task_name.to_string(),
                    task_index: 0,
                    is_active: false,
                },
            }
        }
        #[cfg(not(windows))]
        {
            Self {
                task_name: task_name.to_string(),
                task_index: 0,
                is_active: false,
            }
        }
    }

    /// Returns true if the thread is actively scheduled under MMCSS Pro Audio.
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// The MMCSS task name requested.
    #[inline(always)]
    pub fn task_name(&self) -> &str {
        &self.task_name
    }

    /// The MMCSS task index assigned by Windows.
    #[inline(always)]
    pub fn task_index(&self) -> u32 {
        self.task_index
    }
}

impl Drop for MmcssGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        if let Some(handle) = self.handle.take() {
            if !handle.is_invalid() {
                unsafe {
                    let _ = windows::Win32::System::Threading::AvRevertMmThreadCharacteristics(handle);
                }
            }
        }
    }
}

/// Tracks audio callback execution duration against real-time latency budgets.
#[derive(Debug, Clone, Copy)]
pub struct CallbackBudgetTracker {
    /// Buffer duration in milliseconds (e.g. 10.0 ms for 480 samples at 48 kHz).
    pub buffer_duration_ms: f64,
    /// Maximum allowable callback execution time (60% of buffer duration).
    pub max_allowed_duration_ms: f64,
    /// Last measured callback duration in milliseconds.
    pub last_callback_duration_ms: f64,
    /// Whether the last callback exceeded the 60% budget.
    pub budget_exceeded: bool,
    /// Total callbacks recorded.
    pub total_callbacks: u64,
    /// Number of callbacks that exceeded the 60% budget.
    pub exceeded_callbacks: u64,
}

impl CallbackBudgetTracker {
    /// Initializes a new budget tracker for a given buffer duration in milliseconds.
    pub fn new(buffer_duration_ms: f64) -> Self {
        Self {
            buffer_duration_ms,
            max_allowed_duration_ms: buffer_duration_ms * 0.60,
            last_callback_duration_ms: 0.0,
            budget_exceeded: false,
            total_callbacks: 0,
            exceeded_callbacks: 0,
        }
    }

    /// Records callback duration and updates budget violation statistics.
    ///
    /// Returns `true` if the callback duration exceeded the 60% limit.
    pub fn record_callback(&mut self, elapsed_ms: f64) -> bool {
        self.total_callbacks += 1;
        self.last_callback_duration_ms = elapsed_ms;
        let exceeded = elapsed_ms > self.max_allowed_duration_ms;
        self.budget_exceeded = exceeded;
        if exceeded {
            self.exceeded_callbacks += 1;
        }
        exceeded
    }
}
