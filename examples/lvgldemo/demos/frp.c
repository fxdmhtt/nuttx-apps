#include <nuttx/config.h>

#ifdef CONFIG_EXAMPLES_HELLO_RUST_CARGO
#ifdef main
#undef main
#endif
#include <lvgl/lvgl.h>

#include "lvgl/examples/assets/img_cogwheel_argb.c"

static lv_style_t style_radio;
static lv_style_t style_radio_chk;

lv_obj_t *page;
lv_obj_t *_radio_cont;
lv_obj_t *_img;
lv_obj_t *_img_label;
lv_obj_t *_no_color_btn;
lv_obj_t *_btn1;
lv_obj_t *_btn2;
lv_obj_t *_list;
lv_obj_t *_slider;

int32_t active_index_get(void);
bool active_index_set(int32_t);

void switch_color_event(lv_event_t *e);
void intense_inc_event(lv_event_t *e);
void intense_dec_or_clear_event(lv_event_t *e);
void list_item_changed_event(lv_event_t *e);

lv_color_t lv_color_make_rs(uint8_t r, uint8_t g, uint8_t b)
{
    return lv_color_make(r, g, b);
}

lv_obj_t *create_list_item(lv_obj_t *parent, const char *text)
{
    lv_obj_t *item = lv_list_add_text(_list, text);
    lv_obj_add_flag(item, LV_OBJ_FLAG_CLICKABLE);
    lv_obj_set_style_bg_color(item, lv_color_white(), 0);
    return item;
}

lv_obj_t *create_list_hint(void)
{
    lv_coord_t w = lv_obj_get_width(_list);
    lv_coord_t h = lv_obj_get_height(_list);
    lv_coord_t x = lv_obj_get_x(_list);
    lv_coord_t y = lv_obj_get_y(_list);

    lv_obj_t *cont = lv_obj_create(lv_obj_get_parent(_list));
    lv_obj_set_size(cont, w, h);
    lv_obj_set_pos(cont, x, y);
    lv_obj_t *hint = lv_label_create(cont);
    lv_label_set_text(hint, "Empty!");
    lv_obj_center(hint);

    lv_obj_set_style_bg_color(hint, lv_color_white(), LV_PART_MAIN);
    lv_obj_set_style_text_color(hint, lv_color_hex(0x888888), LV_PART_MAIN);
    lv_obj_set_style_text_align(hint, LV_TEXT_ALIGN_CENTER, LV_PART_MAIN);
    lv_obj_set_style_pad_all(hint, 5, LV_PART_MAIN);

    return cont;
}

static void radio_event_handler(lv_event_t *e)
{
    lv_obj_t *cont = (lv_obj_t *)lv_event_get_current_target(e);
    lv_obj_t *act_cb = lv_event_get_target_obj(e);

    /*Do nothing if the container was clicked*/
    if (act_cb == cont)
        return;

    lv_obj_add_state(act_cb, LV_STATE_CHECKED); /*Ensure the current radio button is checked (multiclick)*/

    active_index_set(lv_obj_get_index(act_cb));
}

static lv_obj_t *radiobutton_create(lv_obj_t *parent, const char *txt)
{
    lv_obj_t *obj = lv_checkbox_create(parent);
    lv_checkbox_set_text(obj, txt);
    lv_obj_add_flag(obj, LV_OBJ_FLAG_EVENT_BUBBLE);
    lv_obj_add_style(obj, &style_radio, LV_PART_INDICATOR);
    lv_obj_add_style(obj, &style_radio_chk, LV_PART_INDICATOR | LV_STATE_CHECKED);
    return obj;
}

