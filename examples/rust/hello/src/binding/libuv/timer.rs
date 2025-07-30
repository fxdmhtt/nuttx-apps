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
        unsafe { rs_delay_start(self.0, timeout, state) }
    }
}

unsafe extern "C" {
    pub unsafe fn uv_timer_new(ui_loop: *mut c_void) -> *mut c_void;
    pub unsafe fn uv_timer_drop(handle: *mut c_void);
    pub unsafe fn rs_delay_start(handle: *mut c_void, timeout: u64, state: *mut c_void);
}
