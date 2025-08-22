#![allow(static_mut_refs)]

use std::cell::RefCell;
use std::ffi::{c_char, CStr};
use std::rc::Rc;
use std::time::Duration;

use async_executor::Task;
use futures::future::LocalBoxFuture;
use futures::stream::FuturesUnordered;
use futures::{pin_mut, select, stream, FutureExt, StreamExt, TryStreamExt};
use itertools_num::linspace;
use reactive_cache::{effect, memo, ref_signal, signal, IEffect, Lazy};
use stack_cstr::cstr;

use crate::binding::lvgl::*;
use crate::runtime::cancelled::CancellationTokenSource;
use crate::runtime::delay::{delay, Delay};
use crate::runtime::{event, TaskRun};
use crate::*;

extern "C" {
    static mut radio_cont: *mut lv_obj_t;
    static mut img: *mut lv_obj_t;
    static mut img_label: *mut lv_obj_t;
    static mut btn1: *mut lv_obj_t;
    static mut btn2: *mut lv_obj_t;
    static mut no_color_btn: *mut lv_obj_t;
    static mut list: *mut lv_obj_t;

    fn lv_color_make_rs(r: u8, g: u8, b: u8) -> lv_color_t;
    fn create_list_item(parent: *mut lv_obj_t, text: *const c_char) -> *mut lv_obj_t;
    fn create_list_hint() -> *mut lv_obj_t;
}

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

signal!(
    static mut INTENSE: u8 = 0x7f;
);

signal!(
    static mut RECOLOR_ANIMATION: bool = false;
);

signal!(
    static mut LIST_ITEM_COUNT: u32 = 0;
);

#[no_mangle]
extern "C" fn active_index_get() -> i32 {
    ACTIVE_INDEX()
}

#[no_mangle]
extern "C" fn active_index_set(value: i32) -> bool {
    ACTIVE_INDEX_set(value)
}

#[memo]
fn state() -> State {
    match ACTIVE_INDEX() {
        0 => Some(Color::Red),
        1 => Some(Color::Green),
        2 => Some(Color::Blue),
        3 => Some(Color::Yellow),
        4 => None,
        _ => unreachable!(),
    }
}

event_decl!(switch_color_event, {
    let id = ACTIVE_INDEX() + 1;
    let id = if RECOLOR_ANIMATION() && id == 4 {
        id + 1
    } else {
        id
    };
    ACTIVE_INDEX_set(id % 5);
});

static mut TASKS: Lazy<FuturesUnordered<LocalBoxFuture<()>>> = Lazy::new(FuturesUnordered::new);
static mut CTS: Lazy<RefCell<CancellationTokenSource>> =
    Lazy::new(|| RefCell::new(CancellationTokenSource::new()));

fn tasks_cleanup_in_background() -> Task<()> {
    TaskRun(async move {
        let mut id = 0;
        loop {
            if (unsafe { TASKS.next() }.await).is_some() {
                println!("A mission completed! id: {id}");
                id += 1;
            } else {
                delay(1).await;
            }
        }
    })
}

async fn list_item_fade(obj: *mut lv_obj_t, cnt: usize) {
    let token = unsafe { CTS.borrow() }.token();

    let _ = stream::iter(linspace(255.0, 0.0, cnt).map(|x: f32| x.round() as u8))
        .map(Ok)
        .try_for_each(|x| {
            let token = token.clone();
            async move {
                unsafe { lv_obj_set_style_opa(obj, x, LV_PART_MAIN) };
                Delay::new(Duration::from_millis(100)).await;
                token.check_cancelled()
            }
        })
        .await;
}

async fn intense_animation(target: u8, duration: Duration) {
    let delay = Duration::from_millis(100);
    let ticks = duration.div_duration_f32(delay) as i16;
    let start = INTENSE();

    let header = if target > start {
        "Increase color density"
    } else {
        "Decrease color density"
    };
    let text = cstr!("{header} - {start}");
    let lbl = unsafe { create_list_item(list, text.as_ptr()) };

    RECOLOR_ANIMATION_set(true);

    stream::iter(
        linspace(start as f32, target as f32, ticks as usize).map(|x: f32| x.round() as u8),
    )
    .for_each(|cur| async move {
        INTENSE_set(cur);

        let text = cstr!("{header} - {cur}");
        unsafe { lv_checkbox_set_text(lbl, text.as_ptr()) };

        Delay::new(delay).await;
    })
    .await;
    INTENSE_set(target);

    RECOLOR_ANIMATION_set(false);

    list_item_fade(lbl, 15).await;
    unsafe { lv_obj_delete(lbl) };
}

event_decl!(intense_inc_event, async {
    let intense = match INTENSE() {
        0 => Some(0x7f),
        0x7f => Some(0xff),
        0xff => None,
        _ => unreachable!(),
    };
    if let Some(intense) = intense {
        intense_animation(intense, Duration::from_secs(5)).await;
    }
});

event_decl!(intense_dec_or_clear_event, async {
    match state() {
        Some(_) => {
            let intense = match INTENSE() {
                0 => None,
                0x7f => Some(0),
                0xff => Some(0x7f),
                _ => unreachable!(),
            };
            if let Some(intense) = intense {
                intense_animation(intense, Duration::from_secs(5)).await;
            }
        }
        None => {
            let cts = unsafe { CTS.replace(CancellationTokenSource::new()) };
            assert!(!cts.is_cancelled());
            cts.cancel();
        }
    };
});

event_decl!(list_item_changed_event, e, {
    let obj = unsafe { lv_event_get_target(e) };
    let cnt = unsafe { lv_obj_get_child_count(obj) };
    LIST_ITEM_COUNT_set(cnt);
});

