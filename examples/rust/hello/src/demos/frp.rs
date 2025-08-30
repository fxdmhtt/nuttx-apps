#![allow(static_mut_refs)]

use std::cell::RefCell;
use std::ffi::{c_char, CStr};
use std::rc::Rc;
use std::time::Duration;

use async_cancellation_token::{CancellationTokenSource, Cancelled};
use async_executor::Task;
use futures::future::LocalBoxFuture;
use futures::stream::FuturesUnordered;
use futures::{pin_mut, select, stream, FutureExt, StreamExt, TryStreamExt};
use itertools_num::linspace;
use reactive_cache::{effect, Effect, Memo, Signal};
use stack_cstr::cstr;
use thiserror::Error;

use crate::binding::lvgl::*;
use crate::runtime::{event, executor, TaskRun};
use crate::*;

extern "C" {
    fn rust_executor_wake();
}

extern "C" {
    static mut radio_cont: *mut lv_obj_t;
    static mut img: *mut lv_obj_t;
    static mut img_label: *mut lv_obj_t;
    static mut btn1: *mut lv_obj_t;
    static mut btn2: *mut lv_obj_t;
    static mut no_color_btn: *mut lv_obj_t;
    static mut list: *mut lv_obj_t;
    static mut slider: *mut lv_obj_t;

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
}

struct ViewModel {
    effects: RefCell<Vec<Rc<Effect>>>,
    tasks: RefCell<FuturesUnordered<LocalBoxFuture<'static, ()>>>,
    _bg_task: Task<()>,

    active_index: RefCell<Rc<Signal<i32>>>,
    intense: RefCell<Rc<Signal<lv_opa_t>>>,
    recolor_animation: RefCell<Rc<Signal<bool>>>,
    list_item_count: RefCell<Rc<Signal<u32>>>,

    state: Rc<Memo<State>>,

    cts_fade: RefCell<CancellationTokenSource>,
    cts_anim: RefCell<CancellationTokenSource>,

    hint: RefCell<Option<*mut lv_obj_t>>,
}

impl ViewModel {
    fn new() -> Self {
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
            effects: effects.into(),
            tasks: FuturesUnordered::new().into(),
            _bg_task: tasks_cleanup_in_background(),

            active_index: active_index.into(),
            intense: intense.into(),
            recolor_animation: recolor_animation.into(),
            list_item_count: list_item_count.into(),

            state,

            cts_fade: CancellationTokenSource::new().into(),
            cts_anim: CancellationTokenSource::new().into(),

            hint: None.into(),
        }
    }
}

static mut VM: Option<Box<ViewModel>> = None;

fn vm() -> Result<&'static ViewModel, NotRunning> {
    (unsafe { VM.as_deref() }).ok_or(NotRunning)
}

#[no_mangle]
extern "C" fn active_index_get() -> i32 {
    *vm().unwrap().active_index.borrow().get()
}

#[no_mangle]
extern "C" fn active_index_set(value: i32) -> bool {
    vm().unwrap().active_index.borrow().set(value)
}

event_decl!(switch_color_event, {
    let id = *vm().unwrap().active_index.borrow().get() + 1;
    let id = if *vm().unwrap().recolor_animation.borrow().get() && id == 4 {
        id + 1
    } else {
        id
    };
    vm().unwrap().active_index.borrow().set(id % 5);
});

fn tasks_cleanup_in_background() -> Task<()> {
    TaskRun(async move {
        let mut id = 0;
        loop {
            if let Ok(vm) = vm() {
                let mut tasks = vm.tasks.replace(FuturesUnordered::new());

                while tasks.next().await.is_some() {
                    println!("A mission completed! id: {id}");
                    id += 1;
                }
            }

            let _ = delay!(Duration::from_millis(1000)).await;
        }
    })
}

fn cts_cancel_and_renew(cts: &RefCell<CancellationTokenSource>) {
    let old = cts.replace(CancellationTokenSource::new());
    assert!(!old.is_cancelled());
    old.cancel();
    unsafe { rust_executor_wake() }; // necessary!
}

