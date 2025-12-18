use std::{
    ffi::c_void,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};

use async_cancellation_token::{CancellationToken, CancellationTokenRegistration, Cancelled, CancelledFuture};
use pin_project::pin_project;

use crate::{binding::libuv::UvTimer, runtime::UI_LOOP};

#[pin_project]
#[derive(Default)]
pub struct Delay {
    #[pin]
    state: State,
    duration: Duration,
    extra: Extra,
}

// Shared state between the future and the `uv_timer_t`
#[derive(Debug, Default)]
struct State {
    // Current status of the delay.
    state: PollState,

    // Task waker, used by `uv_timer_t` callback to wake the future.
    waker: Option<Waker>,

    // Wrapper for the `uv_timer_t`
    handle: Option<UvTimer>,
}

#[derive(Debug, Default, Copy, Clone)]
enum PollState {
    #[default]
    Pending,
    Completed,
    Cancelled,
}

#[derive(Default)]
enum Extra {
    #[default]
    Plain,
    Cancellable(CancelledFuture, Option<CancellationTokenRegistration>),
}

impl Future for Delay {
    type Output = Result<(), Cancelled>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let s = this.state.get_mut();

        // 1) Poll the cancellation future (if any) first.
        //
        // Mechanism and reasoning:
        //
        // - `CancelledFuture` doesn't run independently; it relies on being polled
        //   to observe cancellation.
        // - When `Delay::poll` polls `cancel_future`, it passes its own `cx` waker.
        //   This effectively registers the Delay's waker with the CancellationToken.
        // - If the token is already cancelled or gets cancelled later, the token
        //   will wake this waker, causing Delay to be polled again.
        // - This ensures that cancellation is observed immediately and consistently,
        //   without needing an independent executor for `cancel_future`.
        // - Therefore, we must poll `cancel_future` first in `Delay::poll` so that
        //   cancellation takes priority over the timer, and the Delay future can
        //   return `Err(Cancelled)` as soon as possible.
        //
        // If ready, mark the state as Cancelled and return `Err(Cancelled)` immediately.
        // Note: dropping the handle early is optional, since Delay will drop it
        // automatically when it goes out of scope.
        if let Extra::Cancellable(fut, _reg) = this.extra {
            if let Poll::Ready(()) = Pin::new(fut).poll(cx) {
                assert!(matches!(s.state, PollState::Pending | PollState::Cancelled));
                s.state = PollState::Cancelled;
                return Poll::Ready(Err(Cancelled));
            }
        }

        // 2) Now do the rest under a single borrow_mut to reduce multiple borrows.
        // Create the uv timer only when the delay is still Pending and no handle exists.

        // Working with the `uv_timer_t`
        if matches!(s.state, PollState::Pending) && s.handle.is_none() {
            let uv_timer = UvTimer::new((unsafe { UI_LOOP }).unwrap());
            uv_timer.start(this.duration.as_millis() as u64, s as *mut _ as *mut _);
            s.handle.replace(uv_timer);
        }

        // Look at the shared state to see if the timer has already completed.
        match s.state {
            PollState::Completed => Poll::Ready(Ok(())),
            PollState::Cancelled => Poll::Ready(Err(Cancelled)),
            PollState::Pending => {
                // Set waker so that the `uv_timer_t` can wake up the current task
                // when the timer has completed, ensuring that the future is polled
                // again and sees that `completed = true`.
                //
                // It's tempting to do this once rather than repeatedly cloning
                // the waker each time. However, the `Delay` can move between
                // tasks on the executor, which could cause a stale waker pointing
                // to the wrong task, preventing `Delay` from waking up correctly.
                //
                // Only replace the waker if it's different from the current one or not set,
                // to avoid unnecessary cloning of wakers.
                if s.waker
                    .as_ref()
                    .map(|w| !w.will_wake(cx.waker()))
                    .unwrap_or(true)
                {
                    s.waker.replace(cx.waker().clone());
                }
                Poll::Pending
            }
        }
    }
}

