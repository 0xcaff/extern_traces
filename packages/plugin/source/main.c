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

uint64_t read_call_label()
{
    uint64_t old_value;

    __asm__ volatile(
        "movq %%fs:-32, %0;"
        : "=r"(old_value)
        :
        : "memory");

    return old_value;
}

void start_logger() {
    emit_span_start(read_call_label());
}

void end_logger() {
    emit_span_end();
}

__attribute__((naked)) void hook()
{
    asm volatile(
        // backup argument registers
        "push %rdi\n\t"
        "push %rsi\n\t"
        "push %rdx\n\t"
        "push %rcx\n\t"
        "push %r8\n\t"
        "push %r9\n\t"
        "movdqu %xmm0, -0x10(%rsp)\n\t"
        "movdqu %xmm1, -0x20(%rsp)\n\t"
        "movdqu %xmm2, -0x30(%rsp)\n\t"
        "movdqu %xmm3, -0x40(%rsp)\n\t"
        "movdqu %xmm4, -0x50(%rsp)\n\t"
        "movdqu %xmm5, -0x60(%rsp)\n\t"
        "movdqu %xmm6, -0x70(%rsp)\n\t"
        "movdqu %xmm7, -0x80(%rsp)\n\t"
        "sub $0x88, %rsp\n\t"

        // call logger
        "call start_logger\n\t"

        // restore argument registers
        "add $0x88, %rsp\n\t"
        "movdqu -0x10(%rsp), %xmm0\n\t"
        "movdqu -0x20(%rsp), %xmm1\n\t"
        "movdqu -0x30(%rsp), %xmm2\n\t"
        "movdqu -0x40(%rsp), %xmm3\n\t"
        "movdqu -0x50(%rsp), %xmm4\n\t"
        "movdqu -0x60(%rsp), %xmm5\n\t"
        "movdqu -0x70(%rsp), %xmm6\n\t"
        "movdqu -0x80(%rsp), %xmm7\n\t"
        "pop %r9\n\t"
        "pop %r8\n\t"
        "pop %rcx\n\t"
        "pop %rdx\n\t"
        "pop %rsi\n\t"
        "pop %rdi\n\t"

        // backup return address
        // use r10 as a scratch register. it is a caller saved register and not
        // an argument register.
        "pop %r10\n\t"

        // store the value in thread local storage slot. there are no registers
        // which we can use in this context which will both
        // 1. not need to be backed up
        // 2. saved across call boundary
        // callee saved registers are saved across the call boundary but we
        // need to backup for our caller.
        // caller saved registers will not be saved across the call boundary
        // but do not need to be backed up prior to usage
        "movq %r10, %fs:-16\n\t"

        // execute original function
        "movq %fs:-24, %rax\n\t"
        "call *%rax\n\t"

        // restore return address
        "movq %fs:-16, %r10\n\t"
        "push %r10\n\t"

        // backup rax (also happens to align stack for call)
        "push %rax\n\t"

        // call end logger
        "call end_logger\n\t"

        // restore rax (ignore the return value of end logger)
        "pop %rax\n\t"

        "ret\n\t"
    );
}
