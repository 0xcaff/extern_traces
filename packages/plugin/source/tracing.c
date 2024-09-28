#include "logger.h"
#include "time.h"
#include "tracing.h"
#include "plugin_common.h"

static struct SpecificSymbolsTable sharedTable;

void initialize_specific_symbols_table(struct SpecificSymbolsTable* table) {
    sharedTable = *table;
}

struct SpanStart
{
    uint64_t message_tag;
    uint64_t thread_id;
    uint64_t time;
    uint64_t label_id;
};

struct SpanStartAdditionalData
{
    uint64_t message_tag;
    uint64_t thread_id;
    uint64_t time;
    uint64_t label_id;
    uint64_t extra_data_length;
};

struct SpanEnd
{
    uint64_t message_tag;
    uint64_t thread_id;
    uint64_t time;
};

extern void log_pm4(const uint8_t* ptr, uint64_t len);

void emit_span_start(uint64_t label_id, struct ThreadLoggingState* initial_state, struct Args* args) {
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)lazy_read_value(initial_state);
    uint64_t time = get_current_time_rdtscp();

    if (label_id == sharedTable.sceGnmSubmitAndFlipCommandBuffersForWorkload) {
        uint32_t count = (uint32_t)args->args[1];
        uint8_t **draw_buffers = (uint8_t **)args->args[2];
        uint32_t *draw_sizes = (uint32_t *)args->args[3];
        uint8_t **compute_buffers = (uint8_t **)args->args[4];
        uint32_t *compute_sizes = (uint32_t *)args->args[5];

        uint64_t total_size = sizeof(struct SpanStartAdditionalData) + sizeof(uint32_t);
        total_size += count * sizeof(uint32_t) * 2;

        for (uint32_t i = 0; i < count; i++) {
            total_size += draw_sizes[i];
            total_size += compute_sizes[i];
        }

        struct BufferReservation reservation = thread_logging_state_reserve_space(state, total_size);
        if (reservation.buffer == NULL) {
            return;
        }

        struct SpanStartAdditionalData span_header = {
            .message_tag = 3,
            .thread_id = state->thread_id,
            .time = time,
            .label_id = label_id,
            .extra_data_length = total_size - sizeof(struct SpanStartAdditionalData),
        };

        buffer_reservation_write(&reservation, (const uint8_t *)&span_header, sizeof(span_header));
        buffer_reservation_write(&reservation, (const uint8_t *)&count, sizeof(count));
        buffer_reservation_write(&reservation, (const uint8_t *)draw_sizes, count * sizeof(uint32_t));
        buffer_reservation_write(&reservation, (const uint8_t *)compute_sizes, count * sizeof(uint32_t));

        for (uint32_t i = 0; i < count; i++) {
            uint32_t size = draw_sizes[i];
            uint8_t* buffer = draw_buffers[i];

            log_pm4(buffer, (uint64_t)size);
            buffer_reservation_write(&reservation, buffer, size);
        }

        for (uint32_t i = 0; i < count; i++) {
            uint32_t size = compute_sizes[i];
            uint8_t* buffer = compute_buffers[i];

            buffer_reservation_write(&reservation, buffer, size);
        }

        thread_logging_state_flush_reservation(state, reservation);
    } else {
        struct SpanStart span = {
            .message_tag = 0,
            .thread_id = state->thread_id,
            .time = time,
            .label_id = label_id,
        };

        write_to_buffer(state, (const uint8_t *)&span, sizeof(span));
    }
}

void emit_span_end(struct ThreadLoggingState* initial_state) {
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)lazy_read_value(initial_state);
    uint64_t time = get_current_time_rdtscp();

    struct SpanEnd span = {
        .message_tag = 1,
        .thread_id = state->thread_id,
        .time = time,
    };

    write_to_buffer(state, (const uint8_t *)&span, sizeof(span));
}
