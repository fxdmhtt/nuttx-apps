#![allow(static_mut_refs)]

use std::{ffi::CString, rc::Rc};

use reactive_cache::{effect, memo, signal, IEffect, Lazy};

use crate::binding::lvgl::*;
use crate::event;

extern "C" {
    static mut radio_cont: *mut lv_obj_t;
    static mut img: *mut lv_obj_t;
    static mut img_label: *mut lv_obj_t;

    fn lv_color_make_rs(r: u8, g: u8, b: u8) -> lv_color_t;
}

static mut EFFECTS: Lazy<Vec<Rc<dyn IEffect>>> = Lazy::new(|| {
    vec![
        effect!(|| {
            let id = active_index_get();
            (0..5).filter(|x| *x != id).for_each(|id| unsafe {
                lv_obj_remove_state(lv_obj_get_child(radio_cont, id), LV_STATE_CHECKED)
            });

            unsafe { lv_obj_add_state(lv_obj_get_child(radio_cont, id), LV_STATE_CHECKED) };
        }),
        effect!(|| {
            let color = match state() {
                Some(color) => {
                    unsafe { lv_obj_set_style_image_recolor_opa(img, 0x7f, 0) };
                    match color {
                        Color::Red => unsafe { lv_color_make_rs(0xff, 0, 0) },
                        Color::Green => unsafe { lv_color_make_rs(0, 0xff, 0) },
                        Color::Blue => unsafe { lv_color_make_rs(0, 0, 0xff) },
                        Color::Yellow => unsafe { lv_color_make_rs(0xff, 0xff, 0) },
                    }
                }
                None => {
                    unsafe { lv_obj_set_style_image_recolor_opa(img, 0, 0) };
                    unsafe { lv_color_make_rs(0, 0, 0) }
                }
            };
            unsafe { lv_obj_set_style_image_recolor(img, color, 0) };
        }),
        effect!(|| {
            let color = state()
                .map(|c| format!("Color {c:?}"))
                .unwrap_or("Original Color".to_string());
            unsafe {
                lv_label_set_text(img_label, CString::new(color).unwrap().as_ptr() as *const _)
            };
        }),
    ]
});

#[derive(Debug, Copy, Clone)]
enum Color {
    Red,
    Green,
    Blue,
    Yellow,
}

type State = Option<Color>;

signal!(
    static mut ACTIVE_INDEX: i32 = 4;
);

#[no_mangle]
pub extern "C" fn active_index_get() -> i32 {
    *ACTIVE_INDEX_get()
}

#[no_mangle]
pub extern "C" fn active_index_set(value: i32) -> bool {
    ACTIVE_INDEX_set(value)
}

#[memo]
fn state() -> State {
    match active_index_get() {
        0 => Some(Color::Red),
        1 => Some(Color::Green),
        2 => Some(Color::Blue),
        3 => Some(Color::Yellow),
        4 => None,
        _ => unreachable!(),
    }
}

event!(switch_color_event, {
    active_index_set((active_index_get() + 1) % 5);
});

#[no_mangle]
pub extern "C" fn frp_demo_rs_init() {
    Lazy::force(unsafe { &EFFECTS });
}
