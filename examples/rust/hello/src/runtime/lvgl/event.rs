use std::ptr::null_mut;

use crate::{binding::lvgl::*, runtime::lvgl::LvObjHandle};

struct Closure(Box<dyn Fn(*mut lv_event_t)>);

impl Closure {
    fn new(event_cb: impl Fn(*mut lv_event_t) + 'static) -> Self {
        let boxed = Box::new(event_cb);

        // #[cfg(debug_assertions)]
        // {
        //     let ptr = &*boxed as *const dyn Fn(*mut lv_event_t);
        //     println!("New event Closure at {ptr:p}");
        // }

        Self(boxed)
    }
}

impl Drop for Closure {
    fn drop(&mut self) {
        // #[cfg(debug_assertions)]
        // {
        //     let ptr = &*self.0 as *const dyn Fn(*mut lv_event_t);
        //     println!("Dropping event Closure at {ptr:p}");
        // }
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
    (0..lv_obj_get_event_count(obj))
        .map(|i| lv_obj_get_event_dsc(obj, i))
        .filter(|dsc| std::ptr::fn_addr_eq(lv_event_dsc_get_cb(*dsc), closure_call as lv_event_cb_t))
        .map(|dsc| lv_event_dsc_get_user_data(dsc))
        .map(|closure| Box::from_raw(closure as *mut Closure))
        .for_each(drop);
}

unsafe extern "C" fn closure_call(e: *mut lv_event_t) {
    let closure = lv_event_get_user_data(e) as *mut Closure;
    (*closure).0(e)
}

pub fn add(obj: &LvObjHandle, filter: lv_event_code_t, event_cb: impl Fn(*mut lv_event_t) + 'static) -> *mut lv_event_dsc_t {
    let obj = match obj.try_get() {
        Ok(obj) => obj,
        Err(_) => {
            #[cfg(debug_assertions)]
            panic!();

            #[cfg(not(debug_assertions))]
            return null_mut();
        }
    };

    if filter == LV_EVENT_DELETE {
        // The `closures_cleanup` callback for the `LV_EVENT_DELETE` event
        // must be invoked at the very last moment, as its function is
        // to release memory allocated in Rust by other events.
        //
        // If there are multiple handlers for the `LV_EVENT_DELETE` event,
        // invoking the `closures_cleanup` callback first will lead to
        // memory errors in subsequent callbacks.
        //
        // The correct order is to invoke all other callbacks for the
        // `LV_EVENT_DELETE` event first, then call `closures_cleanup`
        // to clean up the memory, and finally the `lv_obj_t` object will
        // be completely deleted.
        //
        // So, when registering each handler for the `LV_EVENT_DELETE` event,
        // the `closures_cleanup` callback needs to be removed first,
        // and then added again after the event handler function is registered.
        //
        // If LVGL provided a method to move the order of event handler functions,
        // this part of the code could be simplified.
        unsafe { lv_obj_remove_event_cb(obj, closures_cleanup) };

        let closure = Box::into_raw(Box::new(Closure::new(event_cb)));
        let dsc = unsafe { lv_obj_add_event_cb(obj, closure_call, filter, closure as _) };

        unsafe { lv_obj_add_event_cb(obj, closures_cleanup, LV_EVENT_DELETE, null_mut()) };

        dsc
    } else {
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
}

pub fn remove(obj: &LvObjHandle, dsc: *mut lv_event_dsc_t) -> bool {
    let obj = match obj.try_get() {
        Ok(obj) => obj,
        Err(_) => {
            #[cfg(debug_assertions)]
            panic!();

            #[cfg(not(debug_assertions))]
            return false;
        }
    };

    #[cfg(debug_assertions)]
    assert!(!dsc.is_null());

    #[cfg(not(debug_assertions))]
    if dsc.is_null() {
        return false;
    }

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
