#include "logger.h"
#include "time.h"

struct SpanStart
{
    uint64_t message_tag;
    uint64_t thread_id;
    uint64_t time;
    uint64_t label_id;
};

struct SpanEnd
{
    uint64_t message_tag;
    uint64_t thread_id;
    uint64_t time;
};

void emit_span_start(uint64_t label_id) {
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)lazy_read_value();
    uint64_t time = get_current_time_rdtscp();

    struct SpanStart span = {
        .message_tag = 0,
        .thread_id = state->thread_id,
        .time = time,
        .label_id = label_id,
    };

    write_to_buffer(state, (const uint8_t *)&span, sizeof(span));
}

void emit_span_end() {
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)lazy_read_value();
    uint64_t time = get_current_time_rdtscp();

    struct SpanEnd span = {
        .message_tag = 1,
        .thread_id = state->thread_id,
        .time = time,
    };

    write_to_buffer(state, (const uint8_t *)&span, sizeof(span));
}