static mut EFFECTS: Lazy<Vec<Rc<dyn IEffect>>> = Lazy::new(|| {
    vec![
        effect!(|| {
            let id = ACTIVE_INDEX();
            (0..5)
                .map(|x| match x {
                    _ if x == id => lv_obj_add_state,
                    _ => lv_obj_remove_state,
                })
                .zip(0..5)
                .for_each(|(f, id)| unsafe {
                    f(lv_obj_get_child(radio_cont, id), LV_STATE_CHECKED)
                });
        }),
        BindingImageRecolor!(unsafe { img }, {
            match state() {
                Some(color) => match color {
                    Color::Red => unsafe { lv_color_make_rs(0xff, 0, 0) },
                    Color::Green => unsafe { lv_color_make_rs(0, 0xff, 0) },
                    Color::Blue => unsafe { lv_color_make_rs(0, 0, 0xff) },
                    Color::Yellow => unsafe { lv_color_make_rs(0xff, 0xff, 0) },
                },
                None => unsafe { lv_color_make_rs(0, 0, 0) },
            }
        }),
        BindingImageRecolorOpa!(unsafe { img }, {
            match (state(), INTENSE()) {
                (Some(_), intense) => intense,
                (None, _) => 0,
            }
        }),
        BindingText!(unsafe { img_label }, {
            state()
                .map(|c| cstr!("Color {c:?}"))
                .unwrap_or(cstr!("Original Color"))
        }),
        effect!(|| {
            match (state(), RECOLOR_ANIMATION()) {
                (_, true) => unsafe { lv_obj_add_state(btn1, LV_STATE_DISABLED) },
                (None, _) => unsafe { lv_obj_add_state(btn1, LV_STATE_DISABLED) },
                _ => unsafe { lv_obj_remove_state(btn1, LV_STATE_DISABLED) },
            };
        }),
        effect!(|| {
            match (state(), RECOLOR_ANIMATION()) {
                (_, true) => unsafe { lv_obj_add_state(btn2, LV_STATE_DISABLED) },
                (None, _) if LIST_ITEM_COUNT() == 0 => unsafe {
                    lv_obj_add_state(btn2, LV_STATE_DISABLED)
                },
                _ => unsafe { lv_obj_remove_state(btn2, LV_STATE_DISABLED) },
            };
        }),
        BindingText!(unsafe { lv_obj_get_child(btn2, 0) }, {
            match state() {
                Some(_) => cstr!("Intense Dec"),
                None => cstr!("Clear Log"),
            }
        }),
        BindingBgColor!(unsafe { btn2 }, {
            match state() {
                Some(_) => unsafe { lv_palette_main(LV_PALETTE_BLUE) },
                None => unsafe { lv_palette_main(LV_PALETTE_RED) },
            }
        }),
        effect!(|| {
            let btns = unsafe { [no_color_btn] };
            match RECOLOR_ANIMATION() {
                true => btns
                    .iter()
                    .for_each(|btn| unsafe { lv_obj_add_state(*btn, LV_STATE_DISABLED) }),
                false => btns
                    .iter()
                    .for_each(|btn| unsafe { lv_obj_remove_state(*btn, LV_STATE_DISABLED) }),
            }
        }),
        effect!(|| {
            let text = state()
                .map(|c| cstr!("Recolor to {c:?}"))
                .unwrap_or(cstr!("Non Recolor!"));
            let lbl = unsafe { create_list_item(list, text.as_ptr()) };

            {
                let item = lbl;
                let evt1 = event::add(lbl, LV_EVENT_SHORT_CLICKED, move |e| {
                    let obj = unsafe { lv_event_get_target(e) };
                    assert_eq!(obj, item);
                    let text = unsafe { CStr::from_ptr(lv_label_get_text(obj)) };
                    println!("{text:?} Clicked!");
                });
                let evt2 = event::add(lbl, LV_EVENT_SHORT_CLICKED, |_| {});

                assert_eq!(unsafe { lv_obj_get_event_count(lbl) }, 3);
                assert_eq!(
                    unsafe { lv_event_dsc_get_user_data(lv_obj_get_event_dsc(lbl, 0)) },
                    std::ptr::null_mut()
                );
                assert_eq!(unsafe { lv_obj_get_event_dsc(lbl, 1) }, evt1);
                assert_eq!(unsafe { lv_obj_get_event_dsc(lbl, 2) }, evt2);
                assert!(std::ptr::fn_addr_eq(
                    unsafe { lv_event_dsc_get_cb(lv_obj_get_event_dsc(lbl, 1)) },
                    unsafe { lv_event_dsc_get_cb(lv_obj_get_event_dsc(lbl, 2)) }
                ));

                assert!(event::remove(lbl, evt2));
            }

            let token = unsafe { CTS.borrow() }.token();
            let task = TaskRun(async move {
                let delay = delay(5).fuse();
                let cancelled = token.cancelled().fuse();
                pin_mut!(delay, cancelled);

                select! {
                    _ = delay => { list_item_fade(lbl, 10).await; },
                    _ = cancelled => {},
                }
                unsafe { lv_obj_delete(lbl) };
            });
            unsafe { TASKS.push(task.boxed_local()) };
        }),
        effect!(|| {
            static mut HINT: Option<*mut lv_obj_t> = None;
            match LIST_ITEM_COUNT() {
                0 => {
                    if unsafe { HINT }.is_none() {
                        unsafe { HINT.replace(create_list_hint()) };
                    }
                }
                _ => {
                    if let Some(obj) = unsafe { HINT.take() } {
                        unsafe { lv_obj_delete(obj) };
                    }
                }
            };
        }),
    ]
});

#[no_mangle]
extern "C" fn frp_demo_rs_init() {
    Lazy::force(unsafe { &EFFECTS });

    tasks_cleanup_in_background().detach();
}
