use std::{
    ffi::{c_int, c_void},
    ptr::NonNull,
};

use super::uv_loop_t;

#[allow(dead_code)]
extern "C" {
    static uv_timer_size: usize;
    static uv_timer_align: usize;
}

#[allow(dead_code)]
#[cfg(all(
    target_arch = "x86_64",
    target_vendor = "unknown",
    any(target_os = "nuttx", target_os = "linux")
))]
const UV_TIMER_SIZE: usize = 152;

#[allow(dead_code)]
#[cfg(all(
    target_arch = "x86_64",
    target_vendor = "unknown",
    any(target_os = "nuttx", target_os = "linux")
))]
const UV_TIMER_ALIGN: usize = 8;

#[allow(non_camel_case_types)]
type uv_timer_t = c_void;

#[derive(Debug)]
pub struct UvTimer(NonNull<uv_timer_t>);

impl Drop for UvTimer {
    fn drop(&mut self) {
        unsafe { uv_timer_drop(self.0.as_ptr()) }
    }
}

impl UvTimer {
    pub fn new(ui_loop: NonNull<uv_loop_t>) -> Self {
        // assert_eq!(std::mem::size_of::<uv_timer_t>(), unsafe { uv_timer_size });
        // assert_eq!(std::mem::align_of::<uv_timer_t>(), unsafe { uv_timer_align });

        Self(NonNull::new(unsafe { uv_timer_new(ui_loop.as_ptr()) }).unwrap())
    }

    pub fn start(&self, timeout: u64, state: *mut c_void) {
        assert!(!state.is_null());
        unsafe { uv_timer_pending(self.0.as_ptr(), timeout, state) }
    }

    pub fn cancel(&self) {
        unsafe { uv_timer_cancel(self.0.as_ptr()) }
    }
}

#[allow(dead_code)]
extern "C" {
    fn uv_timer_init(ui_loop: *mut uv_loop_t, handle: *mut uv_timer_t) -> c_int;
}

extern "C" {
    fn uv_timer_new(ui_loop: *mut uv_loop_t) -> *mut uv_timer_t;
    fn uv_timer_drop(handle: *mut uv_timer_t);
    fn uv_timer_pending(handle: *mut uv_timer_t, timeout: u64, state: *mut c_void);
    fn uv_timer_cancel(handle: *mut uv_timer_t);
}
