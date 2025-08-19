use std::ffi::c_void;

#[allow(non_camel_case_types)]
pub type lv_event_code_t = u32;
#[allow(non_camel_case_types)]
pub type lv_event_t = c_void;

extern "C" {
    pub fn lv_event_get_code(e: *mut lv_event_t) -> lv_event_code_t;
    pub fn lv_event_get_target(e: *mut lv_event_t) -> *mut c_void;
}
