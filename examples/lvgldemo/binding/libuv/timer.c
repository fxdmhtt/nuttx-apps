#include <nuttx/config.h>
#include <stdlib.h>
#include <assert.h>

#if defined(__GNUC__) || defined(__clang__)
#define UNREACHABLE() do { __builtin_unreachable(); } while(0)
#else
#define UNREACHABLE() assert(0)
#endif

#ifdef CONFIG_LV_USE_NUTTX_LIBUV
#include "uv.h"

#define UV_TIMER_SIZE sizeof(uv_timer_t)
#define UV_TIMER_ALIGN _Alignof(uv_timer_t)

const size_t uv_timer_size = UV_TIMER_SIZE;
const size_t uv_timer_align = UV_TIMER_ALIGN;

#ifdef CONFIG_EXAMPLES_HELLO_RUST_CARGO
static void rust_wake_and_poll(uv_timer_t *handle)
{
    assert(handle != NULL);

    // If a multi-threaded implementation is needed in the future,
    // meaning the callback is not called from the libuv thread,
    // it is necessary to add the state to the thread-safe queue,
    // and then send uv_async_send to consume in the callback
    // and wake up the corresponding Future.
    //
    // However, the limit of reusing uv_async_t is the same type of Future,
    // because different types of Future have different wake-up functions,
    // just like rust_delay_wake.
    void rust_delay_wake(void *);
    rust_delay_wake(uv_handle_get_data((uv_handle_t *)handle));

    // Similarly, waking up the Executor in multi-threaded mode also requires
    // uv_async_t support, but this has already been correctly implemented.
    void rust_executor_wake(void);
    rust_executor_wake();
}
#endif /* CONFIG_EXAMPLES_HELLO_RUST_CARGO */

uv_timer_t *uv_timer_new(uv_loop_t *loop)
{
    uv_timer_t *handle = (uv_timer_t *)malloc(sizeof(uv_timer_t));
    assert(handle != NULL);

    uv_timer_init(loop, handle);
    return handle;
}

void uv_timer_drop(uv_timer_t *handle)
{
    assert(handle != NULL);

    uv_timer_stop(handle);
    uv_close((uv_handle_t *)handle, (void (*)(uv_handle_t *handle))free);
}

#ifdef CONFIG_EXAMPLES_HELLO_RUST_CARGO
void uv_timer_pending(uv_timer_t *handle, uint64_t timeout, void *state)
{
    assert(handle != NULL);
    assert(state != NULL);

    uv_handle_set_data((uv_handle_t *)handle, state);
    uv_timer_start(handle, rust_wake_and_poll, timeout, 0);
}
#endif /* CONFIG_EXAMPLES_HELLO_RUST_CARGO */

void uv_timer_cancel(uv_timer_t *handle)
{
    assert(handle != NULL);

    uv_timer_stop(handle);
}
#endif /* CONFIG_LV_USE_NUTTX_LIBUV */