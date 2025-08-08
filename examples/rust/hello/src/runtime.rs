pub mod delay;
pub mod executor;

use std::{ffi::c_void, ptr::null_mut};

use crate::runtime::executor::PriorityExecutor;

#[macro_export]
macro_rules! event {
    ($func:ident, $arg:ident, async $body:block) => {
        #[no_mangle]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub extern "C" fn $func($arg: *mut lv_event_t) {
            $crate::executor().spawn(async move { $body }).detach();
            $crate::executor().try_tick_all();
        }
    };
    ($func:ident, $arg:ident, $body:block) => {
        #[no_mangle]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub extern "C" fn $func($arg: *mut lv_event_t) {
            $body
        }
    };
}

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