static lv_obj_t *create_slider(lv_obj_t *parent, lv_color_t color)
{
    lv_obj_t *obj = lv_slider_create(parent);
    lv_slider_set_range(obj, 0, 255);
    lv_obj_set_size(obj, 10, LV_PCT(60));
    lv_obj_set_style_bg_color(obj, color, LV_PART_KNOB);
    lv_obj_set_style_bg_color(obj, lv_color_darken(color, LV_OPA_40), LV_PART_INDICATOR);
    return obj;
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
    lv_obj_set_style_outline_width(obj, 2, LV_PART_MAIN);

    lv_obj_t *tier1 = obj = lv_obj_create(obj);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(60));
    lv_obj_set_pos(obj, 0, 0);

    lv_obj_t *tier2 = obj = lv_obj_create(cont);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(20));
    lv_obj_align_to(obj, tier1, LV_ALIGN_OUT_BOTTOM_LEFT, 0, 0);

    lv_obj_t *tier3 = obj = lv_obj_create(cont);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(20));
    lv_obj_align_to(obj, tier2, LV_ALIGN_OUT_BOTTOM_LEFT, 0, 0);

    lv_obj_t *left = obj = lv_obj_create(tier1);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(30), lv_pct(100));
    lv_obj_set_pos(obj, 0, 0);

    lv_obj_t *middle = obj = lv_obj_create(tier1);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(10), lv_pct(100));
    lv_obj_align_to(obj, left, LV_ALIGN_OUT_RIGHT_TOP, 0, 0);

    lv_obj_t *right = obj = lv_obj_create(tier1);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(60), lv_pct(100));
    lv_obj_align_to(obj, middle, LV_ALIGN_OUT_RIGHT_TOP, 0, 0);

    lv_obj_t *up = obj = lv_obj_create(left);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(70));
    lv_obj_set_pos(obj, 0, 0);

    lv_obj_t *down = obj = lv_obj_create(left);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(30));
    lv_obj_align_to(obj, up, LV_ALIGN_OUT_BOTTOM_LEFT, 0, 0);

    LV_IMAGE_DECLARE(img_cogwheel_argb);
    _img = obj = lv_image_create(up);
    lv_obj_center(obj);
    lv_image_set_src(_img, &img_cogwheel_argb);

    _img_label = obj = lv_label_create(down);
    lv_obj_center(obj);

    _slider = obj = create_slider(middle, lv_palette_main(LV_PALETTE_GREY));
    lv_obj_center(obj);

    _list = obj = lv_list_create(right);
    lv_obj_set_size(obj, lv_pct(100), lv_pct(100));
    lv_obj_add_event_cb(_list, list_item_changed_event, LV_EVENT_CHILD_CHANGED, 0);

    _radio_cont = obj = lv_obj_create(tier2);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(80), LV_SIZE_CONTENT);
    lv_obj_set_align(obj, LV_ALIGN_LEFT_MID);
    lv_obj_set_flex_flow(obj, LV_FLEX_FLOW_ROW);
    lv_obj_set_style_pad_gap(obj, 10, 0);
    lv_obj_add_event_cb(_radio_cont, radio_event_handler, LV_EVENT_CLICKED, NULL);

    lv_snprintf(buf, sizeof(buf), "Red");
    radiobutton_create(_radio_cont, buf);
    lv_snprintf(buf, sizeof(buf), "Green");
    radiobutton_create(_radio_cont, buf);
    lv_snprintf(buf, sizeof(buf), "Blue");
    radiobutton_create(_radio_cont, buf);
    lv_snprintf(buf, sizeof(buf), "Yellow");
    radiobutton_create(_radio_cont, buf);
    lv_snprintf(buf, sizeof(buf), "None");
    _no_color_btn = radiobutton_create(_radio_cont, buf);

    obj = lv_obj_create(tier2);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(20), LV_SIZE_CONTENT);
    lv_obj_set_align(obj, LV_ALIGN_RIGHT_MID);
    lv_obj_t *switch_btn = obj = lv_btn_create(obj);
    lv_obj_center(obj);
    lv_obj_add_event_cb(switch_btn, switch_color_event, LV_EVENT_SHORT_CLICKED, NULL);
    lv_obj_t *btn_lbl = lv_label_create(switch_btn);
    lv_label_set_text(btn_lbl, "switch");

    left = obj = lv_obj_create(tier3);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(50), lv_pct(100));
    lv_obj_set_align(obj, LV_ALIGN_LEFT_MID);

    right = obj = lv_obj_create(tier3);
    lv_obj_set_style_pad_all(obj, 0, LV_PART_MAIN);
    lv_obj_set_style_border_width(obj, 0, LV_PART_MAIN);
    lv_obj_set_size(obj, lv_pct(50), lv_pct(100));
    lv_obj_set_align(obj, LV_ALIGN_RIGHT_MID);

    _btn1 = obj = lv_btn_create(left);
    lv_obj_set_size(obj, 120, 40);
    lv_obj_center(obj);
    lv_obj_add_event_cb(obj, intense_inc_event, LV_EVENT_SHORT_CLICKED, 0);
    btn_lbl = lv_label_create(_btn1);
    lv_obj_center(btn_lbl);
    lv_label_set_text(btn_lbl, "Intense Inc");

    _btn2 = obj = lv_btn_create(right);
    lv_obj_set_size(obj, 120, 40);
    lv_obj_center(obj);
    lv_obj_add_event_cb(obj, intense_dec_or_clear_event, LV_EVENT_SHORT_CLICKED, 0);
    btn_lbl = lv_label_create(_btn2);
    lv_obj_center(btn_lbl);
    lv_label_set_text(btn_lbl, "button 2");

    return cont;
}

static void page_delete(void)
{
    lv_style_reset(&style_radio);
    lv_style_reset(&style_radio_chk);

    lv_obj_del(page);
}

static void frp_demo_launcher(lv_event_t *e)
{
    static bool running = false;

    lv_obj_t *lbl = lv_event_get_user_data(e);

    if (running)
    {
        void frp_demo_rs_drop(void);
        frp_demo_rs_drop();

        page_delete();
        page = NULL;

        lv_label_set_text(lbl, "Start FRP demo");
        running = false;
    }
    else
    {
        page = page_create(lv_screen_active(), 500, 360);

        void frp_demo_rs_init(void);
        frp_demo_rs_init();

        lv_obj_add_state(lv_obj_get_child(_radio_cont, active_index_get()), LV_STATE_CHECKED);

        lv_label_set_text(lbl, "Stop FRP demo");
        running = true;
    }
}

static void create_launcher(void)
{
    lv_obj_t *btn = lv_button_create(lv_screen_active());
    lv_obj_set_align(btn, LV_ALIGN_TOP_MID);
    lv_obj_t *lbl = lv_label_create(btn);
    lv_label_set_text(lbl, "Start FRP demo");
    lv_obj_add_event_cb(btn, frp_demo_launcher, LV_EVENT_SHORT_CLICKED, lbl);
}

void frp_demo_main(void)
{
    create_launcher();
}
#endif /* CONFIG_EXAMPLES_HELLO_RUST_CARGO */