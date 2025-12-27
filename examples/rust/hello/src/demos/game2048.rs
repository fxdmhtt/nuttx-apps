// Internationalization support is provided by compiling and
// generating the `_T(Lang, Text) -> &CStr` function via `build.rs`.
include!(concat!(env!("OUT_DIR"), "/i18n.rs"));

use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    ptr::{null_mut, NonNull},
    rc::Rc,
    time::Duration,
    vec,
};

use futures::future::join_all;
use game2048::*;
use reactive_cache::{effect, prelude::*};
use stack_cstr::cstr;

use crate::{
    runtime::{lvgl::*, *},
    *,
};

macro_rules! SCORE_LABEL_TEXT {
    (u) => {
        "\u{E000}"
    };
    (x) => {
        0xE000
    };
}

macro_rules! SCORE_MAX_LABEL_TEXT {
    (u) => {
        "\u{E001}"
    };
    (x) => {
        0xE001
    };
}

// Referencing the built-in font `LVGL` in the C language
extern "C" {
    static lv_font_montserrat_24: lv_font_t;
}

const NUM_COORDS: [[(i32, i32); 4]; 4] = [
    [(20, 78), (104, 78), (188, 78), (272, 78)],
    [(20, 162), (104, 162), (188, 162), (272, 162)],
    [(20, 246), (104, 246), (188, 246), (272, 246)],
    [(20, 330), (104, 330), (188, 330), (272, 330)],
];

// The application's state machine
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
enum State {
    #[default]
    ClickToStart,
    Playing,
    Win,
    GameOver,
}

// The ownership of the active page changes as the page is switched.
#[allow(dead_code)]
struct Page {
    // It is recommended to store only a reference to the root node
    // of the visualization tree, and all references to `lv_obj_t *`
    // must be encapsulated within `LvObjHandle` to prevent dangling references.
    root: LvObjHandle,

    // Define all `Effect`s related to the current page's side effects.
    //
    // It holds ownership of the `Effect`s on the activity page
    // and replaces them when the activity page changes,
    // thus avoiding unnecessary effects.
    effects: Vec<Rc<Effect>>,
}

impl Drop for Page {
    fn drop(&mut self) {
        match self.root.try_get() {
            // A typical scenario is that a button triggers a page switch,
            // which causes the current page containing the button to be dropped.
            // In this case, `lv_obj_delete` will cause an exception;
            // therefore, `lv_obj_delete_async` must be used to delete the page.
            //
            // Refer: https://docs.lvgl.io/master/common-widget-features/api.html#widget-deletion
            Ok(root) => unsafe { lv_obj_delete_async(root) },
            Err(_) => unreachable!(),
        }
    }
}

struct ViewModel {
    // If asynchronous task support is required,
    // all tasks must be attached to the `TaskManager`
    // so that all asynchronous tasks can be terminated
    // when the `ViewModel` is dropped,
    // reducing the reliance on `CancellationTokenSource`.
    tasks: Rc<TaskManager>,

    // Define all `Signal`s.
    state: Rc<Signal<State>>,

    // Define all `Memo`s.

    // Define all `Effect`s that are related to the `ViewModel`
    // rather than the page itself.
    effects: RefCell<Vec<Rc<Effect>>>,

    // Scenarios involving ownership transfer,
    // complete replacement or removal,
    // and where no borrowing occurs,
    // are suitable for non-`Copy` `Cell<T>`
    // rather than `RefCell<T>`.
    active_page: Cell<Option<Page>>,

    game: Rc<RefCell<Game2048>>,

    // Members that are only initialized
    // during the creation of the `ViewModel`
    // do not require `Cell<T>` or `RefCell<T>`
    // to provide mutability.
    _lv_imgfont: NonNull<lv_font_t>,
}

