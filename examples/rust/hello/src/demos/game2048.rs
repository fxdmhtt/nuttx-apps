#![allow(static_mut_refs)]

use std::{cell::RefCell, rc::Rc, time::Duration, vec};

use futures::future::join_all;
use game2048::*;
use reactive_cache::{effect, Effect, Signal};
use stack_cstr::cstr;

use crate::{
    runtime::{lvgl::*, *},
    *,
};

extern "C" {
    pub static lv_font_montserrat_24: lv_font_t;
}

const NUM_COORDS: [[(i32, i32); 4]; 4] = [
    [(20, 78), (104, 78), (188, 78), (272, 78)],
    [(20, 162), (104, 162), (188, 162), (272, 162)],
    [(20, 246), (104, 246), (188, 246), (272, 246)],
    [(20, 330), (104, 330), (188, 330), (272, 330)],
];

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
enum State {
    #[default]
    ClickToStart,
    Playing,
    Win,
    GameOver,
}

#[derive(Default)]
struct ViewModel {
    tasks: Rc<TaskManager>,

    state: Rc<Signal<State>>,

    effects: Vec<Rc<Effect>>,

    game: Rc<RefCell<Game2048>>,

    // It is recommended to only store a reference to the root of the visual tree,
    // and all references to `lv_obj_t *` must be encapsulated within `LvObjHandle`
    // to prevent dangling references.
    ui_tree_root: RefCell<LvObjHandle>,
}

impl Drop for ViewModel {
    fn drop(&mut self) {
        if let Ok(root) = self.ui_tree_root.borrow().try_get() {
            unsafe { lv_obj_delete(root) };
        }

        #[cfg(debug_assertions)]
        {
            assert_eq!(Rc::strong_count(&self.tasks), 1);
            assert_eq!(Rc::strong_count(&self.state), 1);
            self.effects
                .iter()
                .map(Rc::strong_count)
                .for_each(|c| assert_eq!(c, 1));
            assert_eq!(Rc::strong_count(&self.game), 1);
        }
    }
}

impl ViewModel {
    fn new() -> Self {
        Self {
            tasks: TaskManager::new_with_auto_gc(TaskRun::<()>).into(),

            state: Default::default(),

            effects: Default::default(),

            game: Default::default(),

            ui_tree_root: Default::default(),
        }
    }

    fn root_changed(&self, root: LvObjHandle) {
        if let Ok(root) = self.ui_tree_root.borrow().try_get() {
            unsafe { lv_obj_delete_async(root) };
        }

        *self.ui_tree_root.borrow_mut() = root;
    }

    unsafe fn show_clicktostart(&self, parent: *mut lv_obj_t) {
        let bg_img = lv_image_create(parent);
        lv_image_set_src(bg_img, cstr!("A:/game2048/bg2048.png").as_ptr() as _);
        lv_obj_center(bg_img);

        let title = lv_label_create(bg_img);
        lv_label_set_text(title, cstr!("History Record").as_ptr());
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
        lv_label_set_text(start_lbl, cstr!("Click To Start").as_ptr() as _);
        lv_obj_set_style_text_align(start_lbl, LV_TEXT_ALIGN_CENTER, LV_PART_MAIN);
        lv_obj_set_style_text_color(start_lbl, lv_color_hex(0xFFFFFF), LV_PART_MAIN);
        lv_obj_set_style_text_font(start_lbl, &lv_font_montserrat_24 as _, LV_PART_MAIN);

        let start_btn = LvObj::from(start_btn);
        {
            let state = self.state.clone();
            event::add(&start_btn, LV_EVENT_SHORT_CLICKED, move |_| {
                state.set(State::Playing);
            });
        }

        self.root_changed(LvObj::from(bg_img));
    }

    unsafe fn show_playing(&self, parent: *mut lv_obj_t) {
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

            clone!(self.state, self.game, self.tasks);
            clone!(bg_img);
            event::add(&bg_img.clone(), LV_EVENT_GESTURE, move |e| {
                clone!(state, game, bg_img);
                let gesture = lv_indev_get_gesture_dir(lv_indev_active());
                let task = TaskRun(async move {
                    let path = match gesture {
                        LV_DIR_LEFT => game.borrow_mut().left(),
                        LV_DIR_RIGHT => game.borrow_mut().right(),
                        LV_DIR_TOP => game.borrow_mut().up(),
                        LV_DIR_BOTTOM => game.borrow_mut().down(),
                        _ => vec![],
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

                    let score = game.borrow().get_score();
                    lv_label_set_text(title, cstr!("Score: {score}").as_ptr());

                    if game.borrow().is_it_win() {
                        println!("{}", game.borrow());
                        println!("2048!");
                        println!("游戏结束");
                        state.set(State::Win);
                    }

                    random_fill(&game, &bg_img);

                    if game.borrow().is_it_over() {
                        println!("游戏结束");
                        state.set(State::GameOver);
                    }
                });
                tasks.attach(task);
            });
        }

        self.root_changed(bg_img);
    }

