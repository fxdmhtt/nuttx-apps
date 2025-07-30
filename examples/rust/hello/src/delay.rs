use std::{
    ffi::c_void,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    time::Duration,
};

use crate::{binding::libuv::timer::UvTimer, UI_LOOP};

pub struct Delay {
    state: Arc<Mutex<State>>,
    duration: Duration,
}

// Shared state between the future and the waiting thread
#[derive(Default)]
struct State {
    // Whether or not the sleep time has elapsed
    completed: bool,

    // The waker for the task that `Delay` is running on.
    // The thread can use this after setting `completed = true` to tell
    // `Delay`'s task to wake up, see that `completed = true`, and
    // move forward.
    waker: Option<Waker>,

    // Wrapper for libuv timer
    timer_handle: Option<UvTimer>,
}

impl Future for Delay {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Working with uv_timer_t
        self.state.lock().unwrap().timer_handle.get_or_insert_with(|| {
            let uv_timer = UvTimer::new(unsafe { UI_LOOP });
            uv_timer.start(
                self.duration.as_millis() as u64,
                Arc::into_raw(self.state.clone()) as *mut c_void,
            );
            uv_timer
        });

        // Look at the shared state to see if the timer has already completed.
        let mut state = self.state.lock().unwrap();
        if state.completed {
            Poll::Ready(())
        } else {
            // Set waker so that the thread can wake up the current task
            // when the timer has completed, ensuring that the future is polled
            // again and sees that `completed = true`.
            //
            // It's tempting to do this once rather than repeatedly cloning
            // the waker each time. However, the `Delay` can move between
            // tasks on the executor, which could cause a stale waker pointing
            // to the wrong task, preventing `Delay` from waking up
            // correctly.
            //
            // N.B. it's possible to check for this using the `Waker::will_wake`
            // function, but we omit that here to keep things simple.
            state.waker.replace(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl Delay {
    // Create a new `Delay` which will complete after the provided timeout.
    pub fn new(duration: Duration) -> Self {
        Delay {
            state: Arc::new(Mutex::new(State::default())),
            duration,
        }
    }
}

pub async fn delay(secs: u64) {
    Delay::new(std::time::Duration::from_secs(secs)).await;
}

#[no_mangle]
pub extern "C" fn rust_delay_wake(state: *mut c_void) {
    let state = unsafe { Arc::from_raw(state as *const Mutex<State>) };

    let mut state = state.lock().unwrap();
    if let Some(timer_handle) = state.timer_handle.take() {
        drop(timer_handle);
    }

    // Signal that the timer has completed and wake up the last
    // task on which the future was polled, if one exists.
    state.completed = true;
    if let Some(waker) = state.waker.take() {
        waker.wake()
    }
}
