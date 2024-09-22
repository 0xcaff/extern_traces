#include "logger.h"

void emit_span_start(uint64_t label_id, struct ThreadLoggingState *initial_state);
void emit_span_end(struct ThreadLoggingState *initial_state);
