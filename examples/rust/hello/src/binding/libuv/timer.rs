use std::ffi::c_void;

pub struct UvTimer(*mut c_void);

unsafe impl Send for UvTimer {}

impl Drop for UvTimer {
    fn drop(&mut self) {
        unsafe { uv_timer_drop(self.0) }
    }
}

impl UvTimer {
    pub fn new(ui_loop: *mut c_void) -> Self {
        Self(unsafe { uv_timer_new(ui_loop) })
    }

    pub fn start(&self, timeout: u64, state: *mut c_void) {
        unsafe { uv_timer_pending(self.0, timeout, state) }
    }
}

extern "C" {
    fn uv_timer_new(ui_loop: *mut c_void) -> *mut c_void;
    fn uv_timer_drop(handle: *mut c_void);
    fn uv_timer_pending(handle: *mut c_void, timeout: u64, state: *mut c_void);
}
