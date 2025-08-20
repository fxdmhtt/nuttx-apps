pub mod cancelled;
pub mod delay;
pub mod event;
pub mod executor;

use std::{ffi::c_void, future::Future, ptr::null_mut};

use async_executor::Task;

use crate::runtime::executor::PriorityExecutor;

static mut EXECUTOR: PriorityExecutor = PriorityExecutor::new();
pub static mut UI_LOOP: *mut c_void = null_mut();

#[allow(static_mut_refs)]
pub fn executor() -> &'static mut PriorityExecutor {
    unsafe { &mut EXECUTOR }
}

#[no_mangle]
pub extern "C" fn rust_executor_drive() {
    executor().try_tick_all()
}

#[no_mangle]
pub extern "C" fn rust_register_loop(ui_loop: *mut c_void) {
    assert!(!ui_loop.is_null());
    unsafe { UI_LOOP = ui_loop }
}

#[allow(non_snake_case)]
pub fn TaskRun<T: 'static>(future: impl Future<Output = T> + 'static) -> Task<T> {
    let task = executor().spawn(future);
    executor().try_tick_all();
    task
}
