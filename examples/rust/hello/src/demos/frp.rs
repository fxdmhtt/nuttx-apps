#![allow(static_mut_refs)]

use std::cell::RefCell;
use std::ffi::{c_char, CStr};
use std::rc::Rc;
use std::time::Duration;

use async_cancellation_token::{CancellationTokenSource, Cancelled};
use async_executor::Task;
use futures::{pin_mut, select, stream, FutureExt, StreamExt, TryStreamExt};
use itertools_num::linspace;
use reactive_cache::{effect, prelude::*};
use stack_cstr::cstr;
use thiserror::Error;

use crate::{
    runtime::{lvgl::*, *},
    *,
};

extern "C" {
    fn rust_executor_wake();
}

extern "C" {
    static mut _radio_cont: *mut lv_obj_t;
    static mut _img: *mut lv_obj_t;
    static mut _img_label: *mut lv_obj_t;
    static mut _btn1: *mut lv_obj_t;
    static mut _btn2: *mut lv_obj_t;
    static mut _no_color_btn: *mut lv_obj_t;
    static mut _list: *mut lv_obj_t;
    static mut _slider: *mut lv_obj_t;

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

#[derive(Error, Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
#[error("not running")]
struct NotRunning;

#[derive(Error, Debug)]
enum AppError {
    #[error(transparent)]
    NotRunning(#[from] NotRunning),
    #[error(transparent)]
    CancelError(#[from] Cancelled),
    #[error(transparent)]
    LVGLError(#[from] LVGLError),
}

struct ViewModel {
    tasks: TaskManager,
    _bg_task: Task<()>,

    active_index: Rc<Signal<i32>>,
    intense: Rc<Signal<lv_opa_t>>,
    recolor_animation: Rc<Signal<bool>>,
    list_item_count: Rc<Signal<u32>>,

    state: Rc<Memo<State>>,

    effects: RefCell<Vec<Rc<Effect>>>,

    cts_fade: RefCell<CancellationTokenSource>,
    cts_anim: RefCell<CancellationTokenSource>,

    hint: RefCell<LvObjHandle>,

    radio_cont: RefCell<LvObjHandle>,
    img: RefCell<LvObjHandle>,
    img_label: RefCell<LvObjHandle>,
    btn1: RefCell<LvObjHandle>,
    btn2: RefCell<LvObjHandle>,
    no_color_btn: RefCell<LvObjHandle>,
    list: RefCell<LvObjHandle>,
    slider: RefCell<LvObjHandle>,
}

impl Drop for ViewModel {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            debug_assert_eq!(Rc::strong_count(&self.active_index), 1);
            debug_assert_eq!(Rc::strong_count(&self.intense), 1);
            debug_assert_eq!(Rc::strong_count(&self.recolor_animation), 1);
            debug_assert_eq!(Rc::strong_count(&self.list_item_count), 1);
            debug_assert_eq!(Rc::strong_count(&self.state), 1);
            self.effects
                .borrow()
                .iter()
                .map(Rc::strong_count)
                .for_each(|c| debug_assert_eq!(c, 1));
        }
    }
}

impl ViewModel {
    fn new() -> Self {
        let tasks = TaskManager::new();
        let _bg_task = TaskRun(async move {
            loop {
                if let Ok(vm) = vm() {
                    println!("{} tasks remaining!", vm.tasks.gc());
                }

                let _ = delay!(Duration::from_millis(1000)).await;
            }
        });

        let active_index = Signal::new(4);
        let intense = Signal::new(0x7f);
        let recolor_animation = Signal::new(false);
        let list_item_count = Signal::new(0);

        let state = {
            downgrade!(active_index);
            Memo::new(move || match *active_index.upgrade().unwrap().get() {
                0 => Some(Color::Red),
                1 => Some(Color::Green),
                2 => Some(Color::Blue),
                3 => Some(Color::Yellow),
                4 => None,
                _ => unreachable!(),
            })
        };

        // This is a rather cumbersome approach, requiring that all `Effect`s created here can only directly
        // reference member variables within the current environment, and cannot indirectly reference `ViewModel`.
        // This is because `ViewModel` has not yet been created, and when collecting dependencies on the `Effect`,
        // the reference to `ViewModel` will be invalid. Therefore, the `Effect`s defined here must not have
        // indirect references to `ViewModel`, such as `vm()`.
        //
        // Also, the best practice for referencing variables within `Effect`s is `Weak::downgrade`, but even with
        // the help of macros, this can make the code tedious.
        //
        // The simplest approach is to not define `Effect` here. Instead, after `ViewModel` is created, create `Effect`
        // by indirectly referencing `ViewModel` and add it to `self.effects`. An example of this is provided in the code.
        //
        // Defining Effect here is for example purposes only and is not a best practice.
        let effects = vec![];

        Self {
            tasks,
            _bg_task,

            active_index,
            intense,
            recolor_animation,
            list_item_count,

            state,

            effects: effects.into(),

            cts_fade: Default::default(),
            cts_anim: Default::default(),

            hint: Default::default(),

            radio_cont: Default::default(),
            img: Default::default(),
            img_label: Default::default(),
            btn1: Default::default(),
            btn2: Default::default(),
            no_color_btn: Default::default(),
            list: Default::default(),
            slider: Default::default(),
        }
    }
}

static mut VM: Option<Box<ViewModel>> = None;

fn vm() -> Result<&'static ViewModel, NotRunning> {
    (unsafe { VM.as_deref() }).ok_or(NotRunning)
}

#[no_mangle]
extern "C" fn active_index_get() -> i32 {
    *vm().unwrap().active_index.get()
}

#[no_mangle]
extern "C" fn active_index_set(value: i32) -> bool {
    vm().unwrap().active_index.set(value)
}

event_decl!(switch_color_event, {
    let id = *vm().unwrap().active_index.get() + 1;
    let id = if *vm().unwrap().recolor_animation.get() && id == 4 {
        id + 1
    } else {
        id
    };
    vm().unwrap().active_index.set(id % 5);
});

fn cts_cancel_and_renew(cts: &RefCell<CancellationTokenSource>) {
    let old = cts.replace(CancellationTokenSource::new());
    debug_assert!(!old.is_cancelled());
    old.cancel();
    unsafe { rust_executor_wake() }; // necessary!
}

async fn list_item_fade(obj: &LvObjHandle, cnt: usize) -> Result<(), AppError> {
    let token = vm()?.cts_fade.borrow().token();

    stream::iter(linspace(255.0, 0.0, cnt).map(|x: f32| x.round() as u8))
        .map(Ok)
        .try_for_each(|x| {
            let token = token.clone();
            let obj = obj.clone();
            async move {
                vm()?;
                unsafe { lv_obj_set_style_opa(obj.try_get()?, x, LV_PART_MAIN) };
                delay!(Duration::from_millis(100), token)
                    .await
                    .map_err(Into::into)
            }
        })
        .await
}

async fn intense_animation(target: u8, duration: Duration) -> Result<(), NotRunning> {
    let token = vm()?.cts_anim.borrow().token();

    let delay_anim = Duration::from_millis(100);
    let ticks = duration.div_duration_f32(delay_anim) as i16;
    let start = *vm()?.intense.get();

    let header = if target > start {
        "Increase color density"
    } else {
        "Decrease color density"
    };
    let text = cstr!("{header} - {start}");
    let lbl = LvObj::from(unsafe {
        create_list_item(
            vm()?.list.borrow().try_get().map_err(|_| NotRunning)?,
            text.as_ptr(),
        )
    });

    vm()?.recolor_animation.set(true);
    stream::iter(linspace(start as f32, target as f32, ticks as usize).map(|x: f32| x.round() as u8))
        .map(Ok)
        .try_for_each(|cur| {
            let token = token.clone();
            let lbl = lbl.clone();

            async move {
                vm()?.intense.set(cur);

                let text = cstr!("{header} - {cur}");
                unsafe { lv_checkbox_set_text(lbl.try_get()?, text.as_ptr()) };

                delay!(delay_anim, token).await?;

                Ok::<_, AppError>(())
            }
        })
        .await
        .inspect(|_| {
            if let Ok(vm) = vm() {
                vm.intense.set(target);
            }
        })
        .or_else(|e| match e {
            AppError::NotRunning(_) => Err(NotRunning),
            AppError::LVGLError(_) => Err(NotRunning),
            AppError::CancelError(_) => Ok(()),
        })?;
    vm()?.recolor_animation.set(false);

    let token = vm()?.cts_anim.borrow().token();
    let _ = delay!(1, token).await;
    let _ = list_item_fade(&lbl, 15).await;

    vm()?;
    unsafe { lv_obj_delete(lbl.try_get().map_err(|_| NotRunning)?) };

    Ok(())
}

event_decl!(intense_inc_event, async {
    if *vm()?.intense.get() < 0xff {
        return intense_animation(0xff, Duration::from_secs(5)).await;
    }
    Ok(())
});

event_decl!(intense_dec_or_clear_event, async {
    match vm()?.state.get() {
        Some(_) => {
            if *vm()?.intense.get() > 0 {
                return intense_animation(0, Duration::from_secs(5)).await;
            }
        }
        None => {
            cts_cancel_and_renew(&vm()?.cts_fade);
        }
    };
    Ok(())
});

event_decl!(list_item_changed_event, e, {
    let obj = unsafe { lv_event_get_target(e) };
    let cnt = unsafe { lv_obj_get_child_count(obj) };
    vm().unwrap().list_item_count.set(cnt);
});

#[no_mangle]
extern "C" fn frp_demo_rs_drop() {
    vm().unwrap().cts_fade.borrow_mut().cancel();
    vm().unwrap().cts_anim.borrow_mut().cancel();
    executor().try_tick_all(); // necessary!

    vm().unwrap().tasks.cancel_all(); // unnecessary

    let weak_active_index = Rc::downgrade(&vm().unwrap().active_index);
    let weak_intense = Rc::downgrade(&vm().unwrap().intense);
    let weak_recolor_animation = Rc::downgrade(&vm().unwrap().recolor_animation);
    let weak_list_item_count = Rc::downgrade(&vm().unwrap().list_item_count);
    let weak_state = Rc::downgrade(&vm().unwrap().state);
    let weak_effects = vm()
        .unwrap()
        .effects
        .borrow()
        .iter()
        .map(Rc::downgrade)
        .collect::<Vec<_>>();

    drop(unsafe { &mut VM }.take().unwrap());

    debug_assert!(weak_active_index.upgrade().is_none());
    debug_assert!(weak_intense.upgrade().is_none());
    debug_assert!(weak_recolor_animation.upgrade().is_none());
    debug_assert!(weak_list_item_count.upgrade().is_none());
    debug_assert!(weak_state.upgrade().is_none());
    debug_assert!(weak_effects.iter().all(|w| w.upgrade().is_none()));
}

#[no_mangle]
extern "C" fn frp_demo_rs_init() {
    assert!(unsafe { &mut VM }
        .replace(ViewModel::new().into())
        .is_none());

    *vm().unwrap().radio_cont.borrow_mut() = LvObj::from(unsafe { _radio_cont });
    *vm().unwrap().img.borrow_mut() = LvObj::from(unsafe { _img });
    *vm().unwrap().img_label.borrow_mut() = LvObj::from(unsafe { _img_label });
    *vm().unwrap().btn1.borrow_mut() = LvObj::from(unsafe { _btn1 });
    *vm().unwrap().btn2.borrow_mut() = LvObj::from(unsafe { _btn2 });
    *vm().unwrap().no_color_btn.borrow_mut() = LvObj::from(unsafe { _no_color_btn });
    *vm().unwrap().list.borrow_mut() = LvObj::from(unsafe { _list });
    *vm().unwrap().slider.borrow_mut() = LvObj::from(unsafe { _slider });

    vm().unwrap().effects.borrow_mut().extend(vec![
        effect!(|| {
            let radio_cont = match vm().unwrap().radio_cont.borrow().try_get() {
                Ok(obj) => obj,
                Err(_) => return,
            };
            let id = *vm().unwrap().active_index.get();
            (0..5)
                .map(|x| match x {
                    _ if x == id => lv_obj_add_state,
                    _ => lv_obj_remove_state,
                })
                .zip(0..5)
                .for_each(|(f, id)| unsafe { f(lv_obj_get_child(radio_cont, id), LV_STATE_CHECKED) });
        }),
        BindingSliderValue!(
            vm().unwrap().slider.borrow(),
            vm().unwrap().intense,
            ConvertBack | v | {
                cts_cancel_and_renew(&vm().unwrap().cts_anim);
                v
            }
        ),
        effect!(|| {
            let slider = match vm().unwrap().slider.borrow().try_get() {
                Ok(obj) => obj,
                Err(_) => return,
            };
            unsafe {
                lv_obj_update_flag(
                    slider,
                    LV_OBJ_FLAG_HIDDEN,
                    vm().unwrap().state.get().is_none(),
                );
            };
        }),
        BindingImageRecolor!(vm().unwrap().img.borrow(), {
            match vm().unwrap().state.get() {
                Some(color) => match color {
                    Color::Red => unsafe { lv_color_make_rs(0xff, 0, 0) },
                    Color::Green => unsafe { lv_color_make_rs(0, 0xff, 0) },
                    Color::Blue => unsafe { lv_color_make_rs(0, 0, 0xff) },
                    Color::Yellow => unsafe { lv_color_make_rs(0xff, 0xff, 0) },
                },
                None => unsafe { lv_color_make_rs(0, 0, 0) },
            }
        }),
        BindingImageRecolorOpa!(vm().unwrap().img.borrow(), {
            match (vm().unwrap().state.get(), *vm().unwrap().intense.get()) {
                (Some(_), intense) => intense,
                (None, _) => 0,
            }
        }),
        BindingText!(vm().unwrap().img_label.borrow(), {
            vm().unwrap()
                .state
                .get()
                .map(|c| cstr!("Color {c:?}"))
                .unwrap_or(cstr!("Original Color"))
        }),
        effect!(|| {
            let btn1 = match vm().unwrap().btn1.borrow().try_get() {
                Ok(obj) => obj,
                Err(_) => return,
            };
            match (
                vm().unwrap().state.get(),
                *vm().unwrap().recolor_animation.get(),
                *vm().unwrap().intense.get(),
            ) {
                (_, true, _) => unsafe { lv_obj_add_state(btn1, LV_STATE_DISABLED) },
                (None, _, _) => unsafe { lv_obj_add_state(btn1, LV_STATE_DISABLED) },
                (Some(_), _, 0xff) => unsafe { lv_obj_add_state(btn1, LV_STATE_DISABLED) },
                _ => unsafe { lv_obj_remove_state(btn1, LV_STATE_DISABLED) },
            };
        }),
        effect!(|| {
            let btn2 = match vm().unwrap().btn2.borrow().try_get() {
                Ok(obj) => obj,
                Err(_) => return,
            };
            match (
                vm().unwrap().state.get(),
                *vm().unwrap().recolor_animation.get(),
                *vm().unwrap().intense.get(),
            ) {
                (_, true, _) => unsafe { lv_obj_add_state(btn2, LV_STATE_DISABLED) },
                (None, _, _) if *vm().unwrap().list_item_count.get() == 0 => unsafe { lv_obj_add_state(btn2, LV_STATE_DISABLED) },
                (Some(_), _, 0) => unsafe { lv_obj_add_state(btn2, LV_STATE_DISABLED) },
                _ => unsafe { lv_obj_remove_state(btn2, LV_STATE_DISABLED) },
            };
        }),
        BindingText!(
            match vm().unwrap().btn2.borrow().try_get() {
                Ok(obj) => LvObj::from(unsafe { lv_obj_get_child(obj, 0) }),
                Err(_) => return,
            },
            {
                match vm().unwrap().state.get() {
                    Some(_) => cstr!("Intense Dec"),
                    None => cstr!("Clear Log"),
                }
            }
        ),
        BindingBgColor!(vm().unwrap().btn2.borrow(), {
            match vm().unwrap().state.get() {
                Some(_) => unsafe { lv_palette_main(LV_PALETTE_BLUE) },
                None => unsafe { lv_palette_main(LV_PALETTE_RED) },
            }
        }),
        effect!(|| {
            let btns = [vm().unwrap().no_color_btn.borrow().try_get().unwrap()];
            match *vm().unwrap().recolor_animation.get() {
                true => btns
                    .iter()
                    .for_each(|btn| unsafe { lv_obj_add_state(*btn, LV_STATE_DISABLED) }),
                false => btns
                    .iter()
                    .for_each(|btn| unsafe { lv_obj_remove_state(*btn, LV_STATE_DISABLED) }),
            }
        }),
        effect!(|| {
            let list = match vm().unwrap().list.borrow().try_get() {
                Ok(obj) => obj,
                Err(_) => return,
            };
            let text = vm()
                .unwrap()
                .state
                .get()
                .map(|c| cstr!("Recolor to {c:?}"))
                .unwrap_or(cstr!("Non Recolor!"));
            let lbl = LvObj::from(unsafe { create_list_item(list, text.as_ptr()) });
            debug_assert_eq!(unsafe { lv_obj_get_event_count(lbl.try_get().unwrap()) }, 2);

            // Test for event::add / event::remove
            {
                let item = lbl.clone();
                let evt1 = event::add(&lbl, LV_EVENT_SHORT_CLICKED, move |e| {
                    let obj = unsafe { lv_event_get_target(e) };
                    debug_assert_eq!(obj, item.try_get().unwrap());
                    let text = unsafe { CStr::from_ptr(lv_label_get_text(obj)) };
                    println!("{text:?} Clicked!");
                });
                let evt2 = event::add(&lbl, LV_EVENT_SHORT_CLICKED, |_| {});

                if let Ok(lbl) = lbl.try_get() {
                    debug_assert_eq!(unsafe { lv_obj_get_event_count(lbl) }, 4);
                    debug_assert_eq!(unsafe { lv_obj_get_event_dsc(lbl, 2) }, evt1);
                    debug_assert_eq!(unsafe { lv_obj_get_event_dsc(lbl, 3) }, evt2);
                    debug_assert!(std::ptr::fn_addr_eq(
                        unsafe { lv_event_dsc_get_cb(lv_obj_get_event_dsc(lbl, 2)) },
                        unsafe { lv_event_dsc_get_cb(lv_obj_get_event_dsc(lbl, 3)) }
                    ));
                    debug_assert!(std::ptr::fn_addr_eq(
                        unsafe { lv_event_dsc_get_cb(lv_obj_get_event_dsc(lbl, 0)) },
                        unsafe { lv_event_dsc_get_cb(lv_obj_get_event_dsc(lbl, 2)) }
                    ));
                }

                assert!(event::remove(&lbl, evt2));
            }

            let token = vm().unwrap().cts_fade.borrow().token();
            let task = TaskRun(async move {
                let delay = delay!(5).fuse();
                let cancelled = token.cancelled().fuse();
                pin_mut!(delay, cancelled);

                select! {
                    _ = delay => { let _ = list_item_fade(&lbl, 10).await; },
                    _ = cancelled => {},
                }

                if let Ok(lbl) = lbl.try_get() {
                    unsafe { lv_obj_delete(lbl) };
                }
            });
            vm().unwrap().tasks.gc(); // unnecessary
            println!("{} tasks remaining!", vm().unwrap().tasks.attach(task));
        }),
        effect!(|| {
            match *vm().unwrap().list_item_count.get() {
                0 => {
                    let mut hint = vm().unwrap().hint.borrow_mut();
                    debug_assert!(hint.try_get().is_err());
                    *hint = LvObj::from(unsafe { create_list_hint() });
                }
                _ => {
                    if let Ok(obj) = vm().unwrap().hint.borrow().try_get() {
                        unsafe { lv_obj_delete(obj) };
                    }
                }
            };
        }),
    ]);
}
