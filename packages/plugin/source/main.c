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
#include "hook.h"
#include "config.h"

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

    struct proc_info procInfo;
    if (sys_sdk_proc_info(&procInfo) != 0) {
        final_printf("Failed to get process info\n");
        return 1;
    }

    PluginConfig config;
    memset(&config, 0, sizeof(PluginConfig));

    if (!load_config(procInfo.titleid, &config)) {
        final_printf("Failed to load configuration for %s\n", procInfo.titleid);
        return 1;
    }

    final_printf("Configuration loaded: target_address=%s, target_port=%d, original_tls_size=%d\n",
                 config.target_address, config.target_port, config.original_tls_size);

    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0)
    {
        final_printf("socket creation failed\n");
        return 1;
    }

    struct sockaddr_in server_addr;
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(config.target_port);

    if (inet_pton(AF_INET, config.target_address, &server_addr.sin_addr) <= 0)
    {
        final_printf("invalid address\n");
        close(sock);
        return 1;
    }

    if (connect(sock, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0)
    {
        final_printf("connection failed\n");
        close(sock);
        return 1;
    }

    SELFParserState* parser = initialize_self_parser("/app0/eboot.bin");

    int tls_phdr_idx = get_phdr_index_by_type(parser, PT_TLS);
    if (tls_phdr_idx == -1) {
        final_printf("no tls segment found\n");
        teardown_self_parser(parser);
        return 1;
    }

    Elf64_Phdr* tls_phdr = &parser->phdrs[tls_phdr_idx];
    if (tls_phdr->p_memsz >= UINT16_MAX) {
        final_printf("static tls segment too large, not supported: %lu\n", tls_phdr->p_memsz);
        teardown_self_parser(parser);
        return 1;
    }

    uint32_t expected_size = 32 + config.original_tls_size;
    if (tls_phdr->p_memsz != expected_size) {
        final_printf("memsz unexpected size, got: %lu, expected: %u\n", tls_phdr->p_memsz, expected_size);
        teardown_self_parser(parser);
        return 1;
    }

    uint16_t tls_base = config.original_tls_size;

    set_static_tls_base(tls_base);

    int dynlib_segment_idx = get_phdr_index_by_type(parser, PT_SCE_DYNLIBDATA);
    if (dynlib_segment_idx == -1) {
        final_printf("dynlib_segment not found\n");
        teardown_self_parser(parser);
        return 1;
    }
    Elf64_Phdr* dynlib_segment_hdr = &parser->phdrs[dynlib_segment_idx];

    size_t dynlib_segment_size;
    void* dynlib_segment = load_segment(parser, dynlib_segment_idx, &dynlib_segment_size);
    if (!dynlib_segment) {
        teardown_self_parser(parser);
        return 1;
    }

    int dynamic_segment_idx = get_phdr_index_by_type(parser, PT_DYNAMIC);
    if (dynamic_segment_idx == -1) {
        final_printf("dynamic_segment not found\n");
        teardown_self_parser(parser);
        free(dynlib_segment);
        return 1;
    }
    Elf64_Phdr* dynamic_segment_hdr = &parser->phdrs[dynamic_segment_idx];
    uint64_t offset = dynamic_segment_hdr->p_offset - dynlib_segment_hdr->p_offset;
    uint64_t dynamic_segment_size = dynamic_segment_hdr->p_filesz;

    teardown_self_parser(parser);

    DynamicInfo info = parse_dynamic_section(dynlib_segment + offset, dynamic_segment_size, dynlib_segment, dynlib_segment_size);

    JumpSlotRelocationList jump_slot_relocations;
    find_jump_slot_relocations(&info, &jump_slot_relocations);
    
    struct SpecificSymbolsTable specific_symbols_table;
    fill_specific_symbols_table(&jump_slot_relocations, &specific_symbols_table);
    initialize_specific_symbols_table(&specific_symbols_table);

    if (!register_hooks(&jump_slot_relocations, tls_base)) {
        cleanup_jump_slot_relocation_list(&jump_slot_relocations);
        cleanup_dynamic_info(&info);

        free(dynlib_segment);
        return 1;
    }

    init_thread_local_state();
    bool ok = init_lazy_destructor();
    if (!ok)
    {
        final_printf("init lazy destructor failed\n");
    }

    struct FlushThreadArgs flush_thread_args = {
        .is_ready = false,
        .dynamic_info = &info,
        .jump_slot_relocations = &jump_slot_relocations,
        .sock = sock,
    };

    OrbisPthread thread;
    int ret = scePthreadCreate(&thread, NULL, flush_thread, &flush_thread_args, STRINGIFY(flush_thread_start_routine));
    if (ret)
    {
        final_printf("thread create failed %x\n", ret);
    }

    // spin lock
    while (!flush_thread_args.is_ready) {
        // once every 100ms
        sceKernelUsleep(100000);
    }

    cleanup_jump_slot_relocation_list(&jump_slot_relocations);
    cleanup_dynamic_info(&info);

    free(dynlib_segment);

    return 0;
}

s32 attr_module_hidden module_stop(s64 argc, const void *args)
{
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] %s Plugin Ended.\n", g_pluginName);

    fini_thread_local_state();

    return 0;
}
