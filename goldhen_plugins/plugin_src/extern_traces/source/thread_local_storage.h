#include <stdint.h>
#include <stdbool.h>

uint64_t read_tls_value();
void write_tls_value(uint64_t new_value);
bool validate_fs_register();