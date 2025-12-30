#include <nuttx/config.h>

#ifdef CONFIG_INTERPRETERS_WAMR
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "wasm_export.h"

#include "wasm.hex"
#define wasm_buf __target_wasm32_unknown_unknown_release_demo_wasm_aot
#define wasm_size __target_wasm32_unknown_unknown_release_demo_wasm_aot_len

void wasm_demo_main(void)
{
    char error_buf[0xff];
    memset(error_buf, 0, sizeof(error_buf));
    wasm_module_t module = wasm_runtime_load((uint8_t *)wasm_buf, wasm_size, error_buf, sizeof(error_buf));
    if (!module)
    {
        printf("load failed: %s\n", error_buf);
        return;
    }

    uint32_t stack_size = 8 * 1024;
    uint32_t heap_size = 8 * 1024;
    wasm_module_inst_t inst = wasm_runtime_instantiate(module, stack_size, heap_size, error_buf, sizeof(error_buf));
    if (!inst)
    {
        printf("instantiate failed: %s\n", error_buf);
        wasm_runtime_unload(module);
        return;
    }

    struct WASMExecEnv *exec_env = wasm_runtime_get_exec_env_singleton(inst);
    if (!exec_env)
    {
        printf("get exec_env failed\n");
        goto cleanup;
    }

    wasm_function_inst_t func = wasm_runtime_lookup_function(inst, "demo_wasm_hello");
    if (!func)
    {
        printf("lookup function failed\n");
        goto cleanup;
    }

    wasm_val_t args[0];
    wasm_val_t ret;
    if (!wasm_runtime_call_wasm_a(exec_env, func, 0, &ret, 0, args))
    {
        const char *exception = wasm_runtime_get_exception(inst);
        printf("call failed: %s\n", exception ? exception : "unknown");
        goto cleanup;
    }

cleanup:
    wasm_runtime_deinstantiate(inst);
    wasm_runtime_unload(module);
}
#endif /* CONFIG_INTERPRETERS_WAMR */