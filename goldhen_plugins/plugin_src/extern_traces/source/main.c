#include <stdalign.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "logging.h"
#include "plugin_common.h"

attr_public const char *g_pluginName = "extern_traces";
attr_public const char *g_pluginDesc = "Collects traces for external calls";
attr_public const char *g_pluginAuth = "0xcaff";
attr_public uint32_t g_pluginVersion = 0x00000100; // 1.00

#include "trampolines.h"

s32 attr_module_hidden module_start(s64 argc, const void *args)
{
    final_printf("[GoldHEN] %s Plugin Started.\n", g_pluginName);
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] Plugin Author(s): %s\n", g_pluginAuth);

    initialize_key();
    register_hooks();

    OrbisPthread thread;
    int ret = scePthreadCreate(&thread, NULL, flush_thread_start_routine, NULL, STRINGIFY(flush_thread_start_routine));
    if (ret)
    {
        final_printf("scePthreadCreate failed %x\n", ret);
    }

    return 0;
}

s32 attr_module_hidden module_stop(s64 argc, const void *args)
{
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] %s Plugin Ended.\n", g_pluginName);
    UNHOOK(sceAudioOutInit);
    return 0;
}
