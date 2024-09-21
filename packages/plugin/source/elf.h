#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <elf.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

enum SceDynamicTagType {
    DT_SCE_RELA = 0x6100002F,
    DT_SCE_RELASZ = 0x61000031,
    DT_SCE_RELAENT = 0x61000033,
    DT_SCE_STRTAB = 0x61000035,
    DT_SCE_STRSZ = 0x61000037,
    DT_SCE_SYMTAB = 0x61000039,
    DT_SCE_SYMTABSZ = 0x6100003F,
    DT_SCE_SYMENT = 0x6100003B,
    DT_SCE_IMPORT_LIB = 0x61000015,
    DT_SCE_IMPORT_MODULE = 0x6100000F,
};

typedef struct {
    uint32_t module_name_offset;
    uint8_t version_major;
    uint8_t version_minor;
    uint16_t module_id;
} SceModuleValues;

typedef struct {
    uint32_t library_name_offset;
    uint16_t version;
    uint16_t library_id;
} SceLibValues;

typedef struct {
    void* data;
    size_t size;
    size_t capacity;
} ResizableList;

typedef enum {
    SYMBOL_PARSED,
    SYMBOL_RAW
} SymbolType;

typedef struct {
    SymbolType type;
    union {
        struct {
            char name[12];  // 11 characters + null terminator
            uint8_t library_id;
            uint8_t module_id;
        } parsed;
        const char* raw;
    } data;
} SymbolInfo;

typedef struct {
    uint64_t rela_offset;
    uint64_t rela_size;
    uint64_t rela_ent;
    uint64_t strtab_offset;
    uint64_t strtab_size;
    uint64_t symtab_offset;
    uint64_t symtab_size;
    uint64_t sym_ent;
    ResizableList modules;
    ResizableList libraries;
    const Elf64_Rela* rela_entries;
    size_t rela_count;
    const SymbolInfo* symbols;
    size_t symbol_count;
    const char* strtab;
} DynamicInfo;


void* parse_pt_dynamic(const char* filename, size_t* size);

void cleanup_dynamic_info(DynamicInfo* info);
DynamicInfo parse_dynamic_section(const uint8_t* data, size_t size);
void print_relocations(uint8_t* dynamic_data, size_t dynamic_size, const DynamicInfo* info);