impl Delay {
    // Create a new `Delay` which will complete after the specified duration elapses.
    pub fn new(duration: Duration) -> Self {
        Delay {
            duration,
            ..Default::default()
        }
    }

    /// Make this Delay cancellable by the given `CancellationToken`.
    ///
    /// Once registered, the Delay will complete either:
    /// 1. when the timer duration elapses, or
    /// 2. when the token is cancelled.
    ///
    /// Waker notes:
    /// - Polling the `CancelledFuture` registers the task's waker with the token.
    /// - When `cancel()` is called, the task will be woken and `poll` will return `Err(Cancelled)`.
    pub fn set_cancel(self: Pin<&mut Self>, token: CancellationToken) {
        let this = self.project();
        let s = this.state.get_mut();

        assert!(
            matches!(this.extra, Extra::Plain),
            "Delay can only be cancellable once"
        );

        // Register a cancellation callback with the token.
        //
        // Note: the wakeup is performed by `CancellationTokenSource::cancel()` which first
        // runs registered callbacks (like this stop closure) and then wakes token.wakers.
        // The CancelledFuture registers this task's waker with the token, so when
        // cancel() calls wake() the task will be resumed and can observe the Cancelled state.
        *this.extra = Extra::Cancellable(
            token.cancelled(),
            if token.is_cancelled() {
                token.register(|| {})
            } else {
                // To implement a pure stack-based `Delay`, unsafe code is used to avoid `Rc`.
                //
                // Unsafe usage warning:
                //
                // The following unsafe block dereferences a raw pointer to `s.handle`.
                //
                // Safety analysis:
                // - `handle` is a pointer to `Option<UvTimer>` inside `Delay.state`.
                // - `Extra::Cancellable` holds the `CancellationTokenRegistration`.
                // - `CancellationTokenRegistration`'s Drop implementation unregisters the closure from the token.
                //   This ensures that when `Delay` is dropped, the closure is removed and will never be called
                //   after `Delay` no longer exists.
                // - Therefore, this unsafe is safe *as long as*:
                //     1. The `Delay` is pinned and alive for the entire time the `CancellationToken` can call the callback.
                //     2. No other code moves or invalidates `s.handle` while the registration exists.
                // - Risk points:
                //     - If the `CancellationToken` outlives the `Delay` and the registration somehow fires after
                //       `Delay` is dropped, this would be undefined behavior.
                //     - In single-threaded, short-lived Delay usage with registration dropped in Extra::Cancellable's Drop,
                //       this pattern is sound.
                // - Always document this assumption clearly.
                let handle: *const Option<UvTimer> = &s.handle;
                token.register(move || match unsafe { &*handle } {
                    Some(handle) => handle.cancel(),
                    None => unreachable!(),
                })
            },
        );
    }
}

#[no_mangle]
extern "C" fn rust_delay_wake(state: *mut c_void) {
    assert!(!state.is_null());
    let s = unsafe { &mut *(state as *mut State) };

    // Safety:
    // - `state` must point to a valid State inside a pinned Delay.
    // - This is guaranteed by Delay being alive and pinned for the lifetime of uv_timer.
    assert!(matches!(s.state, PollState::Pending));

    // Signal that the timer has completed and wake up the last
    // task on which the future was polled, if one exists.
    s.state = PollState::Completed;
    if let Some(waker) = s.waker.take() {
        waker.wake()
    }
}

#[macro_export]
macro_rules! delay {
    ($secs:literal) => {
        $crate::runtime::Delay::new(std::time::Duration::from_secs($secs))
    };
    ($secs:literal, $token:expr) => {
        async {
            let d = delay!(std::time::Duration::from_secs($secs));
            futures::pin_mut!(d);
            d.as_mut().set_cancel($token);
            d.await
        }
    };
    ($duration:expr) => {
        $crate::runtime::Delay::new($duration)
    };
    ($duration:expr, $token:expr) => {
        async {
            let d = delay!($duration);
            futures::pin_mut!(d);
            d.as_mut().set_cancel($token);
            d.await
        }
    };
}
