use std::{
    cell::RefCell,
    ffi::c_void,
    future::Future,
    pin::Pin,
    ptr::NonNull,
    rc::Rc,
    task::{Context, Poll, Waker},
    time::Duration,
};

use async_cancellation_token::{CancellationToken, CancellationTokenRegistration, Cancelled, CancelledFuture};
use pin_project::pin_project;

use crate::runtime::lvgl::*;

#[pin_project]
pub struct LvAnim<'a> {
    #[pin]
    state: State,
    var: &'a NonNull<c_void>,
    duration: Duration,
    exec_cb: lv_anim_exec_xcb_t,
    values: (i32, i32),
    options: Options,
    extra: Extra,
}

#[derive(Debug, Default)]
struct State {
    state: PollState,
    waker: Option<Waker>,
    handle: Rc<RefCell<Option<crate::binding::lvgl::anim::LvAnim>>>,
    completed_cb: Option<lv_anim_completed_cb_t>,
}

#[derive(Debug, Default, Copy, Clone)]
enum PollState {
    #[default]
    Pending,
    Completed,
    Cancelled,
}

#[derive(Debug, Default)]
pub struct Options {
    completed_cb: Option<lv_anim_completed_cb_t>,
}

#[derive(Default)]
enum Extra {
    #[default]
    Plain,
    Cancellable(CancelledFuture, Option<CancellationTokenRegistration>),
}

impl Future for LvAnim<'_> {
    type Output = Result<(), Cancelled>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let s = this.state.get_mut();

        if let Extra::Cancellable(fut, _reg) = this.extra {
            if let Poll::Ready(()) = Pin::new(fut).poll(cx) {
                assert!(matches!(s.state, PollState::Pending | PollState::Cancelled));
                s.state = PollState::Cancelled;
                return Poll::Ready(Err(Cancelled));
            }
        }

        if matches!(s.state, PollState::Pending) && s.handle.borrow().is_none() {
            let lv_anim = crate::binding::lvgl::anim::LvAnim::new();
            lv_anim.start(
                this.var,
                *this.exec_cb,
                this.duration.as_millis() as u32,
                this.values.0,
                this.values.1,
                s as *mut _ as _,
            );
            s.handle.borrow_mut().replace(lv_anim);
        }

        match s.state {
            PollState::Completed => Poll::Ready(Ok(())),
            PollState::Cancelled => Poll::Ready(Err(Cancelled)),
            PollState::Pending => {
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

impl<'a> LvAnim<'a> {
    pub fn new(var: &'a NonNull<c_void>, duration: Duration, exec_cb: lv_anim_exec_xcb_t, values: (i32, i32)) -> Self {
        LvAnim {
            state: Default::default(),
            var,
            duration,
            exec_cb,
            values,
            options: Default::default(),
            extra: Default::default(),
        }
    }

    pub fn set_options(&mut self, options: Options) {
        self.state.completed_cb = options.completed_cb;
        self.options = options;
    }

    pub fn set_cancel(self: Pin<&mut Self>, token: CancellationToken) {
        let this = self.project();
        let s = this.state.get_mut();

        assert!(
            !matches!(this.extra, Extra::Cancellable(_, _)),
            "LvAnim can only be canceled once."
        );

        *this.extra = Extra::Cancellable(
            token.cancelled(),
            if token.is_cancelled() {
                token.register(|| {})
            } else {
                let handle = Rc::downgrade(&s.handle);
                token.register(move || match handle.upgrade() {
                    Some(handle) => match &*handle.borrow() {
                        Some(handle) => handle.cancel(),
                        None => unreachable!(),
                    },
                    None => unreachable!(),
                })
            },
        );
    }
}

#[no_mangle]
extern "C" fn rust_anim_wake(state: *mut c_void) {
    assert!(!state.is_null());
    let s = unsafe { &mut *(state as *mut State) };

    assert!(matches!(s.state, PollState::Pending));

    if let Some(cb) = s.completed_cb {
        unsafe { cb(s.handle.borrow().as_ref().unwrap().get().unwrap().as_ptr()) };
    }

    s.state = PollState::Completed;
    if let Some(waker) = s.waker.take() {
        waker.wake()
    }
}
