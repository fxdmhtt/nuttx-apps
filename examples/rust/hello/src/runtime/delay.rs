use std::{
    cell::RefCell,
    ffi::c_void,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
    time::Duration,
};

use async_cancellation_token::{CancellationToken, Cancelled, CancelledFuture};

use crate::{binding::libuv::UvTimer, runtime::UI_LOOP};

pub struct Delay {
    state: Rc<RefCell<State>>,
    duration: Duration,
    cancel_future: Option<CancelledFuture>,
}

#[derive(Debug, Default)]
enum DelayState {
    /// The delay is still pending.
    /// - The `uv_timer_t` may be running (or about to be started).
    /// - The waker may be set to wake the current task when the timer completes.
    /// - Cancellation may occur at any time.
    #[default]
    Pending,

    /// The delay has completed naturally (the timer fired).
    /// - The `uv_timer_t` callback has executed `rust_delay_wake()`.
    /// - The future should return `Ok(())` when polled.
    /// - Any further poll calls will observe `Completed` and immediately return `Ok(())`.
    Completed,

    /// The delay has been cancelled via a `CancellationToken`.
    /// - The registered cancellation callback may have stopped the timer.
    /// - The `CancelledFuture` will be ready, so poll should return `Err(Cancelled)`.
    /// - The future is considered terminated, and no further waiting should occur.
    Cancelled,
}

// Shared state between the future and the `uv_timer_t`
#[derive(Debug, Default)]
struct State {
    /// Current status of the delay.
    ///
    /// - `Pending`: The timer is running or about to start, the future is waiting.
    /// - `Completed`: The timer has fired, the future should return `Ok(())`.
    /// - `Cancelled`: Cancellation was triggered, the future should return `Err(Cancelled)`.
    ///
    /// This field drives the logic in `Delay::poll` and `rust_delay_wake`.
    state: DelayState,

    // The waker for the task that `Delay` is running on.
    // The `uv_timer_t` can use this after setting `completed = true`
    // to tell `Delay`'s task to wake up, see that `completed = true`,
    // and move forward.
    waker: Option<Waker>,

    // Wrapper for the `uv_timer_t`
    handle: Option<UvTimer>,
}

impl Future for Delay {
    type Output = Result<(), Cancelled>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        let mut s = this.state.borrow_mut();

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

        if let Some(fut) = &mut this.cancel_future {
            if let Poll::Ready(()) = Pin::new(fut).poll(cx) {
                assert!(matches!(s.state, DelayState::Pending));

                s.state = DelayState::Cancelled;
                return Poll::Ready(Err(Cancelled));
            }
        }

        // 2) Now do the rest under a single borrow_mut to reduce multiple borrows.
        // Create the uv timer only when the delay is still Pending and no handle exists.

        // Working with the `uv_timer_t`
        if s.handle.is_none() {
            // Ensure the timer is only created once for this Delay
            assert!(matches!(s.state, DelayState::Pending));

            let uv_timer = UvTimer::new(unsafe { UI_LOOP });
            uv_timer.start(
                this.duration.as_millis() as u64,
                Rc::as_ptr(&this.state) as *mut _,
            );
            s.handle.replace(uv_timer);
        }

        // Look at the shared state to see if the timer has already completed.
        match s.state {
            DelayState::Completed => Poll::Ready(Ok(())),
            DelayState::Cancelled => Poll::Ready(Err(Cancelled)),
            DelayState::Pending => {
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
            state: Rc::new(RefCell::new(State::default())),
            duration,
            cancel_future: None,
        }
    }

    // Create a new `Delay` with `CancellationToken` which will complete
    // either when the specified duration elapses
    // or when the associated `CancellationToken` is cancelled.
    pub fn new_with_token(duration: Duration, token: CancellationToken) -> Self {
        let state = Rc::new(RefCell::new(State::default()));

        // Register a cancellation callback with the token.
        //
        // We capture a `Weak<State>` so dropping the Delay makes the callback a no-op.
        // On cancellation, if the State is still alive we stop the native timer handle
        // to avoid further timer callbacks and free native resources early.
        //
        // Note: the wakeup is performed by `CancellationTokenSource::cancel()` which first
        // runs registered callbacks (like this stop closure) and then wakes token.wakers.
        // The CancelledFuture registers this task's waker with the token, so when
        // cancel() calls wake() the task will be resumed and can observe the Cancelled state.
        let weak_state = Rc::downgrade(&state);
        token.register(move || {
            if let Some(state) = weak_state.upgrade() {
                if let Some(handle) = state.borrow().handle.as_ref() {
                    handle.cancel();
                }
            }
        });

        Delay {
            state,
            duration,
            cancel_future: Some(token.cancelled()),
        }
    }
}

#[no_mangle]
extern "C" fn rust_delay_wake(state: *mut c_void) {
    assert!(!state.is_null());
    let state = unsafe { &*(state as *mut RefCell<State>) };

    let mut s = state.borrow_mut();

    assert!(matches!(s.state, DelayState::Pending));

    // Signal that the timer has completed and wake up the last
    // task on which the future was polled, if one exists.
    s.state = DelayState::Completed;
    if let Some(waker) = s.waker.take() {
        waker.wake()
    }
}

#[macro_export]
macro_rules! delay {
    ($secs:literal) => {
        $crate::runtime::delay::Delay::new(std::time::Duration::from_secs($secs))
    };
    ($secs:literal, $token:expr) => {
        $crate::runtime::delay::Delay::new_with_token(std::time::Duration::from_secs($secs), $token)
    };
    ($duration:expr) => {
        $crate::runtime::delay::Delay::new($duration)
    };
    ($duration:expr, $token:expr) => {
        $crate::runtime::delay::Delay::new_with_token($duration, $token)
    };
}
