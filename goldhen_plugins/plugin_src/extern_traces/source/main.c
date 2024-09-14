#include <arpa/inet.h>
#include <pthread.h>
#include <stdalign.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <unistd.h>

#include "plugin_common.h"
#include "thread_local_storage.h"

#define TARGET_IP "192.168.1.100"
#define TARGET_PORT 12345
#define BUFFER_SIZE (1024 * 1024)

attr_public const char *g_pluginName = "extern_traces";
attr_public const char *g_pluginDesc = "collects traces for external calls";
attr_public const char *g_pluginAuth = "0xcaff";
attr_public uint32_t g_pluginVersion = 0x00000100; // 1.00

struct ThreadLoggingState
{
    uint64_t thread_id;
    uint64_t write_idx;
    uint64_t read_idx;
    bool is_finished;
    uint8_t buffer[BUFFER_SIZE];
};

static _Atomic(struct ThreadLoggingState *) global_states[256];

struct ThreadLoggingState *unsafe_read_atomic(volatile _Atomic(struct ThreadLoggingState *) *atomic_ptr)
{
    struct ThreadLoggingState **regular_ptr = (struct ThreadLoggingState **)atomic_ptr;
    return *regular_ptr;
}

void unsafe_write_atomic(volatile _Atomic(struct ThreadLoggingState *) *atomic_ptr, struct ThreadLoggingState *new_value)
{
    struct ThreadLoggingState **regular_ptr = (struct ThreadLoggingState **)atomic_ptr;
    *regular_ptr = new_value;
}

bool write_to_buffer(struct ThreadLoggingState *state, const uint8_t *data, size_t length)
{
    size_t free_space;
    uint64_t read_idx = state->read_idx;

    if (state->write_idx >= read_idx)
    {
        free_space = BUFFER_SIZE - (state->write_idx - read_idx);
    }
    else
    {
        free_space = read_idx - state->write_idx;
    }

    if (free_space <= length)
    {
        return false;
    }

    size_t end_pos = (state->write_idx + length) % BUFFER_SIZE;
    if (end_pos < state->write_idx)
    {
        size_t first_chunk = BUFFER_SIZE - state->write_idx;
        memcpy(&state->buffer[state->write_idx], data, first_chunk);
        memcpy(state->buffer, data + first_chunk, end_pos);
    }
    else
    {
        memcpy(&state->buffer[state->write_idx], data, length);
    }

    state->write_idx = end_pos;

    return true;
}

ssize_t send_all(int sock, const uint8_t *buffer, size_t length)
{
    size_t total_bytes_sent = 0;
    while (total_bytes_sent < length)
    {
        ssize_t bytes_sent = send(sock, buffer + total_bytes_sent, length - total_bytes_sent, 0);

        if (bytes_sent < 0)
        {
            return bytes_sent;
        }

        total_bytes_sent += bytes_sent;
    }

    return total_bytes_sent;
}

ssize_t flush_logging_entries(struct ThreadLoggingState *state, int sock)
{
    uint64_t write_idx = state->write_idx;
    if (write_idx >= state->read_idx)
    {
        // no wrapping case
        size_t bytes_to_send = write_idx - state->read_idx;
        if (bytes_to_send == 0)
        {
            return 0;
        }

        ssize_t bytes_sent = send_all(sock, &state->buffer[state->read_idx], bytes_to_send);
        if (bytes_sent < 0)
        {
            return bytes_sent;
        }

        state->read_idx = (state->read_idx + bytes_sent) % BUFFER_SIZE;
        return bytes_sent;
    }
    else
    {
        // wrapping case (write_idx < read_idx)
        ssize_t total_bytes_sent = 0;
        size_t bytes_to_send_first = BUFFER_SIZE - state->read_idx;
        if (bytes_to_send_first > 0)
        {
            ssize_t bytes_sent = send_all(sock, &state->buffer[state->read_idx], bytes_to_send_first);
            if (bytes_sent < 0)
            {
                return bytes_sent;
            }

            total_bytes_sent += bytes_sent;
            state->read_idx = (state->read_idx + bytes_sent) % BUFFER_SIZE;
        }

        size_t bytes_to_send_second = write_idx;
        if (bytes_to_send_second > 0)
        {
            ssize_t bytes_sent = send_all(sock, &state->buffer[0], bytes_to_send_second);

            if (bytes_sent < 0)
            {
                return bytes_sent;
            }

            total_bytes_sent += bytes_sent;
            state->read_idx = (state->read_idx + bytes_sent) % BUFFER_SIZE;
        }

        return total_bytes_sent;
    }
}