impl Drop for ViewModel {
    fn drop(&mut self) {
        // The `imgfont` object is deleted immediately, but the `root` of the `Page`
        // is deleted later in the `Drop` of the `Page`, which may cause a memory
        // access error when referencing the font in the next rendering cycle.
        //
        // To avoid double freeing of memory during Page release, all UI objects under
        // the Page's root must be deleted immediately when the ViewModel is dropped,
        // but the Page's root node itself must be preserved for release by the `Drop`.
        if let Some(page) = self.active_page.take() {
            if let Ok(root) = page.root.try_get() {
                unsafe { lv_obj_clean(root) };
            }
        }

        unsafe { lv_imgfont_destroy(self._lv_imgfont.as_ptr()) };

        // The assertion holds true, requiring:
        //
        // 1. All references to `ViewModel` members are `Weak` references.
        // (current implementation)
        //
        // 2. The current page must be actively deleted to release all closures.
        // ```
        // self.active_page.take();
        // ```
        // However, this requires immediate deletion (`lv_obj_delete`)
        // instead of asynchronous deletion (`lv_obj_delete_async`),
        // so the `Drop` implementation of `Page` needs to be modified.
        //
        // 3. The current page must be deleted immediately to release all closures.
        // ```
        // if let Some(page) = self.active_page.take() {
        //     if let Ok(root) = page.root.try_get() {
        //         unsafe { lv_obj_delete(root) };
        //     }
        // }
        // ```
        // This also requires modifying the `Drop` implementation of `Page`,
        // removing the `unreachable!()` assertion that prevents double freeing
        // when the `Page` is released.
        //
        // 4. All objects on the current page must be deleted immediately to release all closures.
        // (current implementation)
        // ```
        // if let Some(page) = self.active_page.take() {
        //     if let Ok(root) = page.root.try_get() {
        //         unsafe { lv_obj_clean(root) };
        //     }
        // }
        // ```
        #[cfg(debug_assertions)]
        {
            debug_assert_eq!(Rc::strong_count(&self.tasks), 1);
            debug_assert_eq!(Rc::strong_count(&self.state), 1);
            self.effects
                .borrow()
                .iter()
                .map(Rc::strong_count)
                .for_each(|c| debug_assert_eq!(c, 1));
            debug_assert_eq!(Rc::strong_count(&self.game), 1);
        }
    }
}

impl ViewModel {
    fn new() -> Self {
        Self {
            tasks: TaskManager::new_with_auto_gc(TaskRun::<()>).into(),

            state: Default::default(),

            effects: Default::default(),

            active_page: Default::default(),

            game: Default::default(),

            _lv_imgfont: NonNull::new(unsafe { lv_imgfont_create(36, get_imgfont_path, null_mut()) }).unwrap(),
        }
    }

    fn page_changed(&self, new_page: Page) {
        self.active_page.replace(Some(new_page));
    }

    unsafe fn show_clicktostart(&self, parent: *mut lv_obj_t) {
        println!("[{}] {} Created!", here!(), callee!());

        let bg_img = lv_image_create(parent);
        lv_image_set_src(bg_img, cstr!("A:/game2048/bg2048.png").as_ptr() as _);
        lv_obj_center(bg_img);

        let title = lv_label_create(bg_img);
        lv_label_set_text(title, _T(Lang::en_US, Text::HistoryRecord).as_ptr());
        lv_obj_align(title, LV_ALIGN_TOP_MID, 0, 25);
        lv_obj_set_style_text_color(title, lv_color_hex(0xFFFFFF), LV_PART_MAIN);
        lv_obj_set_style_text_font(title, &lv_font_montserrat_24 as _, LV_PART_MAIN);

        let loc_img = lv_image_create(bg_img);
        lv_image_set_src(loc_img, cstr!("A:/game2048/location4.png").as_ptr() as _);
        lv_obj_set_pos(loc_img, 27, 85);

        let start_btn = lv_button_create(bg_img);
        lv_obj_set_size(start_btn, 212, 72);
        lv_obj_set_pos(start_btn, 89, 215);
        lv_obj_set_style_radius(start_btn, 90, LV_PART_MAIN);
        lv_obj_set_style_bg_color(start_btn, lv_color_hex(0x181818), LV_PART_MAIN);
        lv_obj_set_style_bg_opa(start_btn, LV_OPA_30, LV_PART_MAIN);
        let start_lbl = lv_label_create(start_btn);
        lv_obj_center(start_lbl);
        lv_label_set_text(start_lbl, _T(Lang::en_US, Text::ClickToStart).as_ptr());
        lv_obj_set_style_text_align(start_lbl, LV_TEXT_ALIGN_CENTER, LV_PART_MAIN);
        lv_obj_set_style_text_color(start_lbl, lv_color_hex(0xFFFFFF), LV_PART_MAIN);
        lv_obj_set_style_text_font(start_lbl, &lv_font_montserrat_24 as _, LV_PART_MAIN);

        let start_btn = LvObj::from(start_btn);
        {
            downgrade!(self.state);
            event::add(&start_btn, LV_EVENT_SHORT_CLICKED, move |_| {
                if let Some(state) = state.upgrade() {
                    state.set(State::Playing);
                }
            });
        }

        let bg_img = LvObj::from(bg_img);
        event::add(&bg_img, LV_EVENT_DELETE, |_| {
            println!("[{}] {} Deleted!", here!(), callee!());
        });

        self.page_changed(Page { root: bg_img, effects: vec![] });
    }

