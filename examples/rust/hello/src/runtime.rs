pub mod r#async;
pub mod lvgl;

pub use r#async::*;

use std::{ffi::c_void, future::Future, ptr::NonNull};

use async_executor::Task;

use crate::runtime::PriorityExecutor;

static mut EXECUTOR: PriorityExecutor = PriorityExecutor::new();
pub static mut UI_LOOP: Option<NonNull<c_void>> = None;

#[allow(static_mut_refs)]
pub fn executor() -> &'static mut PriorityExecutor {
    unsafe { &mut EXECUTOR }
}

#[no_mangle]
extern "C" fn rust_executor_drive() {
    executor().try_tick_all()
}

#[no_mangle]
extern "C" fn rust_register_loop(ui_loop: *mut c_void) {
    unsafe { UI_LOOP = Some(NonNull::new(ui_loop).unwrap()) }
}

#[allow(non_snake_case)]
pub fn TaskRun<T: 'static>(future: impl Future<Output = T> + 'static) -> Task<T> {
    let task = executor().spawn(future);
    executor().try_tick_all();
    task
}

#[macro_export]
macro_rules! clone {
    ( $( $var:ident ),* $(,)? ) => {
        $(
            let $var = $var.clone();
        )*
    };

    ( $( $self:ident . $field:ident ),* $(,)? ) => {
        $(
            let $field = $self.$field.clone();
        )*
    };
}

#[macro_export]
macro_rules! downgrade {
    ( $( $var:ident ),* $(,)? ) => {
        $(
            let $var = std::rc::Rc::downgrade(&$var);
        )*
    };
}
