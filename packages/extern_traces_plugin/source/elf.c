#include "elf.h"
#include "plugin_common.h"

const unsigned char SELF_MAGIC[8] = { 0x4F, 0x15, 0x3D, 0x1D, 0x00, 0x01, 0x01, 0x12 };

#define BLOCK_SEGMENT 0x800

SELFParserState* initialize_self_parser(const char* filename) {
    SELFParserState* state = malloc(sizeof(SELFParserState));
    if (!state) {
        final_printf("failed to allocate memory for parser state\n");
        return NULL;
    }

    state->fd = open(filename, O_RDONLY);
    if (state->fd == -1) {
        final_printf("failed to open file: %s\n", filename);
        free(state);
        return NULL;
    }

    if (read(state->fd, &state->self_header, sizeof(SELFHeader)) != sizeof(SELFHeader)) {
        final_printf("failed to read SELF header\n");
        close(state->fd);
        free(state);
        return NULL;
    }

    if (memcmp(state->self_header.magic, SELF_MAGIC, 8) != 0) {
        final_printf("invalid SELF magic\n");
        close(state->fd);
        free(state);
        return NULL;
    }

    state->self_segments = malloc(state->self_header.segments_count * sizeof(SELFSegment));
    if (!state->self_segments) {
        final_printf("failed to allocate memory for SELF segments\n");
        close(state->fd);
        free(state);
        return NULL;
    }

    if (read(state->fd, state->self_segments, state->self_header.segments_count * sizeof(SELFSegment)) 
        != state->self_header.segments_count * sizeof(SELFSegment)) {
        final_printf("failed to read SELF segments\n");
        free(state->self_segments);
        close(state->fd);
        free(state);
        return NULL;
    }

    off_t elf_start_offset = lseek(state->fd, 0, SEEK_CUR);

    if (read(state->fd, &state->elf_header, sizeof(Elf64_Ehdr)) != sizeof(Elf64_Ehdr)) {
        final_printf("failed to read ELF header\n");
        free(state->self_segments);
        close(state->fd);
        free(state);
        return NULL;
    }

    state->phdrs = malloc(state->elf_header.e_phnum * sizeof(Elf64_Phdr));
    if (!state->phdrs) {
        final_printf("failed to allocate memory for program headers\n");
        free(state->self_segments);
        close(state->fd);
        free(state);
        return NULL;
    }

    if (lseek(state->fd, elf_start_offset + state->elf_header.e_phoff, SEEK_SET) == -1 ||
        read(state->fd, state->phdrs, state->elf_header.e_phnum * sizeof(Elf64_Phdr)) 
        != state->elf_header.e_phnum * sizeof(Elf64_Phdr)) {
        final_printf("Failed to read program headers\n");
        free(state->phdrs);
        free(state->self_segments);
        close(state->fd);
        free(state);
        return NULL;
    }

    return state;
}

int get_phdr_index_by_type(const SELFParserState* state, uint32_t type) {
    for (int i = 0; i < state->elf_header.e_phnum; i++) {
        if (state->phdrs[i].p_type == type) {
            return i;
        }
    }
    return -1;
}

SELFSegment* find_matching_segment(const SELFParserState* state, int phdr_idx) {
    for (int i = 0; i < state->self_header.segments_count; i++) {
        SELFSegment* segment = &state->self_segments[i];
        if (!(segment->flags & BLOCK_SEGMENT)) {
            continue;
        }
        uint32_t program_header_id = (segment->flags >> 20) & 0xFFF;
        if (program_header_id != phdr_idx) {
            continue;
        }
        return segment;
    }

    return NULL;
}

void* load_segment(const SELFParserState* state, uint64_t phdr_idx, size_t* size) {
    unsigned int matching_segment_offset = 0;

    Elf64_Phdr* dynamic_phdr = &state->phdrs[phdr_idx];

    SELFSegment* matching_segment = find_matching_segment(state, phdr_idx);
    if (!matching_segment) {
        final_printf("failed to find matching segment\n");
        return NULL;
    }

    void* segment_data = malloc(dynamic_phdr->p_filesz);
    if (!segment_data) {
        final_printf("failed to allocate memory for segment data\n");
        return NULL;
    }

    if (lseek(state->fd, matching_segment->offset + matching_segment_offset, SEEK_SET) == -1 ||
        read(state->fd, segment_data, dynamic_phdr->p_filesz) != dynamic_phdr->p_filesz) {
        final_printf("failed to read segment data\n");
        free(segment_data);
        return NULL;
    }

    *size = dynamic_phdr->p_filesz;
    return segment_data;
}

