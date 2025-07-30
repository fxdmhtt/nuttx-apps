use async_executor::{StaticExecutor, Task};
use std::future::Future;

pub struct PriorityExecutor {
    executor: StaticExecutor,
}

unsafe impl Sync for PriorityExecutor {}

impl PriorityExecutor {
    /// Creates a new PriorityExecutor.
    ///
    /// # Examples
    ///
    /// ```
    /// use PriorityExecutor;
    ///
    /// static EXECUTOR: PriorityExecutor = PriorityExecutor::new();
    /// ```
    pub const fn new() -> Self {
        PriorityExecutor {
            executor: StaticExecutor::new(),
        }
    }

    /// Spawns a task onto the executor.
    ///
    /// Note: unlike [`Executor::spawn`], this function requires being called with a `'static`
    /// borrow on the executor.
    ///
    /// # Examples
    ///
    /// ```
    /// use PriorityExecutor;
    ///
    /// static EXECUTOR: PriorityExecutor = PriorityExecutor::new();
    ///
    /// let task = EXECUTOR.spawn(async {
    ///     println!("Hello world");
    /// });
    /// ```
    pub fn spawn<T: Send + 'static>(
        &'static self,
        future: impl Future<Output = T> + Send + 'static,
    ) -> Task<T> {
        self.executor.spawn(future)
    }

    /// Attempts to run a task if at least one is scheduled.
    ///
    /// Running a scheduled task means simply polling its future once.
    ///
    /// # Examples
    ///
    /// ```
    /// use PriorityExecutor;
    ///
    /// static EXECUTOR: PriorityExecutor = PriorityExecutor::new();
    ///
    /// assert!(!EXECUTOR.try_tick()); // no tasks to run
    ///
    /// let task = EXECUTOR.spawn(async {
    ///     println!("Hello world");
    /// });
    ///
    /// assert!(EXECUTOR.try_tick()); // a task was found
    /// ```
    pub fn try_tick(&self) -> bool {
        self.executor.try_tick()
    }

    /// Polls all currently pending or ready tasks once without blocking.
    ///
    /// This method performs a single non-blocking pass over the internal task queue,
    /// polling each task that is ready to make progress. It does **not** wait for I/O,
    /// timers, or other external events, and will return immediately after all
    /// currently scheduled tasks have been polled once.
    ///
    /// This is useful in scenarios where you want to drive the executor forward
    /// without blocking the current thread or waiting for new events.
    ///
    /// # Behavior
    ///
    /// - Only tasks that are currently ready or scheduled will be polled.
    /// - Tasks that are not ready (e.g., waiting on I/O or timers) are skipped.
    /// - Wakers may re-schedule tasks, but those will not be polled again
    ///   in the same `poll_all()` call.
    ///
    /// # Example
    ///
    /// ```
    /// use PriorityExecutor;
    ///
    /// static EXECUTOR: PriorityExecutor = PriorityExecutor::new();
    ///
    /// let task = EXECUTOR.spawn(async {
    ///     println!("Hello world");
    /// });
    ///
    /// EXECUTOR.poll_all(); // Progress all ready tasks one step
    /// ```
    ///
    /// # See also
    /// - [`spawn`] to add a task to the executor.
    ///
    /// # Note
    /// This method does not guarantee that all tasks will complete; it merely
    /// polls them once if they're ready.
    pub fn poll_all(&self) {
        while self.try_tick() {}
    }
}

impl Default for PriorityExecutor {
    fn default() -> Self {
        Self::new()
    }
}
