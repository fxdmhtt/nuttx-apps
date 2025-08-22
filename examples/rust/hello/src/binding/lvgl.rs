#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ffi::{c_char, c_void};

pub type lv_event_cb_t = unsafe extern "C" fn(e: *mut lv_event_t);
pub type lv_event_code_t = u32;
pub type lv_event_dsc_t = c_void;
pub type lv_event_t = c_void;
pub type lv_obj_t = c_void;
pub type lv_obj_flag_t = u32;
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

pub const LV_EVENT_ALL: lv_event_code_t = 0;
pub const LV_EVENT_PRESSED: lv_event_code_t = 1;
pub const LV_EVENT_PRESSING: lv_event_code_t = 2;
pub const LV_EVENT_PRESS_LOST: lv_event_code_t = 3;
pub const LV_EVENT_SHORT_CLICKED: lv_event_code_t = 4;
pub const LV_EVENT_LONG_PRESSED: lv_event_code_t = 5;
pub const LV_EVENT_LONG_PRESSED_REPEAT: lv_event_code_t = 6;
pub const LV_EVENT_CLICKED: lv_event_code_t = 7;
pub const LV_EVENT_RELEASED: lv_event_code_t = 8;
pub const LV_EVENT_SCROLL_BEGIN: lv_event_code_t = 9;
pub const LV_EVENT_SCROLL_THROW_BEGIN: lv_event_code_t = 10;
pub const LV_EVENT_SCROLL_END: lv_event_code_t = 11;
pub const LV_EVENT_SCROLL: lv_event_code_t = 12;
pub const LV_EVENT_GESTURE: lv_event_code_t = 13;
pub const LV_EVENT_KEY: lv_event_code_t = 14;
pub const LV_EVENT_ROTARY: lv_event_code_t = 15;
pub const LV_EVENT_FOCUSED: lv_event_code_t = 16;
pub const LV_EVENT_DEFOCUSED: lv_event_code_t = 17;
pub const LV_EVENT_LEAVE: lv_event_code_t = 18;
pub const LV_EVENT_HIT_TEST: lv_event_code_t = 19;
pub const LV_EVENT_INDEV_RESET: lv_event_code_t = 20;
pub const LV_EVENT_COVER_CHECK: lv_event_code_t = 21;
pub const LV_EVENT_REFR_EXT_DRAW_SIZE: lv_event_code_t = 22;
pub const LV_EVENT_DRAW_MAIN_BEGIN: lv_event_code_t = 23;
pub const LV_EVENT_DRAW_MAIN: lv_event_code_t = 24;
pub const LV_EVENT_DRAW_MAIN_END: lv_event_code_t = 25;
pub const LV_EVENT_DRAW_POST_BEGIN: lv_event_code_t = 26;
pub const LV_EVENT_DRAW_POST: lv_event_code_t = 27;
pub const LV_EVENT_DRAW_POST_END: lv_event_code_t = 28;
pub const LV_EVENT_DRAW_TASK_ADDED: lv_event_code_t = 29;
pub const LV_EVENT_VALUE_CHANGED: lv_event_code_t = 30;
pub const LV_EVENT_INSERT: lv_event_code_t = 31;
pub const LV_EVENT_REFRESH: lv_event_code_t = 32;
pub const LV_EVENT_READY: lv_event_code_t = 33;
pub const LV_EVENT_CANCEL: lv_event_code_t = 34;
pub const LV_EVENT_CREATE: lv_event_code_t = 35;
pub const LV_EVENT_DELETE: lv_event_code_t = 36;
pub const LV_EVENT_CHILD_CHANGED: lv_event_code_t = 37;
pub const LV_EVENT_CHILD_CREATED: lv_event_code_t = 38;
pub const LV_EVENT_CHILD_DELETED: lv_event_code_t = 39;
pub const LV_EVENT_SCREEN_UNLOAD_START: lv_event_code_t = 40;
pub const LV_EVENT_SCREEN_LOAD_START: lv_event_code_t = 41;
pub const LV_EVENT_SCREEN_LOADED: lv_event_code_t = 42;
pub const LV_EVENT_SCREEN_UNLOADED: lv_event_code_t = 43;
pub const LV_EVENT_SIZE_CHANGED: lv_event_code_t = 44;
pub const LV_EVENT_STYLE_CHANGED: lv_event_code_t = 45;
pub const LV_EVENT_LAYOUT_CHANGED: lv_event_code_t = 46;
pub const LV_EVENT_GET_SELF_SIZE: lv_event_code_t = 47;
pub const LV_EVENT_INVALIDATE_AREA: lv_event_code_t = 48;
pub const LV_EVENT_RESOLUTION_CHANGED: lv_event_code_t = 49;
pub const LV_EVENT_COLOR_FORMAT_CHANGED: lv_event_code_t = 50;
pub const LV_EVENT_REFR_REQUEST: lv_event_code_t = 51;
pub const LV_EVENT_REFR_START: lv_event_code_t = 52;
pub const LV_EVENT_REFR_READY: lv_event_code_t = 53;
pub const LV_EVENT_RENDER_START: lv_event_code_t = 54;
pub const LV_EVENT_RENDER_READY: lv_event_code_t = 55;
pub const LV_EVENT_FLUSH_START: lv_event_code_t = 56;
pub const LV_EVENT_FLUSH_FINISH: lv_event_code_t = 57;
pub const LV_EVENT_FLUSH_WAIT_START: lv_event_code_t = 58;
pub const LV_EVENT_FLUSH_WAIT_FINISH: lv_event_code_t = 59;
pub const LV_EVENT_VSYNC: lv_event_code_t = 60;
pub const LV_EVENT_VSYNC_REQUEST: lv_event_code_t = 61;
pub const LV_EVENT_CROWN_SCROLL_BEGIN: lv_event_code_t = 62;
pub const LV_EVENT_CROWN_SCROLL: lv_event_code_t = 63;
pub const LV_EVENT_CROWN_SCROLL_END: lv_event_code_t = 64;
pub const LV_EVENT_CROWN_SCROLL_CHECK: lv_event_code_t = 65;
pub const LV_EVENT_CROWN_SCROLL_CHAIN_CHECK: lv_event_code_t = 66;
pub const LV_EVENT_CROWN_SCROLL_VIBRATION: lv_event_code_t = 67;
pub const _LV_EVENT_LAST: lv_event_code_t = 68;
pub const LV_EVENT_PREPROCESS: lv_event_code_t = 0x8000;