void *flush_thread(void *arg)
{
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0)
    {
        final_printf("socket creation failed\n");
        return NULL;
    }

    struct sockaddr_in server_addr;
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(TARGET_PORT);

    if (inet_pton(AF_INET, TARGET_IP, &server_addr.sin_addr) <= 0)
    {
        final_printf("invalid address\n");
        close(sock);
        return NULL;
    }

    if (connect(sock, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0)
    {
        final_printf("connection failed\n");
        close(sock);
        return NULL;
    }

    while (true)
    {
        for (size_t i = 0; i < 256; ++i)
        {
            struct ThreadLoggingState *state = unsafe_read_atomic(&global_states[i]);

            if (state == NULL)
            {
                continue;
            }

            ssize_t bytes_sent = flush_logging_entries(state, sock);
            if (bytes_sent < 0)
            {
                final_printf("send failed\n");
                close(sock);
                return NULL;
            }

            if (state->is_finished)
            {
                free(state);
                unsafe_write_atomic(&global_states[i], NULL);
                continue;
            }
        }

        scePthreadYield();
    }

    close(sock);
    return NULL;
}

HOOK_INIT(scePthreadCreate);

struct CustomThreadArgs
{
    void *innerArgs;
    void *(*init)(void *);
};

void fini_thread_local_state()
{
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)read_tls_value();
    state->is_finished = true;
}

void init_thread_local_state()
{
    OrbisPthread thread = scePthreadSelf();
    uint64_t thread_id = (uint64_t)thread;

    struct ThreadLoggingState *state = (struct ThreadLoggingState *)malloc(sizeof(struct ThreadLoggingState));
    if (state == NULL)
    {
        return;
    }

    state->is_finished = false;
    state->thread_id = thread_id;
    state->write_idx = 0;
    state->read_idx = 0;

    for (size_t i = 0; i < 256; ++i)
    {
        struct ThreadLoggingState *expected = NULL;
        if (atomic_compare_exchange_strong(&global_states[i], &expected, state))
        {
            write_tls_value((uint64_t)state);
            return;
        }
    }

    write_tls_value((uint64_t)state);
}

void *thread_init_inner(void *rawArgs)
{
    struct CustomThreadArgs *args = rawArgs;

    init_thread_local_state();

    void *result = args->init(args->innerArgs);

    fini_thread_local_state();

    return result;
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

struct Span
{
    uint64_t thread_id;
    uint64_t start_time;
    uint64_t end_time;
    uint64_t label_id;
};

static inline uint64_t get_current_time_rdtscp()
{
    unsigned int aux;
    return __builtin_ia32_rdtscp(&aux);
}

bool emit_span(uint64_t start_time, uint64_t end_time, uint64_t label_id)
{
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)read_tls_value();

    struct Span span = {
        .thread_id = state->thread_id,
        .start_time = start_time,
        .end_time = end_time,
        .label_id = label_id,
    };

    return write_to_buffer(state, (const uint8_t *)&span, sizeof(span));
}

extern int sceSysmoduleLoadModule(uint16_t id);

HOOK_INIT(sceSysmoduleLoadModule);

int sceSysmoduleLoadModule_hook(uint16_t id)
{
    uint64_t start_time = get_current_time_rdtscp();

    int result = HOOK_CONTINUE(
        sceSysmoduleLoadModule,
        int (*)(uint16_t),
        id);

    uint64_t end_time = get_current_time_rdtscp();

    emit_span(start_time, end_time, 1);

    return result;
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
        final_printf("thread create failed %x\n", ret);
    }

    HOOK32(scePthreadCreate);
    HOOK32(sceSysmoduleLoadModule);

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
