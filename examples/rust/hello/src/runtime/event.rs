#[macro_export]
macro_rules! event {
    ($func:ident, $arg:ident, async $(move)? $body:block) => {
        #[no_mangle]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub extern "C" fn $func($arg: *mut $crate::binding::lvgl::lv_event_t) {
            $crate::executor().spawn(async move { $body }).detach();
            $crate::executor().try_tick_all();
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
            $crate::executor().spawn(async move { $body }).detach();
            $crate::executor().try_tick_all();
        }
    };
    ($func:ident, $body:block) => {
        #[no_mangle]
        pub extern "C" fn $func(_: *mut $crate::binding::lvgl::lv_event_t) {
            $body
        }
    };
}