    unsafe fn show_playing(&self, parent: *mut lv_obj_t) {
        println!("[{}] {} Created!", here!(), callee!());

        let bg_img = lv_image_create(parent);
        lv_image_set_src(bg_img, cstr!("A:/game2048/bg2048.png").as_ptr() as _);
        lv_obj_center(bg_img);
        lv_obj_add_flag(bg_img, LV_OBJ_FLAG_CLICKABLE);
        lv_obj_remove_flag(bg_img, LV_OBJ_FLAG_GESTURE_BUBBLE);

        let title = lv_label_create(bg_img);
        lv_label_set_text(title, cstr!("Score: 0").as_ptr());
        lv_obj_align(title, LV_ALIGN_TOP_MID, 0, 25);
        lv_obj_set_style_text_color(title, lv_color_hex(0xFFFFFF), LV_PART_MAIN);
        lv_obj_set_style_text_font(title, &lv_font_montserrat_24 as _, LV_PART_MAIN);

        let bg_img = LvObj::from(bg_img);
        {
            random_fill(&self.game, &bg_img);

            downgrade!(self.state, self.game, self.tasks);
            clone!(bg_img);
            event::add(&bg_img.clone(), LV_EVENT_GESTURE, move |e| {
                clone!(state, game, bg_img);
                let gesture = lv_indev_get_gesture_dir(lv_indev_active());
                let task = TaskRun(async move {
                    let path = match game.upgrade() {
                        Some(game) => match gesture {
                            LV_DIR_LEFT => game.borrow_mut().left(),
                            LV_DIR_RIGHT => game.borrow_mut().right(),
                            LV_DIR_TOP => game.borrow_mut().up(),
                            LV_DIR_BOTTOM => game.borrow_mut().down(),
                            _ => vec![],
                        },
                        None => return,
                    };

                    if path.is_empty() {
                        return;
                    }
                    // dbg!(&path);

                    let parent = unsafe { lv_event_get_target(e) };
                    lv_obj_remove_flag(parent, LV_OBJ_FLAG_CLICKABLE);

                    let tasks = path
                        .into_iter()
                        .map(|Path2D { orig, dest, end }| {
                            let (x1, y1) = NUM_COORDS[orig.row][orig.col];
                            let obj = search_num(parent, x1, y1);
                            let (x2, y2) = NUM_COORDS[dest.row][dest.col];

                            async move {
                                let var = LvObj::from(obj);
                                if x1 != x2 {
                                    let _ = LvAnim::new(
                                        &var.try_into().unwrap(),
                                        Duration::from_millis(100),
                                        lv_obj_set_x,
                                        (x1, x2),
                                    )
                                    .await;
                                } else if y1 != y2 {
                                    let _ = LvAnim::new(
                                        &var.try_into().unwrap(),
                                        Duration::from_millis(100),
                                        lv_obj_set_y,
                                        (y1, y2),
                                    )
                                    .await;
                                } else {
                                    yield_now!().await;
                                }
                                (obj, end)
                            }
                        })
                        .map(TaskRun)
                        .collect::<Vec<_>>();

                    join_all(tasks)
                        .await
                        .into_iter()
                        .for_each(|(obj, end)| match end {
                            NumOp::OnlyMove => {}
                            NumOp::Double => {
                                let x = (lv_obj_get_user_data(obj) as usize) << 1;
                                lv_obj_set_user_data(obj, x as _);
                                lv_image_set_src(obj, cstr!("A:/game2048/num{x}.png").as_ptr() as _);
                            }
                            NumOp::Disappear => {
                                lv_obj_delete(obj);
                            }
                        });

                    lv_obj_add_flag(parent, LV_OBJ_FLAG_CLICKABLE);

                    if let Some(game) = game.upgrade() {
                        let score = game.borrow().get_score();
                        lv_label_set_text(title, cstr!("Score: {score}").as_ptr());

                        if game.borrow().is_it_win() {
                            println!("{}", game.borrow());
                            println!("2048!");
                            println!("Game Over");
                            if let Some(state) = state.upgrade() {
                                state.set(State::Win);
                            }
                        }

                        random_fill(&game, &bg_img);

                        if game.borrow().is_it_over() {
                            println!("Game Over");
                            if let Some(state) = state.upgrade() {
                                state.set(State::GameOver);
                            }
                        }
                    }
                });
                if let Some(tasks) = tasks.upgrade() {
                    tasks.attach(task);
                }
            });
        }

        event::add(&bg_img, LV_EVENT_DELETE, |_| {
            println!("[{}] {} Deleted!", here!(), callee!());
        });

        self.page_changed(Page { root: bg_img, effects: vec![] });
    }

