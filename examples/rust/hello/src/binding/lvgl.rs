#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ffi::{c_char, c_void};

pub type lv_event_code_t = u32;
pub type lv_event_t = c_void;
pub type lv_obj_t = c_void;
pub type lv_opa_t = u8;
pub type lv_state_t = u16;
pub type lv_style_selector_t = u32;
pub type lv_palette_t = u32;

pub const LV_STATE_DEFAULT: lv_state_t = 0x0000;
pub const LV_STATE_CHECKED: lv_state_t = 0x0001;
pub const LV_STATE_FOCUSED: lv_state_t = 0x0002;
pub const LV_STATE_FOCUS_KEY: lv_state_t = 0x0004;
pub const LV_STATE_EDITED: lv_state_t = 0x0008;
pub const LV_STATE_HOVERED: lv_state_t = 0x0010;
pub const LV_STATE_PRESSED: lv_state_t = 0x0020;
pub const LV_STATE_SCROLLED: lv_state_t = 0x0040;
pub const LV_STATE_DISABLED: lv_state_t = 0x0080;
pub const LV_STATE_USER_1: lv_state_t = 0x1000;
pub const LV_STATE_USER_2: lv_state_t = 0x2000;
pub const LV_STATE_USER_3: lv_state_t = 0x4000;
pub const LV_STATE_USER_4: lv_state_t = 0x8000;
pub const LV_STATE_ANY: lv_state_t = 0xFFFF;

pub const LV_PART_MAIN: lv_style_selector_t = 0x000000;
pub const LV_PART_SCROLLBAR: lv_style_selector_t = 0x010000;
pub const LV_PART_INDICATOR: lv_style_selector_t = 0x020000;
pub const LV_PART_KNOB: lv_style_selector_t = 0x030000;
pub const LV_PART_SELECTED: lv_style_selector_t = 0x040000;
pub const LV_PART_ITEMS: lv_style_selector_t = 0x050000;
pub const LV_PART_CURSOR: lv_style_selector_t = 0x060000;
pub const LV_PART_TICKS: lv_style_selector_t = 0x070000;
pub const LV_PART_CUSTOM_FIRST: lv_style_selector_t = 0x080000;
pub const LV_PART_ANY: lv_style_selector_t = 0x0F0000;

pub const LV_PALETTE_RED: lv_palette_t = 1;
pub const LV_PALETTE_PINK: lv_palette_t = 2;
pub const LV_PALETTE_PURPLE: lv_palette_t = 3;
pub const LV_PALETTE_DEEP_PURPLE: lv_palette_t = 4;
pub const LV_PALETTE_INDIGO: lv_palette_t = 5;
pub const LV_PALETTE_BLUE: lv_palette_t = 6;
pub const LV_PALETTE_LIGHT_BLUE: lv_palette_t = 7;
pub const LV_PALETTE_CYAN: lv_palette_t = 8;
pub const LV_PALETTE_TEAL: lv_palette_t = 9;
pub const LV_PALETTE_GREEN: lv_palette_t = 10;
pub const LV_PALETTE_LIGHT_GREEN: lv_palette_t = 11;
pub const LV_PALETTE_LIME: lv_palette_t = 12;
pub const LV_PALETTE_YELLOW: lv_palette_t = 13;
pub const LV_PALETTE_AMBER: lv_palette_t = 14;
pub const LV_PALETTE_ORANGE: lv_palette_t = 15;
pub const LV_PALETTE_DEEP_ORANGE: lv_palette_t = 16;
pub const LV_PALETTE_BROWN: lv_palette_t = 17;
pub const LV_PALETTE_BLUE_GREY: lv_palette_t = 18;
pub const LV_PALETTE_GREY: lv_palette_t = 19;
pub const _LV_PALETTE_LAST: lv_palette_t = 20;
pub const LV_PALETTE_NONE: lv_palette_t = 0xff;

pub const LV_OPA_TRANSP: lv_opa_t = 0;
pub const LV_OPA_0: lv_opa_t = 0;
pub const LV_OPA_10: lv_opa_t = 25;
pub const LV_OPA_20: lv_opa_t = 51;
pub const LV_OPA_30: lv_opa_t = 76;
pub const LV_OPA_40: lv_opa_t = 102;
pub const LV_OPA_50: lv_opa_t = 127;
pub const LV_OPA_60: lv_opa_t = 153;
pub const LV_OPA_70: lv_opa_t = 178;
pub const LV_OPA_80: lv_opa_t = 204;
pub const LV_OPA_90: lv_opa_t = 229;
pub const LV_OPA_100: lv_opa_t = 255;
pub const LV_OPA_COVER: lv_opa_t = 255;

#[repr(C)]
pub struct lv_color_t {
    blue: u8,
    green: u8,
    red: u8,
}

extern "C" {
    pub fn lv_checkbox_set_text(obj: *mut lv_obj_t, txt: *const c_char);
    pub fn lv_event_get_code(e: *mut lv_event_t) -> lv_event_code_t;
    pub fn lv_event_get_target(e: *mut lv_event_t) -> *mut c_void;
    pub fn lv_label_set_text(obj: *mut lv_obj_t, text: *const c_char);
    pub fn lv_obj_get_child(obj: *const lv_obj_t, idx: i32) -> *mut lv_obj_t;
    pub fn lv_obj_get_child_count(obj: *const lv_obj_t) -> u32;
    pub fn lv_obj_delete(obj: *mut lv_obj_t);
    pub fn lv_obj_add_state(obj: *mut lv_obj_t, state: lv_state_t);
    pub fn lv_obj_remove_state(obj: *mut lv_obj_t, state: lv_state_t);
    pub fn lv_obj_set_style_opa(obj: *mut lv_obj_t, value: lv_opa_t, selector: lv_style_selector_t);
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
    pub fn lv_obj_set_style_bg_color(
        obj: *mut lv_obj_t,
        value: lv_color_t,
        selector: lv_style_selector_t,
    );
    pub fn lv_palette_main(p: lv_palette_t) -> lv_color_t;
}