async fn list_item_fade(obj: *mut lv_obj_t, cnt: usize) -> Result<(), AppError> {
    let token = vm()?.cts_fade.borrow().token();

    stream::iter(linspace(255.0, 0.0, cnt).map(|x: f32| x.round() as u8))
        .map(Ok)
        .try_for_each(|x| {
            let token = token.clone();
            async move {
                vm()?;
                unsafe { lv_obj_set_style_opa(obj, x, LV_PART_MAIN) };
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
    let start = *vm()?.intense.borrow().get();

    let header = if target > start {
        "Increase color density"
    } else {
        "Decrease color density"
    };
    let text = cstr!("{header} - {start}");
    let lbl = unsafe { create_list_item(list, text.as_ptr()) };

    vm()?.recolor_animation.borrow().set(true);
    stream::iter(
        linspace(start as f32, target as f32, ticks as usize).map(|x: f32| x.round() as u8),
    )
    .map(Ok)
    .try_for_each(|cur| {
        let token = token.clone();

        async move {
            vm()?.intense.borrow().set(cur);

            let text = cstr!("{header} - {cur}");
            unsafe { lv_checkbox_set_text(lbl, text.as_ptr()) };

            delay!(delay_anim, token).await?;

            Ok::<_, AppError>(())
        }
    })
    .await
    .inspect(|_| {
        if let Ok(vm) = vm() {
            vm.intense.borrow().set(target);
        }
    })
    .or_else(|e| match e {
        AppError::NotRunning(_) => Err(NotRunning),
        AppError::CancelError(_) => Ok(()),
    })?;
    vm()?.recolor_animation.borrow().set(false);

    let token = vm()?.cts_anim.borrow().token();
    let _ = delay!(1, token).await;
    let _ = list_item_fade(lbl, 15).await;

    vm()?;
    unsafe { lv_obj_delete(lbl) };

    Ok(())
}

event_decl!(intense_inc_event, async {
    if *vm()?.intense.borrow().get() < 0xff {
        return intense_animation(0xff, Duration::from_secs(5)).await;
    }
    Ok(())
});

event_decl!(intense_dec_or_clear_event, async {
    match vm()?.state.get() {
        Some(_) => {
            if *vm()?.intense.borrow().get() > 0 {
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
    vm().unwrap().list_item_count.borrow().set(cnt);
});

#[no_mangle]
extern "C" fn frp_demo_rs_drop() {
    vm().unwrap().cts_fade.borrow_mut().cancel();
    vm().unwrap().cts_anim.borrow_mut().cancel();
    executor().try_tick_all();

    vm().unwrap().tasks.borrow_mut().clear();
    executor().try_tick_all();

    let weak_active_index = Rc::downgrade(&vm().unwrap().active_index.borrow());
    let weak_intense = Rc::downgrade(&vm().unwrap().intense.borrow());
    let weak_recolor_animation = Rc::downgrade(&vm().unwrap().recolor_animation.borrow());
    let weak_list_item_count = Rc::downgrade(&vm().unwrap().list_item_count.borrow());
    let weak_state = Rc::downgrade(&vm().unwrap().state);
    let weak_effects = vm()
        .unwrap()
        .effects
        .borrow()
        .iter()
        .map(Rc::downgrade)
        .collect::<Vec<_>>();

    drop(unsafe { &mut VM }.take().unwrap());

    assert!(weak_active_index.upgrade().is_none());
    assert!(weak_intense.upgrade().is_none());
    assert!(weak_recolor_animation.upgrade().is_none());
    assert!(weak_list_item_count.upgrade().is_none());
    assert!(weak_state.upgrade().is_none());
    assert!(weak_effects.iter().all(|w| w.upgrade().is_none()));
}

#[no_mangle]
extern "C" fn frp_demo_rs_init() {
    assert!(unsafe { &mut VM }
        .replace(ViewModel::new().into())
        .is_none());

    vm().unwrap().effects.borrow_mut().extend(vec![
        effect!(|| {
            let id = *vm().unwrap().active_index.borrow().get();
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
        BindingSliderValue!(
            unsafe { slider },
            vm().unwrap().intense.borrow(),
            ConvertBack | v | {
                cts_cancel_and_renew(&vm().unwrap().cts_anim);
                v
            }
        ),
        effect!(|| {
            unsafe {
                lv_obj_update_flag(
                    slider,
                    LV_OBJ_FLAG_HIDDEN,
                    vm().unwrap().state.get().is_none(),
                );
            };
        }),
        BindingImageRecolor!(unsafe { img }, {
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
        BindingImageRecolorOpa!(unsafe { img }, {
            match (
                vm().unwrap().state.get(),
                *vm().unwrap().intense.borrow().get(),
            ) {
                (Some(_), intense) => intense,
                (None, _) => 0,
            }
        }),
        BindingText!(unsafe { img_label }, {
            vm().unwrap()
                .state
                .get()
                .map(|c| cstr!("Color {c:?}"))
                .unwrap_or(cstr!("Original Color"))
        }),
        effect!(|| {
            match (
                vm().unwrap().state.get(),
                *vm().unwrap().recolor_animation.borrow().get(),
                *vm().unwrap().intense.borrow().get(),
            ) {
                (_, true, _) => unsafe { lv_obj_add_state(btn1, LV_STATE_DISABLED) },
                (None, _, _) => unsafe { lv_obj_add_state(btn1, LV_STATE_DISABLED) },
                (Some(_), _, 0xff) => unsafe { lv_obj_add_state(btn1, LV_STATE_DISABLED) },
                _ => unsafe { lv_obj_remove_state(btn1, LV_STATE_DISABLED) },
            };
        }),
        effect!(|| {
            match (
                vm().unwrap().state.get(),
                *vm().unwrap().recolor_animation.borrow().get(),
                *vm().unwrap().intense.borrow().get(),
            ) {
                (_, true, _) => unsafe { lv_obj_add_state(btn2, LV_STATE_DISABLED) },
                (None, _, _) if *vm().unwrap().list_item_count.borrow().get() == 0 => unsafe {
                    lv_obj_add_state(btn2, LV_STATE_DISABLED)
                },
                (Some(_), _, 0) => unsafe { lv_obj_add_state(btn2, LV_STATE_DISABLED) },
                _ => unsafe { lv_obj_remove_state(btn2, LV_STATE_DISABLED) },
            };
        }),
        BindingText!(unsafe { lv_obj_get_child(btn2, 0) }, {
            match vm().unwrap().state.get() {
                Some(_) => cstr!("Intense Dec"),
                None => cstr!("Clear Log"),
            }
        }),
        BindingBgColor!(unsafe { btn2 }, {
            match vm().unwrap().state.get() {
                Some(_) => unsafe { lv_palette_main(LV_PALETTE_BLUE) },
                None => unsafe { lv_palette_main(LV_PALETTE_RED) },
            }
        }),
        effect!(|| {
            let btns = unsafe { [no_color_btn] };
            match *vm().unwrap().recolor_animation.borrow().get() {
                true => btns
                    .iter()
                    .for_each(|btn| unsafe { lv_obj_add_state(*btn, LV_STATE_DISABLED) }),
                false => btns
                    .iter()
                    .for_each(|btn| unsafe { lv_obj_remove_state(*btn, LV_STATE_DISABLED) }),
            }
        }),
        effect!(|| {
            let text = vm()
                .unwrap()
                .state
                .get()
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

            let token = vm().unwrap().cts_fade.borrow().token();
            let task = TaskRun(async move {
                let delay = delay!(5).fuse();
                let cancelled = token.cancelled().fuse();
                pin_mut!(delay, cancelled);

                select! {
                    _ = delay => { let _ = list_item_fade(lbl, 10).await; },
                    _ = cancelled => {},
                }

                unsafe { lv_obj_delete(lbl) };
            });
            vm().unwrap().tasks.borrow_mut().push(task.boxed_local());
        }),
        effect!(|| {
            match *vm().unwrap().list_item_count.borrow().get() {
                0 => {
                    if vm().unwrap().hint.borrow().is_none() {
                        unsafe { vm().unwrap().hint.borrow_mut().replace(create_list_hint()) };
                    }
                }
                _ => {
                    if let Some(obj) = vm().unwrap().hint.take() {
                        unsafe { lv_obj_delete(obj) };
                    }
                }
            };
        }),
    ]);
}