void teardown_self_parser(SELFParserState* state) {
    if (state) {
        if (state->fd != -1) {
            close(state->fd);
        }

        free(state->self_segments);
        free(state->phdrs);
        free(state);
    }
}

const uint8_t INDEX_ENCODING_TABLE[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-";

uint8_t decode_index(uint8_t value) {
    for (int i = 0; i < sizeof(INDEX_ENCODING_TABLE) - 1; i++) {
        if (INDEX_ENCODING_TABLE[i] == value) {
            return i;
        }
    }
    return 0xFF;  // Invalid character
}

void init_resizable_list(ResizableList* list, size_t initial_capacity) {
    list->data = malloc(initial_capacity * sizeof(void*));
    list->size = 0;
    list->capacity = initial_capacity;
}

void add_to_resizable_list(ResizableList* list, void* item) {
    if (list->size == list->capacity) {
        list->capacity *= 2;
        list->data = realloc(list->data, list->capacity * sizeof(void*));
    }
    ((void**)list->data)[list->size++] = item;
}

void cleanup_resizable_list(ResizableList* list) {
    for (size_t i = 0; i < list->size; i++) {
        free(((void**)list->data)[i]);
    }
    free(list->data);
    list->data = NULL;
    list->size = 0;
    list->capacity = 0;
}

SymbolInfo parse_symbol_name(const char* name) {
    size_t name_len = strnlen(name, 11);
    
    if (name_len < 11) {
        SymbolInfo info = {.type = SYMBOL_RAW};
        info.data.raw = name;
        return info;
    }

    SymbolInfo info = {.type = SYMBOL_PARSED};
    strncpy(info.data.parsed.name, name, 11);
    info.data.parsed.name[11] = '\0';

    const char* p = name + 11;
    if (*p == '#') {
        p++;
        uint8_t library_id = 0;
        while (*p != '#' && *p != '\0') {
            uint8_t decoded = decode_index(*p);
            if (decoded == 0xFF) {
                info.type = SYMBOL_RAW;
                info.data.raw = name;
                return info;
            }
            library_id = library_id * 64 + decoded;
            p++;
        }
        info.data.parsed.library_id = library_id;

        if (*p == '#') {
            p++;
            uint8_t decoded = decode_index(*p);
            if (decoded == 0xFF) {
                info.type = SYMBOL_RAW;
                info.data.raw = name;
                return info;
            }
            info.data.parsed.module_id = decoded;
        }
    } else {
        info.type = SYMBOL_RAW;
        info.data.raw = name;
    }

    return info;
}

DynamicInfo parse_dynamic_section(const uint8_t* data, size_t size, const uint8_t* dynlib_segment, size_t dynlib_segment_size) {
    DynamicInfo info = {0};

    uint64_t* dynamic = (uint64_t*)data;
    init_resizable_list(&info.modules, 10);
    init_resizable_list(&info.libraries, 10);

    uint64_t rela_offset;
    uint64_t rela_size;

    uint64_t plt_rela_offset;
    uint64_t plt_rela_size;

    uint64_t strtab_offset;
    uint64_t strtab_size;

    uint64_t symtab_offset;
    uint64_t symtab_size;

    for (size_t i = 0; ; i++) {
        uint64_t tag = dynamic[i * 2];
        uint64_t value = dynamic[i * 2 + 1];

        if (tag == DT_NULL) {
            break;
        }

        switch (tag) {
            case DT_SCE_RELA:
                rela_offset = value;
                break;
            case DT_SCE_RELASZ:
                rela_size = value;
                break;
            case DT_SCE_RELAENT:
                if (!(value == sizeof(Elf64_Rela))) {
                    final_printf("SCE_RELAENT invalid: %lu\n", value);
                 };
                break;
            case DT_SCE_JMPREL:
                plt_rela_offset = value;
                break;
            case DT_SCE_PLTRELSZ:
                plt_rela_size = value;
                break;
            case DT_SCE_PLTREL:
                if (value != DT_RELA) {
                    final_printf("SCE_PLTREL is not DT_RELA: %lu\n", value);
                    break;
                }
                break;
            case DT_SCE_STRTAB:
                strtab_offset = value;
                break;
            case DT_SCE_STRSZ:
                strtab_size = value;
                break;
            case DT_SCE_SYMTAB:
                symtab_offset = value;
                break;
            case DT_SCE_SYMTABSZ:
                symtab_size = value;
                break;
            case DT_SCE_SYMENT:
                if (!(value == sizeof(Elf64_Sym))) {
                    final_printf("SCE_SYMENT invalid: %lu\n", value);
                 };
                break;
            case DT_SCE_IMPORT_MODULE: {
                SceModuleValues* module = malloc(sizeof(SceModuleValues));
                module->module_name_offset = value & 0xFFFFFFFF;
                module->version_major = (value >> 32) & 0xFF;
                module->version_minor = (value >> 40) & 0xFF;
                module->module_id = (value >> 48) & 0xFFFF;
                add_to_resizable_list(&info.modules, module);
                break;
            }
            case DT_SCE_IMPORT_LIB: {
                SceLibValues* library = malloc(sizeof(SceLibValues));
                library->library_name_offset = value & 0xFFFFFFFF;
                library->version = (value >> 32) & 0xFFFF;
                library->library_id = (value >> 48) & 0xFFFF;
                add_to_resizable_list(&info.libraries, library);
                break;
            }
        }
    }

    info.rela_entries = (const Elf64_Rela*)(dynlib_segment + rela_offset);
    info.rela_count = rela_size / sizeof(Elf64_Rela);

    info.plt_rela_entries = (const Elf64_Rela*)(dynlib_segment + plt_rela_offset);
    info.plt_rela_count = plt_rela_size / sizeof(Elf64_Rela);

    info.strtab = (const char*)(dynlib_segment + strtab_offset);
    info.symtab = (const Elf64_Sym*)(dynlib_segment + symtab_offset);

    info.symbol_count = symtab_size / sizeof(Elf64_Sym);
    SymbolInfo* symbols = malloc(info.symbol_count * sizeof(SymbolInfo));
    const Elf64_Sym* symtab = (const Elf64_Sym*)(dynlib_segment + symtab_offset);

    for (size_t i = 0; i < info.symbol_count; i++) {
        const char* name = info.strtab + symtab[i].st_name;
        symbols[i] = parse_symbol_name(name);
    }
    info.symbols = symbols;

    return info;
}

void cleanup_dynamic_info(DynamicInfo* info) {
    if (info) {
        cleanup_resizable_list(&info->modules);
        cleanup_resizable_list(&info->libraries);
        free((void*)info->symbols);
        memset(info, 0, sizeof(DynamicInfo));
    }
}

const char* find_library_name(const DynamicInfo* info, uint16_t library_id) {
    for (size_t i = 0; i < info->libraries.size; i++) {
        SceLibValues* lib = ((SceLibValues**)info->libraries.data)[i];
        if (lib->library_id == library_id) {
            return info->strtab + lib->library_name_offset;
        }
    }
    return "Unknown Library";
}

const char* find_module_name(const DynamicInfo* info, uint16_t module_id) {
    for (size_t i = 0; i < info->modules.size; i++) {
        SceModuleValues* module = ((SceModuleValues**)info->modules.data)[i];
        if (module->module_id == module_id) {
            return info->strtab + module->module_name_offset;
        }
    }
    return "Unknown Module";
}

const char* get_symbol_type_name(unsigned char st_info) {
    switch (ELF64_ST_TYPE(st_info)) {
        case STT_NOTYPE: return "NOTYPE";
        case STT_OBJECT: return "OBJECT";
        case STT_FUNC: return "FUNC";
        case STT_SECTION: return "SECTION";
        case STT_FILE: return "FILE";
        case STT_COMMON: return "COMMON";
        case STT_TLS: return "TLS";
        default: return "UNKNOWN";
    }
}

void init_jump_slot_relocation_list(JumpSlotRelocationList* list, size_t initial_capacity) {
    list->items = malloc(initial_capacity * sizeof(JumpSlotRelocation));
    list->count = 0;
    list->capacity = initial_capacity;
}

void add_jump_slot_relocation(JumpSlotRelocationList* list, const SymbolInfo* symbol_info, uint64_t offset) {
    if (list->count == list->capacity) {
        list->capacity *= 2;
        list->items = realloc(list->items, list->capacity * sizeof(JumpSlotRelocation));
    }
    list->items[list->count].symbol_info = symbol_info;
    list->items[list->count].relocation_offset = offset;
    list->count++;
}

void find_jump_slot_relocations(const DynamicInfo* info, JumpSlotRelocationList* result) {
    init_jump_slot_relocation_list(result, 100);

    const Elf64_Rela* relas[] = {info->rela_entries, info->plt_rela_entries};
    size_t rela_counts[] = {info->rela_count, info->plt_rela_count};

    for (int section = 0; section < 2; section++) {
        const Elf64_Rela* current_relas = relas[section];
        size_t current_count = rela_counts[section];

        for (size_t i = 0; i < current_count; i++) {
            const Elf64_Rela* rela = &current_relas[i];
            uint32_t sym_index = ELF64_R_SYM(rela->r_info);
            uint32_t rela_type = ELF64_R_TYPE(rela->r_info);

            if (rela_type != R_X86_64_JUMP_SLOT) {
                continue;
            }

            const SymbolInfo* sym = &info->symbols[sym_index];
            if (rela->r_addend != 0) {
                final_printf("JUMP_SLOT relocation with non-zero addend @ %d, %zu %ld\n", section, i, rela->r_addend);
                continue;
            }

            if (sym->type == SYMBOL_RAW) {
                final_printf("raw symbol encountered @ %d %zu %s", section, i, sym->data.raw);
                continue;
            }

            const char* library_name = find_library_name(info, sym->data.parsed.library_id);
            if (strstr(library_name, "libc") != NULL) {
                continue;
            }

            add_jump_slot_relocation(result, sym, rela->r_offset);
        }
    }
}

void cleanup_jump_slot_relocation_list(JumpSlotRelocationList* list) {
    free(list->items);
    list->items = NULL;
    list->count = 0;
    list->capacity = 0;
}

void fill_specific_symbols_table(const JumpSlotRelocationList* list, struct SpecificSymbolsTable* table) {
    table->sceGnmSubmitAndFlipCommandBuffersForWorkload = -1;
    table->sceGnmSubmitAndFlipCommandBuffers = -1;
    table->sceGnmSubmitCommandBuffers = -1;
    table->sceSysmoduleLoadModule = -1;
    table->sceAjmBatchJobRunBufferRa = -1;
    table->sceAjmBatchJobControlBufferRa = -1;
    table->sceHttpSendRequest = -1;

    for (size_t i = 0; i < list->count; i++) {
        const SymbolInfo* symbol_info = list->items[i].symbol_info;
        
        if (
            strncmp(
                symbol_info->data.parsed.name,
                "Ga6r7H6Y0RI",
                sizeof(symbol_info->data.parsed.name) - 1
            ) == 0
        ) {
            table->sceGnmSubmitAndFlipCommandBuffersForWorkload = i;
        }

        if (
            strncmp(
                symbol_info->data.parsed.name,
                "xbxNatawohc",
                sizeof(symbol_info->data.parsed.name) - 1
            ) == 0
        ) {
            table->sceGnmSubmitAndFlipCommandBuffers = i;
        }

        if (
            strncmp(
                symbol_info->data.parsed.name,
                "zwY0YV91TTI",
                sizeof(symbol_info->data.parsed.name) - 1
            ) == 0
        ) {
            table->sceGnmSubmitCommandBuffers = i;
        }

        if (
            strncmp(
                symbol_info->data.parsed.name,
                "g8cM39EUZ6o",
                sizeof(symbol_info->data.parsed.name) - 1
            ) == 0
        ) {
            table->sceSysmoduleLoadModule = i;
        }

        if (
            strncmp(
                symbol_info->data.parsed.name,
                "ElslOCpOIns",
                sizeof(symbol_info->data.parsed.name) - 1
            ) == 0
        ) {
            table->sceAjmBatchJobRunBufferRa = i;
        }

        if (
            strncmp(
                symbol_info->data.parsed.name,
                "dmDybN--Fn8",
                sizeof(symbol_info->data.parsed.name) - 1
            ) == 0
        ) {
            table->sceAjmBatchJobControlBufferRa = i;
        }

        if (
            strncmp(
                symbol_info->data.parsed.name,
                "1e2BNwI-XzE",
                sizeof(symbol_info->data.parsed.name) - 1
            ) == 0
        ) {
            table->sceHttpSendRequest = i;
        }
    }
}
