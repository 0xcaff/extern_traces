#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

#define BUFFER_SIZE (1024 * 1024 * 10)

struct ThreadLoggingState
{
    uint64_t thread_id;
    uint64_t write_idx;
    uint64_t read_idx;
    bool is_finished;
    uint8_t buffer[BUFFER_SIZE];
};

void *flush_thread(void *arg);
void fini_thread_local_state();
struct ThreadLoggingState *init_thread_local_state();
bool init_lazy_destructor();
struct ThreadLoggingState *lazy_read_value();
bool write_to_buffer(struct ThreadLoggingState *state, const uint8_t *data, size_t length);
