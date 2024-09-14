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

    init_thread_local_state();

    void* result = args->init(args->innerArgs);

    fini_thread_local_state();

    return result;
}

struct ThreadLoggingState {
    uint64_t thread_id;
    uint64_t write_idx;
    bool is_finished;
    uint8_t buffer[1024 * 1024];
};

void init_thread_local_state() {
    OrbisPthread thread = scePthreadSelf();
    uint64_t thread_id = thread;

    struct ThreadLoggingState* state = (struct ThreadLoggingState*) malloc(sizeof(struct ThreadLoggingState));

    if (state == NULL) {
        return;
    }

    OrbisPthread thread = scePthreadSelf();
    state->is_finished = false;
    state->thread_id = (uint64_t)thread;
    state->write_idx = 0;

    write_tls_value((uint64_t)state);
}

void fini_thread_local_state() {
    struct ThreadLoggingState* state = read_tls_value();
    state->is_finished = true;
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

    init_thread_local_state();

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

    fini_thread_local_state();

    UNHOOK(scePthreadCreate);

    return 0;
}
