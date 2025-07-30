#include <nuttx/config.h>

#ifdef CONFIG_LV_USE_NUTTX_LIBUV
#include "uv.h"

static void _rust_delay_wake(uv_timer_t *handle)
{
    void rust_delay_wake(void *);
    rust_delay_wake(uv_handle_get_data((uv_handle_t *)handle));

    void rust_executor_pending(void);
    rust_executor_pending();
}

uv_timer_t *uv_timer_new(uv_loop_t *loop)
{
    uv_timer_t *handle = (uv_timer_t *)malloc(sizeof(uv_timer_t));
    uv_timer_init(loop, handle);
    return handle;
}

void uv_timer_drop(uv_timer_t *handle)
{
    free(handle);
}

void rs_delay_start(uv_timer_t *handle, uint64_t timeout, void *state)
{
    uv_handle_set_data((uv_handle_t *)handle, state);
    uv_timer_start(handle, _rust_delay_wake, timeout, 0);
}
#endif
