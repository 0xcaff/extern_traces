#include "thread_local_storage.h"

uint64_t read_tls_value() {
    uint64_t old_value;

    __asm__ volatile (
        "movq %%fs:-8, %0;"
        : "=r" (old_value)
        :
        : "memory"
    );

    return old_value;
}

void write_tls_value(uint64_t new_value) {
    __asm__ volatile (
        "movq %0, %%fs:-8;"
        :
        : "r" (new_value)
        : "memory"
    );
}
