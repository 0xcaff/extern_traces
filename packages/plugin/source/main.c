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
#include "trampolines.h"
#include "elf.h"

attr_public const char *g_pluginName = "extern_traces";
attr_public const char *g_pluginDesc = "collects traces for external calls";
attr_public const char *g_pluginAuth = "0xcaff";
attr_public uint32_t g_pluginVersion = 0x00000100; // 1.00

#define N_BYTES 256
#define PT_SCE_DYNLIBDATA 0x61000000

s32 attr_module_hidden module_start(s64 argc, const void *args)
{
    final_printf("[GoldHEN] %s Plugin Started.\n", g_pluginName);
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] Plugin Author(s): %s\n", g_pluginAuth);

    {
        SELFParserState* parser = initialize_self_parser("/app0/eboot.bin");

        size_t dynamic_segment_size;
        void* dynamic_segment = load_segment(parser, PT_DYNAMIC, &dynamic_segment_size);
        if (!dynamic_segment) {
            teardown_self_parser(parser);
            return 1;
        }

        size_t dynlib_segment_size;
        void* dynlib_segment = load_segment(parser, PT_SCE_DYNLIBDATA, &dynlib_segment_size);
        if (!dynlib_segment) {
            teardown_self_parser(parser);
            free(dynamic_segment);
            return 1;
        }

        teardown_self_parser(parser);

        DynamicInfo info = parse_dynamic_section(dynamic_segment, dynamic_segment_size, dynlib_segment, dynlib_segment_size);
        print_relocations(&info);
        cleanup_dynamic_info(&info);

        free(dynamic_segment);
        free(dynlib_segment);
    }

    init_thread_local_state();
    bool ok = init_lazy_destructor();
    if (!ok)
    {
        final_printf("init lazy destructor failed\n");
    }

    // OrbisPthread thread;
    // int ret = scePthreadCreate(&thread, NULL, flush_thread, NULL, STRINGIFY(flush_thread_start_routine));
    // if (ret)
    // {
    //     final_printf("thread create failed %x\n", ret);
    // }

    // register_hooks();

    return 0;
}

s32 attr_module_hidden module_stop(s64 argc, const void *args)
{
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] %s Plugin Ended.\n", g_pluginName);

    fini_thread_local_state();

    return 0;
}
