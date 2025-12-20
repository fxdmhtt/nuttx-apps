use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

#[derive(Default)]
pub struct Yield {
    state: State,
}

#[derive(Debug, Default)]
struct State {
    state: PollState,
}

#[derive(Debug, Default, Copy, Clone)]
enum PollState {
    #[default]
    Pending,
    Completed,
}

impl Future for Yield {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let s = &mut self.get_mut().state;
        match s.state {
            PollState::Pending => {
                s.state = PollState::Completed;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            PollState::Completed => Poll::Ready(()),
        }
    }
}

#[macro_export]
macro_rules! yield_now {
    () => {
        $crate::runtime::Yield::default()
    };
}
