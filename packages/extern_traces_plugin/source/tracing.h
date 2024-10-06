#include "logger.h"

struct Args {
    uint64_t xmm0[2];
    uint64_t xmm1[2];
    uint64_t xmm2[2];
    uint64_t xmm3[2];
    uint64_t xmm4[2];
    uint64_t xmm5[2];
    uint64_t xmm6[2];
    uint64_t xmm7[2];

    union {
        uint64_t args[6];

        struct {
            uint64_t rdi;
            uint64_t rsi;
            uint64_t rdx;
            uint64_t rcx;
            uint64_t r8;
            uint64_t r9;
        } registers;
    };

    uint64_t extra_args[];
};

void emit_span_start(uint64_t label_id, struct ThreadLoggingState *initial_state, struct Args* args);
void emit_span_end(struct ThreadLoggingState *initial_state);

void initialize_specific_symbols_table(struct SpecificSymbolsTable* table);
void capture_next_submit();