#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include "elf.h"
#include "config.h"

#define INITIAL_ALLOCATION_SIZE (16 * 1024)

struct BufferState {
    uint64_t write_idx;
    uint64_t read_idx;
    struct BufferState* last_buffer;
    uint64_t size;

    uint8_t buffer[];
};

struct ThreadLoggingState
{
    uint64_t thread_id;
    bool is_finished;
    struct BufferState* current_buffer;

    uint64_t dropped_packets_count;
    uint64_t last_dropped_packets_count;
    uint64_t last_counter_flush_time;
};

struct FlushThreadArgs {
    bool is_ready;
    DynamicInfo* dynamic_info;
    JumpSlotRelocationList* jump_slot_relocations;
    int sock;
};

void *flush_thread(void *arg);
void fini_thread_local_state();
struct ThreadLoggingState *init_thread_local_state();
bool init_lazy_destructor();
struct ThreadLoggingState *lazy_read_value();

void set_static_tls_base(uint16_t base);

struct BufferReservation {
    struct BufferState* buffer;
    uint64_t write_idx;
    uint64_t length;
    bool is_new;
};

void write_to_buffer(struct ThreadLoggingState *state, const uint8_t *data, size_t length);

struct BufferReservation thread_logging_state_reserve_space(
    struct ThreadLoggingState* thread_state,
    size_t length
);


void buffer_reservation_write(
    struct BufferReservation* reservation,
    const uint8_t *data,
    size_t length
);

void thread_logging_state_flush_reservation(
    struct ThreadLoggingState* thread_state,
    struct BufferReservation reservation
);
