use std::ffi::c_void;

use super::uv_loop_t;

#[allow(non_camel_case_types)]
type uv_timer_t = c_void;

#[derive(Debug)]
pub struct UvTimer(*mut uv_timer_t);

impl Drop for UvTimer {
    fn drop(&mut self) {
        unsafe { uv_timer_drop(self.0) }
    }
}

impl UvTimer {
    pub fn new(ui_loop: *mut uv_loop_t) -> Self {
        assert!(!ui_loop.is_null());
        Self(unsafe { uv_timer_new(ui_loop) })
    }

    pub fn start(&self, timeout: u64, state: *mut c_void) {
        assert!(!state.is_null());
        unsafe { uv_timer_pending(self.0, timeout, state) }
    }
}

extern "C" {
    fn uv_timer_new(ui_loop: *mut uv_loop_t) -> *mut uv_timer_t;
    fn uv_timer_drop(handle: *mut uv_timer_t);
    fn uv_timer_pending(handle: *mut uv_timer_t, timeout: u64, state: *mut c_void);
}
