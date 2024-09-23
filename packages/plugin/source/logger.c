#include <arpa/inet.h>
#include <orbis/libkernel.h>
#include <stdatomic.h>
#include <sys/socket.h>
#include <unistd.h>

#include "logger.h"
#include "time.h"

#include "plugin_common.h"

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

struct BufferState* new_buffer_state(uint64_t size) {
    final_printf("allocating new buffer state @ %lu\n", size);

    void *addr = NULL;
    int ret = sceKernelMapFlexibleMemory(
        &addr,
        size,
        0x02,
        0
    );
    if (ret != 0)
    {
        final_printf("sceKernelMapFlexibleMemory failed: 0x%x\n", ret);
        return NULL;
    }

    struct BufferState* buffer_state = (struct BufferState*)addr;
    buffer_state->write_idx = 0;
    buffer_state->read_idx = 0;
    buffer_state->last_buffer = NULL;
    buffer_state->size = size;

    return buffer_state;
}

static int64_t thread_logging_base = 0;
static int64_t thread_logging_computed_offset = -8;

void set_static_tls_base(uint16_t base) {
    thread_logging_base = base;
    thread_logging_computed_offset = -(int32_t)base - 8;
}

uint64_t read_thread_logging_state_slow()
{
    uint64_t old_value;
    asm volatile(
        "movq %%fs:(%1), %0;"
        : "=r"(old_value)
        : "r"(thread_logging_computed_offset)
        : "memory");
    return old_value;
}

void write_thread_logging_state_slow(uint64_t new_value)
{
    asm volatile(
        "movq %0, %%fs:(%1);"
        :
        : "r"(new_value), "r"(thread_logging_computed_offset)
        : "memory");
}

bool try_write_to_buffer(struct BufferState *state, const uint8_t *data, size_t length)
{
    uint64_t free_space;
    uint64_t read_idx = state->read_idx;
    uint64_t size = state->size - sizeof(struct BufferState);

    if (state->write_idx >= read_idx)
    {
        free_space = size - (state->write_idx - read_idx);
    }
    else
    {
        free_space = read_idx - state->write_idx;
    }

    if (free_space <= length)
    {
        return false;
    }

    size_t end_pos = (state->write_idx + length) % size;
    if (end_pos < state->write_idx)
    {
        size_t first_chunk = size - state->write_idx;
        memcpy(&state->buffer[state->write_idx], data, first_chunk);
        memcpy(state->buffer, data + first_chunk, end_pos);
    }
    else
    {
        memcpy(&state->buffer[state->write_idx], data, length);
    }

    // this operation must occur after writing to buffer completes as updating
    // write_idx marks allows the region to be read
    state->write_idx = end_pos;

    return true;
}

bool write_to_thread_logger(
    struct ThreadLoggingState* thread_state,
    const uint8_t *data,
    size_t length
) {
    struct BufferState* current_buffer = thread_state->current_buffer;
    bool current_buffer_accepted = try_write_to_buffer(current_buffer, data, length);
    if (current_buffer_accepted) {
        return true;
    }

    struct BufferState* next_buffer = new_buffer_state(current_buffer->size * 2);
    if (!next_buffer) {
        return false;
    }

    next_buffer->last_buffer = current_buffer;

    bool next_buffer_accepted = try_write_to_buffer(next_buffer, data, length);
    if (!next_buffer_accepted) {
        final_printf("[INVARIANT VIOLATED] failed to write to new buffer\n");
        return false;
    }

    thread_state->current_buffer = next_buffer;
    return true;
}

