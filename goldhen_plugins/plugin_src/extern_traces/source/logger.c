#include <arpa/inet.h>
#include <orbis/libkernel.h>
#include <stdatomic.h>
#include <stddef.h>
#include <sys/socket.h>
#include <unistd.h>

#include "logger.h"
#include "thread_local_storage.h"

#include "plugin_common.h"

#define TARGET_IP "192.168.1.104"
#define TARGET_PORT 9090

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

    // this operation must occur after writing to buffer completes as updating
    // write_idx marks allows the region to be read
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

        {
            // read first half (from read index to end of buffer)
            size_t bytes_to_send_first = BUFFER_SIZE - state->read_idx;
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
                state->read_idx = (state->read_idx + bytes_sent) % BUFFER_SIZE;
            }
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

        sceKernelSleep(1);
        // scePthreadYield();
    }

    close(sock);
    return NULL;
}

void fini_thread_local_state()
{
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)read_tls_value();
    state->is_finished = true;
}

struct ThreadLoggingState *init_thread_local_state()
{
    OrbisPthread thread = scePthreadSelf();
    uint64_t thread_id = (uint64_t)thread;

    struct ThreadLoggingState *state = (struct ThreadLoggingState *)malloc(sizeof(struct ThreadLoggingState));
    if (state == NULL)
    {
        return NULL;
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
            return state;
        }
    }

    final_printf("no space for state\n");
    write_tls_value((uint64_t)state);
    return state;
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

struct ThreadLoggingState *lazy_read_value()
{
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)read_tls_value();
    if (state != NULL)
    {
        return state;
    }

    // attach destructor
    state = init_thread_local_state();
    int ret = scePthreadSetspecific(key, state);
    if (ret != 0)
    {
        final_printf("scePthreadSetspecific failed %d\n", ret);
    }

    return state;
}
