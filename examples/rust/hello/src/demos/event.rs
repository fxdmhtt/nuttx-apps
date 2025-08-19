use crate::{
    binding::lvgl::{lv_event_get_code, lv_event_get_target},
    runtime::delay::delay, event,
};

event!(button_short_clicked_event_demo, e, async {
    let code = unsafe { lv_event_get_code(e) };
    let target = unsafe { lv_event_get_target(e) };

    println!("The async event {code:?} on {target:?} is invoking...");
    delay(1).await;
    println!("The async event {code:?} on {target:?} has been invoked!");
});

event!(button_long_pressed_event_demo, {
    println!("The long pressed event has been invoked!");
});
