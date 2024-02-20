#include <stdio.h>
#include <stddef.h>
#include <stdint.h>

#include "plugin_common.h"
#include "hook.h"

attr_public const char *g_pluginName = "extern_traces";
attr_public const char *g_pluginDesc = "Collects traces for external calls";
attr_public const char *g_pluginAuth = "0xcaff";
attr_public uint32_t g_pluginVersion = 0x00000100; // 1.00

#define TRAMPOLINE_INIT(FUNC_NAME) \
    HOOK_INIT(FUNC_NAME); \
    extern void* FUNC_NAME(); \
    void* FUNC_NAME##_hook() { \
        RegisterArgsState state; \
        save_args_state(&state); \
        final_printf(#FUNC_NAME "\n"); \
        /* \
         * r10 is not an argument register and will not be clobbered by the \
         * restore_args_state call. \
         */ \
        asm volatile("mov %0, %%r10" : : "m" (Detour_##FUNC_NAME.StubPtr)); \
        /* Retores args and stack, clobbering any locals defined above in this \
         * scope. \
         */ \
        restore_args_state(&state); \
        asm volatile ("call *%%r10" : : ); \
    }

TRAMPOLINE_INIT(sceAudioOutInit);

s32 attr_module_hidden module_start(s64 argc, const void *args)
{
    final_printf("[GoldHEN] %s Plugin Started.\n", g_pluginName);
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] Plugin Author(s): %s\n", g_pluginAuth);

    HOOK32(sceAudioOutInit);

    return 0;
}

s32 attr_module_hidden module_stop(s64 argc, const void *args)
{
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] %s Plugin Ended.\n", g_pluginName);
    UNHOOK(sceAudioOutInit);
    return 0;
}

