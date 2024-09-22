#include <stdbool.h>

#include "elf.h"

bool patch_hooks_tls_base(uint16_t static_tls_base);
void register_hooks(JumpSlotRelocationList* relocs, uint16_t static_tls_base);