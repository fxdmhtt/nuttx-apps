#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ffi::{c_char, c_void};

pub type lv_event_code_t = u32;
pub type lv_event_t = c_void;
pub type lv_obj_t = c_void;
pub type lv_opa_t = u8;
pub type lv_style_selector_t = u32;

pub const LV_STATE_DEFAULT: i32 = 0x0000;
pub const LV_STATE_CHECKED: i32 = 0x0001;
pub const LV_STATE_FOCUSED: i32 = 0x0002;
pub const LV_STATE_FOCUS_KEY: i32 = 0x0004;
pub const LV_STATE_EDITED: i32 = 0x0008;
pub const LV_STATE_HOVERED: i32 = 0x0010;
pub const LV_STATE_PRESSED: i32 = 0x0020;
pub const LV_STATE_SCROLLED: i32 = 0x0040;
pub const LV_STATE_DISABLED: i32 = 0x0080;
pub const LV_STATE_USER_1: i32 = 0x1000;
pub const LV_STATE_USER_2: i32 = 0x2000;
pub const LV_STATE_USER_3: i32 = 0x4000;
pub const LV_STATE_USER_4: i32 = 0x8000;
pub const LV_STATE_ANY: i32 = 0xFFFF;

#[repr(C)]
pub struct lv_color_t {
    blue: u8,
    green: u8,
    red: u8,
}

extern "C" {
    pub fn lv_event_get_code(e: *mut lv_event_t) -> lv_event_code_t;
    pub fn lv_event_get_target(e: *mut lv_event_t) -> *mut c_void;
    pub fn lv_label_set_text(obj: *mut lv_obj_t, text: *const c_char);
    pub fn lv_obj_get_child(obj: *const lv_obj_t, idx: i32) -> *mut lv_obj_t;
    pub fn lv_obj_add_state(obj: *mut lv_obj_t, state: i32);
    pub fn lv_obj_remove_state(obj: *mut lv_obj_t, state: i32);
    pub fn lv_obj_set_style_image_recolor(
        obj: *mut lv_obj_t,
        value: lv_color_t,
        selector: lv_style_selector_t,
    );
    pub fn lv_obj_set_style_image_recolor_opa(
        obj: *mut lv_obj_t,
        value: lv_opa_t,
        selector: lv_style_selector_t,
    );
}