void write_to_buffer(
    struct ThreadLoggingState* thread_state,
    const uint8_t *data,
    size_t length
) {
    if (write_to_thread_logger(thread_state, data, length)) {
        return;
    }

    thread_state->dropped_packets_count += 1;
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

ssize_t flush_buffer_contents(struct BufferState *state, int sock) {
    uint64_t write_idx = state->write_idx;
    if (write_idx == state->read_idx)
    {
        return 0;
    }

    if (write_idx > state->read_idx)
    {
        // no wrapping case
        size_t bytes_to_send = write_idx - state->read_idx;

        ssize_t bytes_sent = send_all(sock, &state->buffer[state->read_idx], bytes_to_send);
        if (bytes_sent < 0)
        {
            return bytes_sent;
        }

        state->read_idx = write_idx;
        return bytes_sent;
    }
    else
    {
        // wrapping case (write_idx < read_idx)
        ssize_t total_bytes_sent = 0;
        uint64_t size = state->size - sizeof(struct BufferState);

        {
            // read first half (from read index to end of buffer)
            size_t bytes_to_send_first = size - state->read_idx;
            if (bytes_to_send_first > 0)
            {
                ssize_t bytes_sent = send_all(sock, &state->buffer[state->read_idx], bytes_to_send_first);
                if (bytes_sent < 0)
                {
                    return bytes_sent;
                }

                total_bytes_sent += bytes_sent;
                state->read_idx = 0;
            }
        }

        {
            // read second half (start of buffer to write idx)
            size_t bytes_to_send_second = write_idx;
            if (bytes_to_send_second > 0)
            {
                ssize_t bytes_sent = send_all(sock, &state->buffer[0], bytes_to_send_second);
                if (bytes_sent < 0)
                {
                    return bytes_sent;
                }

                total_bytes_sent += bytes_sent;
                state->read_idx = (state->read_idx + bytes_sent) % size;
            }
        }

        return total_bytes_sent;
    }
}

ssize_t flush_buffer(struct BufferState* state, int sock)
{
    struct BufferState* last_buffer = state->last_buffer;
    if (last_buffer) {
        ssize_t bytes_sent = flush_buffer(last_buffer, sock);
        if (bytes_sent < 0) {
            return bytes_sent;
        }

        sceKernelMunmap(last_buffer, last_buffer->size);
        state->last_buffer = NULL;
    }

    return flush_buffer_contents(state, sock);
}

ssize_t flush_logging_entries(struct ThreadLoggingState *state, int sock)
{
    return flush_buffer(state->current_buffer, sock);
}

struct CountersUpdate
{
    uint64_t message_tag;
    uint64_t thread_id;
    uint64_t dropped_packets_delta;
    uint64_t last_time;
    uint64_t time;
};

ssize_t flush_counters(struct ThreadLoggingState *state, int sock) {
    uint64_t dropped_packets_count = state->dropped_packets_count;
    uint64_t time = get_current_time_rdtscp();

    uint64_t delta = dropped_packets_count - state->last_dropped_packets_count;
    if (delta == 0) {
        state->last_counter_flush_time = time;
        return 0;
    }

    struct CountersUpdate counters_update_message = {
        .message_tag = 2,
        .thread_id = state->thread_id,
        .dropped_packets_delta = delta,
        .last_time = state->last_counter_flush_time,
        .time = time,
    };

    ssize_t bytes_sent = send_all(sock, (const uint8_t *)&counters_update_message, sizeof(struct CountersUpdate));
    if (bytes_sent < 0) {
        return bytes_sent;
    }

    state->last_dropped_packets_count = dropped_packets_count;
    state->last_counter_flush_time = time;

    return bytes_sent;
}

struct InitialMessageHeader {
    /// The number of timestamp steps in a second.
    uint64_t tsc_frequency;

    /// Timestamp in epoch timestamp of the anchor.
    int64_t anchor_seconds;
    int64_t anchor_nanoseconds;

    /// Timestamp from CPU clock of anchor.
    uint64_t anchor_timestamp;
};

void *flush_thread(void *arg)
{
    struct FlushThreadArgs* args = arg;
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock < 0)
    {
        final_printf("socket creation failed\n");
        return NULL;
    }

    struct sockaddr_in server_addr;
    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(args->plugin_config->target_port);

    if (inet_pton(AF_INET, args->plugin_config->target_address, &server_addr.sin_addr) <= 0)
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

    struct InitialMessageHeader initial_message = {
        .tsc_frequency = sceKernelGetTscFrequency(),
        .anchor_timestamp = get_current_time_rdtscp(),
    };

    {
        struct localTimespec {
            int64_t tv_sec;
            int64_t tv_nsec;
        };

        typedef int (*sceKernelClockGettime_func)(int32_t, struct localTimespec*);
        sceKernelClockGettime_func sceKernelClockGettime_impl = (sceKernelClockGettime_func)sceKernelClockGettime;

        struct localTimespec t;

        sceKernelClockGettime_impl(0, &t);

        initial_message.anchor_seconds = t.tv_sec;
        initial_message.anchor_nanoseconds = t.tv_nsec;
    }

    send_all(sock, (uint8_t *)&initial_message, sizeof(struct InitialMessageHeader));

    final_printf("sent initial message header\n");

    uint32_t module_count = args->dynamic_info->modules.size;
    send_all(sock, (uint8_t*)&module_count, sizeof(uint32_t));

    final_printf("sending %d modules\n", module_count);

    for (size_t i = 0; i < module_count; i++) {
        SceModuleValues* module = ((SceModuleValues**)args->dynamic_info->modules.data)[i];
        const char* module_name = args->dynamic_info->strtab + module->module_name_offset;
        uint32_t name_length = strlen(module_name);

        // Send module info
        send_all(sock, (uint8_t*)&module->module_id, sizeof(uint16_t));
        send_all(sock, (uint8_t*)&module->version_major, sizeof(uint8_t));
        send_all(sock, (uint8_t*)&module->version_minor, sizeof(uint8_t));
        send_all(sock, (uint8_t*)&name_length, sizeof(uint32_t));
        send_all(sock, (uint8_t*)module_name, name_length);
    }

    final_printf("sent module metadata\n");

    uint32_t library_count = args->dynamic_info->libraries.size;
    send_all(sock, (uint8_t*)&library_count, sizeof(uint32_t));

    final_printf("sending %d libraries\n", library_count);

    for (size_t i = 0; i < library_count; i++) {
        SceLibValues* library = ((SceLibValues**)args->dynamic_info->libraries.data)[i];
        const char* library_name = args->dynamic_info->strtab + library->library_name_offset;
        uint32_t name_length = strlen(library_name);

        // Send library info
        send_all(sock, (uint8_t*)&library->library_id, sizeof(uint16_t));
        send_all(sock, (uint8_t*)&library->version, sizeof(uint16_t));
        send_all(sock, (uint8_t*)&name_length, sizeof(uint32_t));
        send_all(sock, (uint8_t*)library_name, name_length);
    }

    final_printf("sent library metadata\n");

    // Send jump slot relocation info
    uint32_t symbol_count = args->jump_slot_relocations->count;
    send_all(sock, (uint8_t*)&symbol_count, sizeof(uint32_t));

    final_printf("sending %d symbols\n", symbol_count);

    for (size_t i = 0; i < symbol_count; i++) {
        const JumpSlotRelocation* relocation = &args->jump_slot_relocations->items[i];
        const SymbolInfo* symbol = relocation->symbol_info;

        uint32_t name_length = strlen(symbol->data.parsed.name);

        // Send NID (symbol name)
        send_all(sock, (uint8_t*)&name_length, sizeof(uint32_t));
        send_all(sock, (uint8_t*)symbol->data.parsed.name, name_length);
        
        // Send library ID
        send_all(sock, (uint8_t*)&symbol->data.parsed.library_id, sizeof(uint8_t));
        
        // Send module ID
        send_all(sock, (uint8_t*)&symbol->data.parsed.module_id, sizeof(uint8_t));
    }

    final_printf("sent symbols metadata\n");

    args->is_ready = true;

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

            bytes_sent = flush_counters(state, sock);
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

        // once every 10ms
        sceKernelUsleep(10000);
        // scePthreadYield();
    }

    close(sock);
    return NULL;
}

