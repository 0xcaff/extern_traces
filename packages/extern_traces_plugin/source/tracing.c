#include "logger.h"
#include "time.h"
#include "tracing.h"
#include "plugin_common.h"
#include "hook.h"

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

static bool should_capture_next_submit = false;

void capture_next_submit() {
    should_capture_next_submit = true;
}

void emit_span_start(uint64_t label_id, struct ThreadLoggingState* initial_state, struct Args* args) {
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)lazy_read_value(initial_state);
    state->last_label_id = label_id;
    uint64_t time = get_current_time_rdtscp();
    if (label_id == sharedTable.sceGnmSubmitAndFlipCommandBuffersForWorkload && should_capture_next_submit) {
        should_capture_next_submit = false;

        sceGnmSubmitAndFlipCommandBuffersForWorkload_trace(
            args, state, time, label_id, state->thread_id
        );
    } else if (label_id == sharedTable.sceGnmSubmitAndFlipCommandBuffers && should_capture_next_submit) {
        should_capture_next_submit = false;

        sceGnmSubmitAndFlipCommandBuffers_trace(
            args, state, time, label_id, state->thread_id
        );

    } else if (label_id == sharedTable.sceGnmSubmitCommandBuffers && should_capture_next_submit) {
        should_capture_next_submit = false;

        sceGnmSubmitCommandBuffers_trace(
            args, state, time, label_id, state->thread_id
        );
    } else if (label_id == sharedTable.sceSysmoduleLoadModule) {
        sceSysmoduleLoadModule_trace(args);

        struct SpanStart span = {
            .message_tag = 0,
            .thread_id = state->thread_id,
            .time = time,
            .label_id = label_id,
        };

        write_to_buffer(state, (const uint8_t *)&span, sizeof(span));
    } else if (label_id == sharedTable.sceAjmBatchJobRunBufferRa) {
        sceAjmBatchJobRunBufferRa_trace(args, state, time, label_id, state->thread_id);
    } else if (label_id == sharedTable.sceAjmBatchJobControlBufferRa) {
        sceAjmBatchJobControlBufferRa_trace(args, state, time, label_id, state->thread_id);
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

void emit_span_end(struct ThreadLoggingState* initial_state, void* return_value) {
    struct ThreadLoggingState *state = (struct ThreadLoggingState *)lazy_read_value(initial_state);
    uint64_t time = get_current_time_rdtscp();

    if (state->last_label_id == sharedTable.sceSysmoduleLoadModule) {
        reregister_hooks();
    }

    struct SpanEnd span = {
        .message_tag = 1,
        .thread_id = state->thread_id,
        .time = time,
    };

    write_to_buffer(state, (const uint8_t *)&span, sizeof(span));
}
