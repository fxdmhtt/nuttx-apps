#include <nuttx/config.h>

#ifdef main
#undef main
#endif
#include <lvgl/lvgl.h>

#include "lvgl/examples/assets/img_cogwheel_argb.c"

static lv_obj_t *page;
static lv_style_t style_radio;
static lv_style_t style_radio_chk;
static int32_t active_index = 4;

static void radio_event_handler(lv_event_t *e)
{
    int32_t *active_id = (int32_t *)lv_event_get_user_data(e);
    lv_obj_t *cont = (lv_obj_t *)lv_event_get_current_target(e);
    lv_obj_t *act_cb = lv_event_get_target_obj(e);
    lv_obj_t *old_cb = lv_obj_get_child(cont, *active_id);

    /*Do nothing if the container was clicked*/
    if (act_cb == cont)
        return;

    lv_obj_remove_state(old_cb, LV_STATE_CHECKED); /*Uncheck the previous radio button*/
    lv_obj_add_state(act_cb, LV_STATE_CHECKED);    /*Check the current radio button*/

    *active_id = lv_obj_get_index(act_cb);
}

static void radiobutton_create(lv_obj_t *parent, const char *txt)
{
    lv_obj_t *obj = lv_checkbox_create(parent);
    lv_checkbox_set_text(obj, txt);
    lv_obj_add_flag(obj, LV_OBJ_FLAG_EVENT_BUBBLE);
    lv_obj_add_style(obj, &style_radio, LV_PART_INDICATOR);
    lv_obj_add_style(obj, &style_radio_chk, LV_PART_INDICATOR | LV_STATE_CHECKED);
}

static lv_obj_t *page_create(lv_obj_t *parent, int width, int height)
{
    lv_style_init(&style_radio);
    lv_style_set_radius(&style_radio, LV_RADIUS_CIRCLE);

    lv_style_init(&style_radio_chk);
    lv_style_set_bg_image_src(&style_radio_chk, NULL);

    lv_obj_t *obj;
    char buf[32];

    lv_obj_t *cont = obj = lv_obj_create(parent);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, width, height);
    lv_obj_center(obj);
    lv_obj_set_style_bg_color(obj, lv_color_white(), LV_PART_MAIN);

    lv_obj_t *tier1 = obj = lv_obj_create(obj);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(60));
    lv_obj_set_pos(obj, 0, 0);
    // lv_obj_set_style_bg_color(obj, lv_color_make(0xff, 0, 0), 0);

    lv_obj_t *tier2 = obj = lv_obj_create(cont);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(20));
    lv_obj_align_to(obj, tier1, LV_ALIGN_OUT_BOTTOM_LEFT, 0, 0);
    // lv_obj_set_style_bg_color(obj, lv_color_make(0, 0xff, 0), 0);

    lv_obj_t *tier3 = obj = lv_obj_create(cont);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(20));
    lv_obj_align_to(obj, tier2, LV_ALIGN_OUT_BOTTOM_LEFT, 0, 0);
    // lv_obj_set_style_bg_color(obj, lv_color_make(0, 0, 0xff), 0);

    lv_obj_t *left = obj = lv_obj_create(tier1);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(40), lv_pct(100));
    lv_obj_set_pos(obj, 0, 0);
    // lv_obj_set_style_bg_color(obj, lv_color_make(0xff, 0xff, 0), 0);

    lv_obj_t *right = obj = lv_obj_create(tier1);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(60), lv_pct(100));
    lv_obj_align_to(obj, left, LV_ALIGN_OUT_RIGHT_TOP, 0, 0);
    // lv_obj_set_style_bg_color(obj, lv_color_make(0, 0xff, 0xff), 0);

    lv_obj_t *up = obj = lv_obj_create(left);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(70));
    lv_obj_set_pos(obj, 0, 0);
    // lv_obj_set_style_bg_color(obj, lv_color_make(0x7f, 0xff, 0), 0);

    lv_obj_t *down = obj = lv_obj_create(left);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(30));
    lv_obj_align_to(obj, up, LV_ALIGN_OUT_BOTTOM_LEFT, 0, 0);
    // lv_obj_set_style_bg_color(obj, lv_color_make(0xff, 0x7f, 0), 0);

    LV_IMAGE_DECLARE(img_cogwheel_argb);
    lv_obj_t *img = obj = lv_image_create(up);
    lv_obj_center(obj);
    lv_image_set_src(img, &img_cogwheel_argb);

    lv_obj_t *img_label = obj = lv_label_create(down);
    lv_obj_center(obj);
    lv_label_set_text(img_label, "label");

    lv_obj_t *list = obj = lv_list_create(right);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(100));
    for (int i = 0; i < 5; i++)
    {
        lv_list_add_text(list, "list item");
    }

    lv_obj_t *radio_cont = obj = lv_obj_create(tier2);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(80), LV_SIZE_CONTENT);
    lv_obj_set_align(obj, LV_ALIGN_LEFT_MID);
    lv_obj_set_flex_flow(obj, LV_FLEX_FLOW_ROW);
    lv_obj_set_style_pad_gap(obj, 10, 0);
    lv_obj_add_event_cb(radio_cont, radio_event_handler, LV_EVENT_CLICKED, &active_index);

    lv_snprintf(buf, sizeof(buf), "Red");
    radiobutton_create(radio_cont, buf);
    lv_snprintf(buf, sizeof(buf), "Green");
    radiobutton_create(radio_cont, buf);
    lv_snprintf(buf, sizeof(buf), "Blue");
    radiobutton_create(radio_cont, buf);
    lv_snprintf(buf, sizeof(buf), "Yellow");
    radiobutton_create(radio_cont, buf);
    lv_snprintf(buf, sizeof(buf), "None");
    radiobutton_create(radio_cont, buf);

    lv_obj_add_state(lv_obj_get_child(radio_cont, -1), LV_STATE_CHECKED);

    obj = lv_obj_create(tier2);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(20), LV_SIZE_CONTENT);
    lv_obj_set_align(obj, LV_ALIGN_RIGHT_MID);
    // lv_obj_set_style_bg_color(obj, lv_color_make(0xff, 0, 0xff), 0);
    lv_obj_t *switch_btn = obj = lv_btn_create(obj);
    lv_obj_center(obj);
    lv_obj_t *btn_lbl = lv_label_create(switch_btn);
    lv_label_set_text(btn_lbl, "switch");

    left = obj = lv_obj_create(tier3);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(50), lv_pct(100));
    lv_obj_set_align(obj, LV_ALIGN_LEFT_MID);
    // lv_obj_set_style_bg_color(obj, lv_color_make(0xff, 0xff, 0), 0);

    right = obj = lv_obj_create(tier3);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(50), lv_pct(100));
    lv_obj_set_align(obj, LV_ALIGN_RIGHT_MID);
    // lv_obj_set_style_bg_color(obj, lv_color_make(0, 0xff, 0xff), 0);

    lv_obj_t *btn1 = obj = lv_btn_create(left);
    lv_obj_center(obj);
    btn_lbl = lv_label_create(btn1);
    lv_label_set_text(btn_lbl, "button 1");

    lv_obj_t *btn2 = obj = lv_btn_create(right);
    lv_obj_center(obj);
    btn_lbl = lv_label_create(btn2);
    lv_label_set_text(btn_lbl, "button 2");

    return cont;
}

// static void page_delete(void) {
//     lv_obj_del(page);
// }

void frp_demo_main(void)
{
    page = page_create(lv_screen_active(), 500, 360);
}