    unsafe fn show_gameover(&self, parent: *mut lv_obj_t) {
        println!("[{}] {} Created!", here!(), callee!());

        let bg_img = lv_image_create(parent);
        lv_image_set_src(bg_img, cstr!("A:/game2048/endbg.png").as_ptr() as _);
        lv_obj_center(bg_img);

        let title = lv_image_create(bg_img);
        lv_image_set_src(title, cstr!("A:/game2048/youxijieshu.png").as_ptr() as _);
        lv_obj_align(title, LV_ALIGN_TOP_MID, 0, 90);

        let retry = lv_image_create(bg_img);
        lv_image_set_src(retry, cstr!("A:/game2048/game2048_retry.png").as_ptr() as _);
        lv_obj_align(retry, LV_ALIGN_BOTTOM_MID, 0, -16);
        lv_obj_add_flag(retry, LV_OBJ_FLAG_CLICKABLE);

        let score = self.game.borrow().get_score();

        let fenshu = lv_label_create(bg_img);
        lv_label_set_text(
            fenshu,
            cstr!(concat!(SCORE_LABEL_TEXT!(u), ":{}"), score).as_ptr(),
        );
        lv_obj_set_style_text_font(fenshu, self._lv_imgfont.as_ptr(), LV_PART_MAIN);
        lv_obj_align(fenshu, LV_ALIGN_TOP_MID, 0, 192);

        let fenshu_max = lv_label_create(bg_img);
        lv_label_set_text(
            fenshu_max,
            cstr!(concat!(SCORE_MAX_LABEL_TEXT!(u), ":{}"), score).as_ptr(),
        );
        lv_obj_set_style_text_font(fenshu_max, self._lv_imgfont.as_ptr(), LV_PART_MAIN);
        lv_obj_align(fenshu_max, LV_ALIGN_TOP_MID, 0, 253);

        let retry = LvObj::from(retry);
        {
            downgrade!(self.state);
            event::add(&retry, LV_EVENT_SHORT_CLICKED, move |_| {
                if let Some(state) = state.upgrade() {
                    state.set(State::Playing);
                }
            });
        }

        let bg_img = LvObj::from(bg_img);
        event::add(&bg_img, LV_EVENT_DELETE, |_| {
            println!("[{}] {} Deleted!", here!(), callee!());
        });

        self.page_changed(Page { root: bg_img, effects: vec![] });
    }
}

