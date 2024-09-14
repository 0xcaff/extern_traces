#include <stdalign.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "plugin_common.h"
#include "thread_local_storage.h"

attr_public const char *g_pluginName = "extern_traces";
attr_public const char *g_pluginDesc = "collects traces for external calls";
attr_public const char *g_pluginAuth = "0xcaff";
attr_public uint32_t g_pluginVersion = 0x00000100; // 1.00

void *flush_thread(void *arg)
{
    return NULL;
}

HOOK_INIT(scePthreadCreate);

struct CustomThreadArgs
{
    void *innerArgs;
    void *(*init)(void *);
};

void *thread_init_inner(void *rawArgs)
{
    struct CustomThreadArgs *args = rawArgs;

    return args->init(args->innerArgs);
}

int32_t scePthreadCreate_hook(
    OrbisPthread *thread,
    const OrbisPthreadAttr *attr,
    void *(*init)(void *),
    void *args,
    const char *name)
{
    struct CustomThreadArgs wrappedArgs = {
        .innerArgs = args,
        .init = init,
    };

    return HOOK_CONTINUE(
        scePthreadCreate,
        int32_t(*)(OrbisPthread *, const OrbisPthreadAttr *, void *(*)(void *), void *, const char *),
        thread, attr, thread_init_inner, &wrappedArgs, name);
}

s32 attr_module_hidden module_start(s64 argc, const void *args)
{
    final_printf("[GoldHEN] %s Plugin Started.\n", g_pluginName);
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] Plugin Author(s): %s\n", g_pluginAuth);

    OrbisPthread thread;
    int ret = scePthreadCreate(&thread, NULL, flush_thread, NULL, STRINGIFY(flush_thread_start_routine));
    if (ret)
    {
        final_printf("[extern_traces] scePthreadCreate failed %x\n", ret);
    }

    HOOK32(scePthreadCreate);

    return 0;
}

s32 attr_module_hidden module_stop(s64 argc, const void *args)
{
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] %s Plugin Ended.\n", g_pluginName);

    UNHOOK(scePthreadCreate);

    return 0;
}
