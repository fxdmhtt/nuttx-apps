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

    /// Attempts to run up to `num` ready tasks by polling them once.
    ///
    /// Stops early if no more tasks are available.
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
    ///
    /// EXECUTOR.try_tick_n(10); // Try to run up to 10 tasks
    /// ```
    #[allow(dead_code)]
    pub fn try_tick_n(&self, num: u32) {
        for _ in 0..num {
            if !self.try_tick() {
                break;
            }
        }
    }

    /// Non-blocking pass over all ready tasks, polling each once.
    ///
    /// Tasks that become ready during this call are deferred to the next tick.
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
    /// EXECUTOR.try_tick_all(); // Progress all ready tasks one step
    /// ```
    pub fn try_tick_all(&self) {
        while self.try_tick() {}
    }
}

impl Default for PriorityExecutor {
    fn default() -> Self {
        Self::new()
    }
}
