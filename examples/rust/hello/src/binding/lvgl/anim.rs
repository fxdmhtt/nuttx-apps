use std::{ffi::c_void, ptr::NonNull};

use super::*;

#[derive(Debug)]
pub struct LvAnim(NonNull<lv_anim_t>);

impl Drop for LvAnim {
    fn drop(&mut self) {
        unsafe { lv_anim_drop(self.0.as_ptr()) }
    }
}

impl LvAnim {
    pub fn new() -> Self {
        Self(NonNull::new(unsafe { lv_anim_new() }).unwrap())
    }

    pub fn start(&self, var: &NonNull<c_void>, exec_cb: lv_anim_exec_xcb_t, duration: u32, start: i32, end: i32, state: *mut c_void) {
        debug_assert!(!state.is_null());

        unsafe { lv_anim_set_var(self.0.as_ptr(), var.as_ptr()) };
        unsafe { lv_anim_set_exec_cb(self.0.as_ptr(), exec_cb) };
        unsafe { lv_anim_set_duration(self.0.as_ptr(), duration) };
        unsafe { lv_anim_set_values(self.0.as_ptr(), start, end) };
        unsafe { lv_anim_pending(self.0.as_ptr(), state) };
    }

    pub fn get(&self) -> Option<NonNull<lv_anim_t>> {
        NonNull::new(unsafe { lv_anim_query(self.0.as_ptr()) })
    }

    pub fn cancel(&self) {
        let ret = unsafe { lv_anim_cancel(self.0.as_ptr()) };
        debug_assert!(ret);
    }

    pub fn set_delay(self, delay: u32) -> Self {
        unsafe { lv_anim_set_delay(self.0.as_ptr(), delay) };
        self
    }

    pub fn set_custom_exec_cb(self, exec_cb: lv_anim_custom_exec_cb_t) -> Self {
        unsafe { lv_anim_set_custom_exec_cb(self.0.as_ptr(), exec_cb) };
        self
    }

    pub fn set_path_cb(self, path_cb: lv_anim_path_cb_t) -> Self {
        unsafe { lv_anim_set_path_cb(self.0.as_ptr(), path_cb) };
        self
    }

    pub fn set_start_cb(self, start_cb: lv_anim_start_cb_t) -> Self {
        unsafe { lv_anim_set_start_cb(self.0.as_ptr(), start_cb) };
        self
    }

    pub fn set_get_value_cb(self, get_value_cb: lv_anim_get_value_cb_t) -> Self {
        unsafe { lv_anim_set_get_value_cb(self.0.as_ptr(), get_value_cb) };
        self
    }

    #[deprecated]
    fn set_completed_cb(self, completed_cb: lv_anim_completed_cb_t) -> Self {
        unsafe { lv_anim_set_completed_cb(self.0.as_ptr(), completed_cb) };
        self
    }

    pub fn set_deleted_cb(self, deleted_cb: lv_anim_deleted_cb_t) -> Self {
        unsafe { lv_anim_set_deleted_cb(self.0.as_ptr(), deleted_cb) };
        self
    }

    pub fn set_playback_duration(self, duration: u32) -> Self {
        unsafe { lv_anim_set_playback_duration(self.0.as_ptr(), duration) };
        self
    }

    pub fn set_playback_delay(self, delay: u32) -> Self {
        unsafe { lv_anim_set_playback_delay(self.0.as_ptr(), delay) };
        self
    }

    pub fn set_repeat_count(self, cnt: u16) -> Self {
        unsafe { lv_anim_set_repeat_count(self.0.as_ptr(), cnt) };
        self
    }

    pub fn set_repeat_delay(self, delay: u32) -> Self {
        unsafe { lv_anim_set_repeat_delay(self.0.as_ptr(), delay) };
        self
    }

    pub fn set_early_apply(self, en: bool) -> Self {
        unsafe { lv_anim_set_early_apply(self.0.as_ptr(), en) };
        self
    }
}

extern "C" {
    fn lv_anim_set_var(a: *mut lv_anim_t, var: *mut c_void);
    fn lv_anim_set_exec_cb(a: *mut lv_anim_t, exec_cb: lv_anim_exec_xcb_t);
    fn lv_anim_set_duration(a: *mut lv_anim_t, duration: u32);
    fn lv_anim_set_delay(a: *mut lv_anim_t, delay: u32);
    fn lv_anim_set_values(a: *mut lv_anim_t, start: i32, end: i32);
    fn lv_anim_set_custom_exec_cb(a: *mut lv_anim_t, exec_cb: lv_anim_custom_exec_cb_t);
    fn lv_anim_set_path_cb(a: *mut lv_anim_t, path_cb: lv_anim_path_cb_t);
    fn lv_anim_set_start_cb(a: *mut lv_anim_t, start_cb: lv_anim_start_cb_t);
    fn lv_anim_set_get_value_cb(a: *mut lv_anim_t, get_value_cb: lv_anim_get_value_cb_t);
    fn lv_anim_set_completed_cb(a: *mut lv_anim_t, completed_cb: lv_anim_completed_cb_t);
    fn lv_anim_set_deleted_cb(a: *mut lv_anim_t, deleted_cb: lv_anim_deleted_cb_t);
    fn lv_anim_set_playback_duration(a: *mut lv_anim_t, duration: u32);
    fn lv_anim_set_playback_delay(a: *mut lv_anim_t, delay: u32);
    fn lv_anim_set_repeat_count(a: *mut lv_anim_t, cnt: u16);
    fn lv_anim_set_repeat_delay(a: *mut lv_anim_t, delay: u32);
    fn lv_anim_set_early_apply(a: *mut lv_anim_t, en: bool);
}

extern "C" {
    fn lv_anim_new() -> *mut lv_anim_t;
    fn lv_anim_drop(a: *mut lv_anim_t);
    fn lv_anim_pending(a: *mut lv_anim_t, state: *mut c_void) -> *mut lv_anim_t;
    fn lv_anim_query(a: *mut lv_anim_t) -> *mut lv_anim_t;
    fn lv_anim_cancel(a: *mut lv_anim_t) -> bool;
}
