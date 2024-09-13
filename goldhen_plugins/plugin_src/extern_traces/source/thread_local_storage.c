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

bool validate_fs_register() {
    uint64_t fs_value;
    uint64_t fs0_value;

    // Read the value at fs:0 into fs0_value
    __asm__ volatile (
        "movq %%fs:0, %0;"      // Read the QWORD at [fs:0] into fs0_value
        : "=r" (fs0_value)      // Output register for fs0_value
        :                       // No input registers
        : "memory"
    );

    // Read the value of the fs segment register into fs_value
    __asm__ volatile (
        "movq %%fs, %0;"        // Read the fs segment base address into fs_value
        : "=r" (fs_value)       // Output register for fs_value
        :                       // No input registers
        : "memory"
    );

    // Assert that the values are equal
    return fs_value == fs0_value;
}