unsafe fn search_num(parent: *mut lv_obj_t, x: i32, y: i32) -> *mut lv_obj_t {
    let objs = (0..lv_obj_get_child_count(parent))
        .map(|i| lv_obj_get_child(parent, i as i32))
        .filter(|o| lv_obj_get_x(*o) == x && lv_obj_get_y(*o) == y)
        .collect::<Vec<_>>();

    debug_assert_eq!(objs.len(), 1);
    objs[0]
}

fn random_fill(game: &Rc<RefCell<Game2048>>, parent: &LvObjHandle) {
    let (x, p) = game.borrow_mut().random_fill().unwrap();
    println!("{}", game.borrow());

    let parent = match parent.try_get() {
        Ok(obj) => obj,
        Err(_) => return,
    };

    let num = unsafe { lv_image_create(parent) };
    unsafe { lv_obj_set_user_data(num, x as _) };
    unsafe { lv_image_set_src(num, cstr!("A:/game2048/num{x}.png").as_ptr() as _) };

    let (x, y) = NUM_COORDS[p.row][p.col];
    unsafe { lv_obj_set_pos(num, x, y) };
}

unsafe extern "C" fn get_imgfont_path(
    _font: *const lv_font_t,
    unicode: u32,
    _unicode_next: u32,
    _offset_y: *mut i32,
    _user_data: *mut c_void,
) -> *const c_void {
    match unicode {
        0x0030 => c"A:/game2048/big0.png".as_ptr() as _,
        0x0031 => c"A:/game2048/big1.png".as_ptr() as _,
        0x0032 => c"A:/game2048/big2.png".as_ptr() as _,
        0x0033 => c"A:/game2048/big3.png".as_ptr() as _,
        0x0034 => c"A:/game2048/big4.png".as_ptr() as _,
        0x0035 => c"A:/game2048/big5.png".as_ptr() as _,
        0x0036 => c"A:/game2048/big6.png".as_ptr() as _,
        0x0037 => c"A:/game2048/big7.png".as_ptr() as _,
        0x0038 => c"A:/game2048/big8.png".as_ptr() as _,
        0x0039 => c"A:/game2048/big9.png".as_ptr() as _,
        0x003A => c"A:/game2048/maohaobig.png".as_ptr() as _,
        SCORE_LABEL_TEXT!(x) => c"A:/game2048/fenshu.png".as_ptr() as _,
        SCORE_MAX_LABEL_TEXT!(x) => c"A:/game2048/zuigaofenshu.png".as_ptr() as _,
        _ => null_mut(),
    }
}

#[no_mangle]
extern "C" fn game2048_new() -> *const ViewModel {
    let vm = Rc::new(ViewModel::new());

    *vm.effects.borrow_mut() = vec![{
        downgrade!(vm);
        effect!(move || {
            if let Some(vm) = vm.upgrade() {
                match *vm.state.get() {
                    State::ClickToStart => unsafe { vm.show_clicktostart(lv_screen_active()) },
                    State::Playing => {
                        *vm.game.borrow_mut() = Game2048::default();
                        unsafe { vm.show_playing(lv_screen_active()) }
                    }
                    State::Win => todo!(),
                    State::GameOver => unsafe { vm.show_gameover(lv_screen_active()) },
                }
            }
        })
    }];

    Rc::into_raw(vm)
}

#[no_mangle]
extern "C" fn game2048_drop(vm: *const ViewModel) {
    let vm = unsafe { Rc::from_raw(vm) };

    let weak_vm = Rc::downgrade(&vm);
    drop(vm);
    debug_assert!(weak_vm.upgrade().is_none());
}
