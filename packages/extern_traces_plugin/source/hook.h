#include <stdbool.h>

#include "elf.h"

void reregister_hooks();
bool register_hooks(JumpSlotRelocationList* relocs, uint16_t static_tls_base);