    unsafe fn show_gameover(&self, parent: *mut lv_obj_t) {
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

        {
            let fenshu_cont = lv_image_create(bg_img);
            lv_obj_align(fenshu_cont, LV_ALIGN_TOP_MID, 0, 192);
            lv_obj_set_flex_flow(fenshu_cont, LV_FLEX_FLOW_ROW);
            lv_obj_set_flex_align(
                fenshu_cont,
                LV_FLEX_ALIGN_START,
                LV_FLEX_ALIGN_CENTER,
                LV_FLEX_ALIGN_START,
            );
            lv_obj_set_style_pad_column(fenshu_cont, 0, LV_PART_MAIN);
            let fenshu = lv_image_create(fenshu_cont);
            lv_image_set_src(fenshu, cstr!("A:/game2048/fenshu.png").as_ptr() as _);
            let maohao = lv_image_create(fenshu_cont);
            lv_image_set_src(maohao, cstr!("A:/game2048/maohaobig.png").as_ptr() as _);
            let score = self.game.borrow().get_score();
            create_imgs_from_nums(fenshu_cont, score);
        }

        {
            let fenshu_max_cont = lv_image_create(bg_img);
            lv_obj_align(fenshu_max_cont, LV_ALIGN_TOP_MID, 0, 253);
            lv_obj_set_flex_flow(fenshu_max_cont, LV_FLEX_FLOW_ROW);
            lv_obj_set_flex_align(
                fenshu_max_cont,
                LV_FLEX_ALIGN_START,
                LV_FLEX_ALIGN_CENTER,
                LV_FLEX_ALIGN_START,
            );
            lv_obj_set_style_pad_column(fenshu_max_cont, 0, LV_PART_MAIN);
            let fenshu_max = lv_image_create(fenshu_max_cont);
            lv_image_set_src(
                fenshu_max,
                cstr!("A:/game2048/zuigaofenshu.png").as_ptr() as _,
            );
            let maohao = lv_image_create(fenshu_max_cont);
            lv_image_set_src(maohao, cstr!("A:/game2048/maohaobig.png").as_ptr() as _);
            let score = self.game.borrow().get_score();
            create_imgs_from_nums(fenshu_max_cont, score);
        }

        let retry = LvObj::from(retry);
        {
            let state = self.state.clone();
            event::add(&retry, LV_EVENT_SHORT_CLICKED, move |_| {
                state.set(State::Playing);
            });
        }

        self.root_changed(LvObj::from(bg_img));
    }
}

unsafe fn create_imgs_from_nums(parent: *mut lv_obj_t, num: usize) -> *mut lv_obj_t {
    let cont = lv_image_create(parent);
    lv_obj_set_flex_flow(cont, LV_FLEX_FLOW_ROW);
    lv_obj_set_flex_align(
        cont,
        LV_FLEX_ALIGN_START,
        LV_FLEX_ALIGN_END,
        LV_FLEX_ALIGN_START,
    );
    lv_obj_set_style_pad_column(cont, 0, LV_PART_MAIN);

    let mut score = num;
    std::iter::from_fn(move || {
        if score == 0 {
            None
        } else {
            let d = score % 10;
            score /= 10;
            Some(d)
        }
    })
    .collect::<Vec<_>>()
    .iter()
    .rev()
    .for_each(|n| {
        let obj = lv_image_create(cont);
        lv_image_set_src(obj, cstr!("A:/game2048/big{n}.png").as_ptr() as _);
        lv_obj_set_style_width(obj, 20, LV_PART_MAIN);
    });

    cont
}

unsafe fn search_num(parent: *mut lv_obj_t, x: i32, y: i32) -> *mut lv_obj_t {
    let objs = (0..lv_obj_get_child_count(parent))
        .map(|i| lv_obj_get_child(parent, i as i32))
        .filter(|o| lv_obj_get_x(*o) == x && lv_obj_get_y(*o) == y)
        .collect::<Vec<_>>();

    assert_eq!(objs.len(), 1);
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

#[no_mangle]
extern "C" fn game2048_new() -> *const RefCell<ViewModel> {
    let vm = Rc::new(RefCell::new(ViewModel::new()));

    vm.borrow_mut().effects = vec![{
        downgrade!(vm);
        effect!(move || {
            if let Some(vm) = vm.upgrade() {
                let vm = vm.borrow();
                match *vm.state.get() {
                    State::ClickToStart => unsafe { vm.show_clicktostart(lv_screen_active()) },
                    State::Playing => {
                        *vm.game.borrow_mut() = Game2048::default();
                        unsafe { vm.show_playing(lv_screen_active()) }
                    }
                    State::Win => todo!(),
                    State::GameOver => unsafe { vm.show_gameover(lv_screen_active()) },
                };
            }
        })
    }];

    Rc::into_raw(vm)
}

#[no_mangle]
extern "C" fn game2048_drop(vm: *const RefCell<ViewModel>) {
    let vm = unsafe { Rc::from_raw(vm) };

    let weak_vm = Rc::downgrade(&vm);
    drop(vm);
    assert!(weak_vm.upgrade().is_none());
}
