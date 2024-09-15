#include <arpa/inet.h>
#include <pthread.h>
#include <stdalign.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/socket.h>

#include "logger.h"
#include "tracing.h"
#include "plugin_common.h"

attr_public const char *g_pluginName = "extern_traces";
attr_public const char *g_pluginDesc = "collects traces for external calls";
attr_public const char *g_pluginAuth = "0xcaff";
attr_public uint32_t g_pluginVersion = 0x00000100; // 1.00

extern int32_t sceAudioOutInit(void);

HOOK_INIT(sceAudioOutInit);

int sceAudioOutInit_hook(void)
{
    emit_span_start(1);

    int result = HOOK_CONTINUE(
        sceAudioOutInit,
        int (*)(void));

    emit_span_end();

    return result;
}

s32 attr_module_hidden module_start(s64 argc, const void *args)
{
    final_printf("[GoldHEN] %s Plugin Started.\n", g_pluginName);
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] Plugin Author(s): %s\n", g_pluginAuth);

    init_thread_local_state();
    bool ok = init_lazy_destructor();
    if (!ok)
    {
        final_printf("init lazy destructor failed\n");
    }

    OrbisPthread thread;
    int ret = scePthreadCreate(&thread, NULL, flush_thread, NULL, STRINGIFY(flush_thread_start_routine));
    if (ret)
    {
        final_printf("thread create failed %x\n", ret);
    }

    HOOK(sceAudioOutInit);

    return 0;
}

s32 attr_module_hidden module_stop(s64 argc, const void *args)
{
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] %s Plugin Ended.\n", g_pluginName);

    fini_thread_local_state();
    UNHOOK(sceAudioOutInit);

    return 0;
}
