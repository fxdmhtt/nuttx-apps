use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

#[derive(Default)]
struct Inner {
    cancelled: Cell<bool>,
    wakers: RefCell<Vec<Waker>>,
}

#[derive(Clone)]
pub struct CancellationTokenSource {
    inner: Rc<Inner>,
}

#[derive(Clone)]
pub struct CancellationToken {
    inner: Rc<Inner>,
}

#[derive(Debug)]
pub struct Cancelled;

impl CancellationTokenSource {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(Inner::default()),
        }
    }

    pub fn token(&self) -> CancellationToken {
        CancellationToken {
            inner: self.inner.clone(),
        }
    }

    pub fn cancel(&self) {
        if !self.inner.cancelled.replace(true) {
            for w in self.inner.wakers.borrow_mut().drain(..) {
                w.wake();
            }
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.get()
    }
}

impl CancellationToken {
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.get()
    }

    pub fn check_cancelled(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }

    pub fn cancelled(&self) -> CancelledFuture {
        CancelledFuture {
            token: self.clone(),
        }
    }
}

pub struct CancelledFuture {
    token: CancellationToken,
}

impl Future for CancelledFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            Poll::Ready(())
        } else {
            let mut wakers = self.token.inner.wakers.borrow_mut();
            if !wakers.iter().any(|w| w.will_wake(cx.waker())) {
                wakers.push(cx.waker().clone());
            }
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::{executor::LocalPool, pin_mut, select, task::LocalSpawnExt, FutureExt};
    use futures_timer::Delay;

    use super::*;

    #[test]
    fn cancel_two_tasks() {
        let cancelled_a = Rc::new(Cell::new(false));
        let cancelled_b = Rc::new(Cell::new(false));

        let task_a = |token: CancellationToken| {
            let cancelled_a = Rc::clone(&cancelled_a);

            async move {
                println!("Task A started");

                for i in 1..=10 {
                    let delay = Delay::new(Duration::from_millis(300)).fuse();
                    let cancelled = token.cancelled().fuse();

                    pin_mut!(delay, cancelled);

                    select! {
                        _ = delay => {
                            println!("Task A step {i}");
                        },
                        _ = cancelled => {
                            println!("Task A detected cancellation, cleaning up...");
                            // Cleanup && Dispose
                            cancelled_a.set(true);
                            break;
                        },
                    }
                }

                println!("Task A finished");
            }
        };

        let task_b = |token: CancellationToken| {
            let cancelled_b = Rc::clone(&cancelled_b);

            async move {
                println!("Task B started");

                for i in 1..=10 {
                    let delay = Delay::new(Duration::from_millis(500)).fuse();

                    pin_mut!(delay);

                    select! {
                        _ = delay => {
                            println!("Task B step {i}");
                            if token.check_cancelled().is_err() {
                                println!("Task B noticed cancellation after step {i}");
                                // Cleanup && Dispose
                                cancelled_b.set(true);
                                break;
                            }
                        }
                    }
                }

                println!("Task B finished");
            }
        };

        let cts = CancellationTokenSource::new();

        let mut pool = LocalPool::new();
        let spawner = pool.spawner();

        spawner
            .spawn_local(task_a(cts.token()).map(|_| ()))
            .unwrap();
        spawner
            .spawn_local(task_b(cts.token()).map(|_| ()))
            .unwrap();

        {
            let cts = cts.clone();
            spawner
                .spawn_local(
                    async move {
                        Delay::new(Duration::from_secs(2)).await;
                        println!("Cancelling all tasks!");
                        cts.cancel();
                    }
                    .map(|_| ()),
                )
                .unwrap();
        }

        pool.run();

        assert!(cts.is_cancelled());
        assert!(cancelled_a.get());
        assert!(cancelled_b.get());
    }
}
