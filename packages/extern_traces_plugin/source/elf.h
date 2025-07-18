#pragma once

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
    DT_SCE_JMPREL = 0x61000029,
    DT_SCE_PLTREL = 0x6100002B,
    DT_SCE_PLTRELSZ = 0x6100002D,
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
    ResizableList modules;
    ResizableList libraries;

    const Elf64_Rela* rela_entries;
    size_t rela_count;

    const Elf64_Rela* plt_rela_entries;
    size_t plt_rela_count;

    const SymbolInfo* symbols;
    size_t symbol_count;
    const char* strtab;
    const Elf64_Sym* symtab;
} DynamicInfo;

typedef struct {
    uint8_t magic[8];
    uint8_t category;
    uint8_t program_type;
    uint16_t padding;
    uint16_t header_size;
    uint16_t signature_size;
    uint32_t file_size;
    uint32_t padding2;
    uint16_t segments_count;
    uint16_t padding3[3];
} SELFHeader;

typedef struct {
    uint64_t flags;
    uint64_t offset;
    uint64_t encrypted_compressed_size;
    uint64_t decrypted_decompressed_size;
} SELFSegment;

typedef struct {
    int fd;
    SELFHeader self_header;
    SELFSegment* self_segments;
    Elf64_Ehdr elf_header;
    Elf64_Phdr* phdrs;
} SELFParserState;

SELFParserState* initialize_self_parser(const char* filename);
int get_phdr_index_by_type(const SELFParserState* state, uint32_t type);
void* load_segment(const SELFParserState* state, uint64_t phdr_idx, size_t* size);
void teardown_self_parser(SELFParserState* state);

void cleanup_dynamic_info(DynamicInfo* info);
DynamicInfo parse_dynamic_section(const uint8_t* data, size_t size, const uint8_t* dynlib_segment, size_t dynlib_segment_size);

typedef struct {
    const SymbolInfo* symbol_info;
    uint64_t relocation_offset;
} JumpSlotRelocation;

typedef struct {
    JumpSlotRelocation* items;
    size_t count;
    size_t capacity;
} JumpSlotRelocationList;

void find_jump_slot_relocations(const DynamicInfo* info, JumpSlotRelocationList* result);
void cleanup_jump_slot_relocation_list(JumpSlotRelocationList* list);

struct SpecificSymbolsTable {
    int64_t sceGnmSubmitAndFlipCommandBuffersForWorkload;
    int64_t sceGnmSubmitAndFlipCommandBuffers;
    int64_t sceGnmSubmitCommandBuffers;
    int64_t sceSysmoduleLoadModule;
    int64_t sceAjmBatchJobRunBufferRa;
    int64_t sceAjmBatchJobControlBufferRa;
};

void fill_specific_symbols_table(const JumpSlotRelocationList* list, struct SpecificSymbolsTable* table);