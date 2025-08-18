#include <nuttx/config.h>

#ifdef main
#undef main
#endif
#include <lvgl/lvgl.h>

void event_demo_main(void) {
    lv_obj_t *btn = lv_button_create(lv_screen_active());
    lv_obj_set_align(btn, LV_ALIGN_BOTTOM_MID);
    lv_obj_t *lbl = lv_label_create(btn);
    lv_label_set_text(lbl, "Try a short click or a long press on it!");
    void button_short_clicked_event_demo(lv_event_t *);
    lv_obj_add_event_cb(btn, button_short_clicked_event_demo, LV_EVENT_SHORT_CLICKED, NULL);
    void button_long_pressed_event_demo(lv_event_t *);
    lv_obj_add_event_cb(btn, button_long_pressed_event_demo, LV_EVENT_LONG_PRESSED, NULL);
}
