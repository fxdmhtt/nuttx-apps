use std::ptr::null_mut;

use crate::binding::lvgl::{
    lv_event_cb_t, lv_event_code_t, lv_event_dsc_get_cb, lv_event_dsc_get_user_data,
    lv_event_dsc_t, lv_event_get_target, lv_event_get_user_data, lv_event_t, lv_obj_add_event_cb,
    lv_obj_get_event_count, lv_obj_get_event_dsc, lv_obj_remove_event_dsc, lv_obj_t,
    LV_EVENT_DELETE,
};

struct Closure(Box<dyn Fn(*mut lv_event_t)>);

impl Closure {
    fn new(event_cb: impl Fn(*mut lv_event_t) + 'static) -> Self {
        let boxed = Box::new(event_cb);

        let ptr = &*boxed as *const dyn Fn(*mut lv_event_t);
        println!("New Closure at {ptr:p}");

        Self(boxed)
    }
}

impl Drop for Closure {
    fn drop(&mut self) {
        let ptr = &*self.0 as *const dyn Fn(*mut lv_event_t);
        println!("Dropping Closure at {ptr:p}");
    }
}

unsafe extern "C" fn closures_cleanup(e: *mut lv_event_t) {
    let obj = lv_event_get_target(e);

    // This optimization works by deleting all `Closure` objects at once,
    // and then letting LVGL remove all registered events.
    //
    // Important: After the `closures_cleanup` function returns, all registered
    // events must be deleted immediately. Otherwise, state inconsistencies may
    // occur, potentially leading to a crash.
    //
    // In the LVGL source (`lvgl/src/core/lv_obj_tree.c`), this happens around line 507,
    // where `lv_obj_send_event` emits the `LV_EVENT_DELETE` event, and shortly after
    // (line 514), `lv_event_remove_all` deletes all registered events.
    //
    // However, there is a risk of an early return between these calls, which can break
    // the assumption. If necessary, consider calling the `remove` function to clean up
    // events one by one more precisely.
    let _ = (0..lv_obj_get_event_count(obj))
        .map(|i| lv_obj_get_event_dsc(obj, i))
        .filter(|dsc| {
            std::ptr::fn_addr_eq(lv_event_dsc_get_cb(*dsc), closure_call as lv_event_cb_t)
        })
        .map(|dsc| lv_event_dsc_get_user_data(dsc))
        .map(|closure| Box::from_raw(closure as *mut Closure))
        .collect::<Vec<_>>();
}

unsafe extern "C" fn closure_call(e: *mut lv_event_t) {
    let closure = lv_event_get_user_data(e) as *mut Closure;
    (*closure).0(e)
}

pub fn add(
    obj: *mut lv_obj_t,
    filter: lv_event_code_t,
    event_cb: impl Fn(*mut lv_event_t) + 'static,
) -> *mut lv_event_dsc_t {
    if !(0..unsafe { lv_obj_get_event_count(obj) })
        .map(|i| unsafe { lv_obj_get_event_dsc(obj, i) })
        .map(|dsc| unsafe { lv_event_dsc_get_cb(dsc) })
        .any(|cb| std::ptr::fn_addr_eq(cb, closures_cleanup as lv_event_cb_t))
    {
        unsafe { lv_obj_add_event_cb(obj, closures_cleanup, LV_EVENT_DELETE, null_mut()) };
    }

    let closure = Box::into_raw(Box::new(Closure::new(event_cb)));
    unsafe { lv_obj_add_event_cb(obj, closure_call, filter, closure as _) }
}

pub fn remove(obj: *mut lv_obj_t, dsc: *mut lv_event_dsc_t) -> bool {
    if (0..unsafe { lv_obj_get_event_count(obj) })
        .map(|i| unsafe { lv_obj_get_event_dsc(obj, i) })
        .any(|d| d == dsc)
    {
        let closure = unsafe { lv_event_dsc_get_user_data(dsc) as *mut Closure };
        let _ = unsafe { Box::from_raw(closure) };

        unsafe { lv_obj_remove_event_dsc(obj, dsc) }
    } else {
        false
    }
}

#[macro_export]
macro_rules! event_decl {
    ($func:ident, $arg:ident, async $(move)? $body:block) => {
        #[no_mangle]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub extern "C" fn $func($arg: *mut $crate::binding::lvgl::lv_event_t) {
            $crate::runtime::executor()
                .spawn(async move { $body })
                .detach();
            $crate::runtime::executor().try_tick_all();
        }
    };
    ($func:ident, $arg:ident, $body:block) => {
        #[no_mangle]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub extern "C" fn $func($arg: *mut $crate::binding::lvgl::lv_event_t) {
            $body
        }
    };
    ($func:ident, async $(move)? $body:block) => {
        #[no_mangle]
        pub extern "C" fn $func(_: *mut $crate::binding::lvgl::lv_event_t) {
            $crate::runtime::executor()
                .spawn(async move { $body })
                .detach();
            $crate::runtime::executor().try_tick_all();
        }
    };
    ($func:ident, $body:block) => {
        #[no_mangle]
        pub extern "C" fn $func(_: *mut $crate::binding::lvgl::lv_event_t) {
            $body
        }
    };
}