pub const LV_OBJ_FLAG_HIDDEN: lv_obj_flag_t = 1 << 0;
pub const LV_OBJ_FLAG_CLICKABLE: lv_obj_flag_t = 1 << 1;
pub const LV_OBJ_FLAG_CLICK_FOCUSABLE: lv_obj_flag_t = 1 << 2;
pub const LV_OBJ_FLAG_CHECKABLE: lv_obj_flag_t = 1 << 3;
pub const LV_OBJ_FLAG_SCROLLABLE: lv_obj_flag_t = 1 << 4;
pub const LV_OBJ_FLAG_SCROLL_ELASTIC: lv_obj_flag_t = 1 << 5;
pub const LV_OBJ_FLAG_SCROLL_MOMENTUM: lv_obj_flag_t = 1 << 6;
pub const LV_OBJ_FLAG_SCROLL_ONE: lv_obj_flag_t = 1 << 7;
pub const LV_OBJ_FLAG_SCROLL_CHAIN_HOR: lv_obj_flag_t = 1 << 8;
pub const LV_OBJ_FLAG_SCROLL_CHAIN_VER: lv_obj_flag_t = 1 << 9;
pub const LV_OBJ_FLAG_SCROLL_CHAIN: lv_obj_flag_t =
    LV_OBJ_FLAG_SCROLL_CHAIN_HOR | LV_OBJ_FLAG_SCROLL_CHAIN_VER;