void fini_thread_local_state()
{
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)read_thread_logging_state_slow();
    state->is_finished = true;
}

struct ThreadLoggingState *init_thread_local_state()
{
    OrbisPthread thread = scePthreadSelf();
    uint64_t thread_id = (uint64_t)thread;

    final_printf("init_thread_local_state\n");

    struct ThreadLoggingState* thread_logging_state = (struct ThreadLoggingState*)malloc(sizeof(struct ThreadLoggingState));
    if (thread_logging_state == NULL)
    {
        final_printf("allocation failed\n");
        return NULL;
    }

    thread_logging_state->is_finished = false;
    thread_logging_state->thread_id = thread_id;
    thread_logging_state->dropped_packets_count = 0;
    thread_logging_state->last_dropped_packets_count = 0;
    thread_logging_state->last_counter_flush_time = 0;

    struct BufferState* buffer_state = new_buffer_state(INITIAL_ALLOCATION_SIZE);
    if (!buffer_state) {
        return NULL;
    }

    thread_logging_state->current_buffer = buffer_state;

    for (size_t i = 0; i < 256; ++i)
    {
        struct ThreadLoggingState *expected = NULL;
        if (atomic_compare_exchange_strong(&global_states[i], &expected, thread_logging_state))
        {
            write_thread_logging_state_slow((uint64_t)thread_logging_state);
            final_printf("written into slot %zu\n", i);
            return thread_logging_state;
        }
    }

    final_printf("no space for state\n");
    write_thread_logging_state_slow((uint64_t)thread_logging_state);
    return thread_logging_state;
}

void destructor_function(void *ptr)
{
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)ptr;
    state->is_finished = true;
}

static OrbisPthreadKey key;

bool init_lazy_destructor()
{
    int ret = scePthreadKeyCreate(&key, destructor_function);
    return ret != 0;
}

struct ThreadLoggingState *lazy_read_value(struct ThreadLoggingState *initial_state)
{
    if (initial_state != NULL)
    {
        return initial_state;
    }

    // attach destructor
    struct ThreadLoggingState* state = init_thread_local_state();
    int ret = scePthreadSetspecific(key, state);
    if (ret != 0)
    {
        final_printf("scePthreadSetspecific failed %d\n", ret);
    }

    return state;
}
