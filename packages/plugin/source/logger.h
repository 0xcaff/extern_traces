#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include "elf.h"
#include "config.h"

#define ALLOCATION_SIZE 16 * 1024

struct ThreadLoggingState
{
    uint64_t thread_id;
    uint64_t write_idx;
    uint64_t read_idx;
    bool is_finished;

    uint64_t dropped_packets_count;
    uint64_t last_dropped_packets_count;
    uint64_t last_counter_flush_time;

    uint8_t buffer[];
};

#define BUFFER_SIZE (ALLOCATION_SIZE - sizeof(struct ThreadLoggingState))

struct FlushThreadArgs {
    bool is_ready;
    DynamicInfo* dynamic_info;
    JumpSlotRelocationList* jump_slot_relocations;
    const PluginConfig* plugin_config;
};

void *flush_thread(void *arg);
void fini_thread_local_state();
struct ThreadLoggingState *init_thread_local_state();
bool init_lazy_destructor();
struct ThreadLoggingState *lazy_read_value();
bool write_to_buffer(struct ThreadLoggingState *state, const uint8_t *data, size_t length, uint64_t packets_count);