pub const LV_OBJ_FLAG_SCROLL_ON_FOCUS: lv_obj_flag_t = 1 << 10;
pub const LV_OBJ_FLAG_SCROLL_WITH_ARROW: lv_obj_flag_t = 1 << 11;
pub const LV_OBJ_FLAG_SNAPPABLE: lv_obj_flag_t = 1 << 12;
pub const LV_OBJ_FLAG_PRESS_LOCK: lv_obj_flag_t = 1 << 13;
pub const LV_OBJ_FLAG_EVENT_BUBBLE: lv_obj_flag_t = 1 << 14;
pub const LV_OBJ_FLAG_GESTURE_BUBBLE: lv_obj_flag_t = 1 << 15;
pub const LV_OBJ_FLAG_ADV_HITTEST: lv_obj_flag_t = 1 << 16;
pub const LV_OBJ_FLAG_IGNORE_LAYOUT: lv_obj_flag_t = 1 << 17;
pub const LV_OBJ_FLAG_FLOATING: lv_obj_flag_t = 1 << 18;
pub const LV_OBJ_FLAG_SEND_DRAW_TASK_EVENTS: lv_obj_flag_t = 1 << 19;
pub const LV_OBJ_FLAG_OVERFLOW_VISIBLE: lv_obj_flag_t = 1 << 20;
pub const LV_OBJ_FLAG_FLEX_IN_NEW_TRACK: lv_obj_flag_t = 1 << 21;
pub const LV_OBJ_FLAG_LAYOUT_1: lv_obj_flag_t = 1 << 23;
pub const LV_OBJ_FLAG_LAYOUT_2: lv_obj_flag_t = 1 << 24;
pub const LV_OBJ_FLAG_WIDGET_1: lv_obj_flag_t = 1 << 25;
pub const LV_OBJ_FLAG_WIDGET_2: lv_obj_flag_t = 1 << 26;
pub const LV_OBJ_FLAG_USER_1: lv_obj_flag_t = 1 << 27;
pub const LV_OBJ_FLAG_USER_2: lv_obj_flag_t = 1 << 28;
pub const LV_OBJ_FLAG_USER_3: lv_obj_flag_t = 1 << 29;
pub const LV_OBJ_FLAG_USER_4: lv_obj_flag_t = 1 << 30;

#[repr(C)]
pub struct lv_color_t {
    blue: u8,
    green: u8,
    red: u8,
}

extern "C" {
    pub fn lv_checkbox_set_text(obj: *mut lv_obj_t, txt: *const c_char);
    pub fn lv_event_get_code(e: *mut lv_event_t) -> lv_event_code_t;
    pub fn lv_event_get_current_target(e: *mut lv_event_t) -> *mut c_void;
    pub fn lv_event_get_target(e: *mut lv_event_t) -> *mut c_void;
    pub fn lv_event_get_user_data(e: *mut lv_event_t) -> *mut c_void;
    pub fn lv_event_dsc_get_cb(dsc: *mut lv_event_dsc_t) -> lv_event_cb_t;
    pub fn lv_event_dsc_get_user_data(dsc: *mut lv_event_dsc_t) -> *mut c_void;
    pub fn lv_label_get_text(obj: *const lv_obj_t) -> *mut c_char;
    pub fn lv_label_set_text(obj: *mut lv_obj_t, text: *const c_char);
    pub fn lv_obj_add_event_cb(
        obj: *mut lv_obj_t,
        event_cb: lv_event_cb_t,
        filter: lv_event_code_t,
        user_data: *mut c_void,
    ) -> *mut lv_event_dsc_t;
    pub fn lv_obj_delete(obj: *mut lv_obj_t);
    pub fn lv_obj_get_child(obj: *const lv_obj_t, idx: i32) -> *mut lv_obj_t;
    pub fn lv_obj_get_child_count(obj: *const lv_obj_t) -> u32;
    pub fn lv_obj_get_event_count(obj: *mut lv_obj_t) -> u32;
    pub fn lv_obj_get_event_dsc(obj: *mut lv_obj_t, index: u32) -> *mut lv_event_dsc_t;
    pub fn lv_obj_remove_event_cb(obj: *mut lv_obj_t, event_cb: lv_event_cb_t) -> bool;
    pub fn lv_obj_remove_event_cb_with_user_data(
        obj: *mut lv_obj_t,
        event_cb: lv_event_cb_t,
        user_data: *mut c_void,
    ) -> u32;
    pub fn lv_obj_remove_event_dsc(obj: *mut lv_obj_t, dsc: *mut lv_event_dsc_t) -> bool;
    pub fn lv_obj_add_flag(obj: *mut lv_obj_t, f: lv_obj_flag_t);
    pub fn lv_obj_remove_flag(obj: *mut lv_obj_t, f: lv_obj_flag_t);
    pub fn lv_obj_update_flag(obj: *mut lv_obj_t, f: lv_obj_flag_t, v: bool);
    pub fn lv_obj_add_state(obj: *mut lv_obj_t, state: lv_state_t);
    pub fn lv_obj_remove_state(obj: *mut lv_obj_t, state: lv_state_t);
    pub fn lv_obj_set_state(obj: *mut lv_obj_t, state: lv_state_t, v: bool);
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
