use crate::{
    binding::lvgl::{lv_event_get_code, lv_event_get_target},
    delay, event_decl,
};

event_decl!(button_short_clicked_event_demo, e, async {
    let code = unsafe { lv_event_get_code(e) };
    let target = unsafe { lv_event_get_target(e) };

    println!("The async event {code:?} on {target:?} is invoking...");
    let _ = delay!(1).await;
    println!("The async event {code:?} on {target:?} has been invoked!");
});

event_decl!(button_long_pressed_event_demo, {
    println!("The long pressed event has been invoked!");
});
