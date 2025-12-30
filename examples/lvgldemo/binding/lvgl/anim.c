#include <nuttx/config.h>
#include <stdlib.h>
#include <assert.h>

#ifdef CONFIG_GRAPHICS_LVGL
#ifdef main
#undef main
#endif
#include <lvgl/lvgl.h>

#ifdef CONFIG_EXAMPLES_HELLO_RUST_CARGO
static void rust_wake_and_poll(lv_anim_t *a)
{
    assert(a != NULL);

    void rust_anim_wake(void *);
    rust_anim_wake(lv_anim_get_user_data(a));

    void rust_executor_wake(void);
    rust_executor_wake();
}
#endif /* CONFIG_EXAMPLES_HELLO_RUST_CARGO */

lv_anim_t *lv_anim_new(void)
{
    lv_anim_t *a = (lv_anim_t *)malloc(sizeof(lv_anim_t));
    assert(a != NULL);

    lv_anim_init(a);
    return a;
}

void lv_anim_drop(lv_anim_t *a)
{
    assert(a != NULL);
    free((void *)a);
}

#ifdef CONFIG_EXAMPLES_HELLO_RUST_CARGO
lv_anim_t *lv_anim_pending(lv_anim_t *a, void *state)
{
    assert(a != NULL);
    assert(state != NULL);

    lv_anim_set_user_data(a, state);
    lv_anim_set_completed_cb(a, (lv_anim_completed_cb_t)rust_wake_and_poll);
    return lv_anim_start(a);
}
#endif /* CONFIG_EXAMPLES_HELLO_RUST_CARGO */

lv_anim_t *lv_anim_query(lv_anim_t *a)
{
    return lv_anim_get(a->var, a->exec_cb);
}

bool lv_anim_cancel(lv_anim_t *a)
{
    assert(a != NULL);

    // lv_anim_t *a = lv_anim_query(a);
    return lv_anim_delete(a->var, a->exec_cb);
}
#endif /* CONFIG_GRAPHICS_LVGL */