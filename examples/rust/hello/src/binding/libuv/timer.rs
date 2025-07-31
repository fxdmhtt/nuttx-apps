#![allow(non_camel_case_types)]

use std::ffi::c_void;

use super::uv_loop_t;

type uv_timer_t = c_void;

pub struct UvTimer(*mut uv_timer_t);

impl Drop for UvTimer {
    fn drop(&mut self) {
        unsafe { uv_timer_drop(self.0) }
    }
}

impl UvTimer {
    pub fn new(ui_loop: *mut uv_loop_t) -> Self {
        Self(unsafe { uv_timer_new(ui_loop) })
    }

    pub fn start(&self, timeout: u64, state: *mut c_void) {
        unsafe { uv_timer_pending(self.0, timeout, state) }
    }
}

extern "C" {
    fn uv_timer_new(ui_loop: *mut uv_loop_t) -> *mut uv_timer_t;
    fn uv_timer_drop(handle: *mut uv_timer_t);
    fn uv_timer_pending(handle: *mut uv_timer_t, timeout: u64, state: *mut c_void);
}
