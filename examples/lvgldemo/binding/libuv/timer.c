#include <nuttx/config.h>
#include <syslog.h>

#ifdef CONFIG_LV_USE_NUTTX_LIBUV
#include "uv.h"

static void rust_wake_and_poll(uv_timer_t *handle)
{
    void rust_delay_wake(void *);
    rust_delay_wake(uv_handle_get_data((uv_handle_t *)handle));

    int rust_executor_wake(void);
    int ret = rust_executor_wake();
    if (ret < 0)
    {
        syslog(LOG_ERR, "A libuv error %s[%d] occurred, which caused the Rust task to be blocked in Delay.await.", uv_err_name(ret), ret);
    }
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

void uv_timer_pending(uv_timer_t *handle, uint64_t timeout, void *state)
{
    uv_handle_set_data((uv_handle_t *)handle, state);
    uv_timer_start(handle, rust_wake_and_poll, timeout, 0);
}
#endif
