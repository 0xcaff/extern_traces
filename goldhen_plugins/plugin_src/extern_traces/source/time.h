#include <stdint.h>

static inline uint64_t get_current_time_rdtscp()
{
    unsigned int aux;
    return __builtin_ia32_rdtscp(&aux);
}
