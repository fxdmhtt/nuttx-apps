use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

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
}

impl Future for Delay {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Spawn the new thread
        let state = self.state.clone();
        let duration = self.duration;
        thread::spawn(move || {
            thread::sleep(duration);

            // Completed
            let mut state = state.lock().unwrap();
            // Signal that the timer has completed and wake up the last
            // task on which the future was polled, if one exists.
            state.completed = true;
            if let Some(waker) = state.waker.take() {
                waker.wake()
            }
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
