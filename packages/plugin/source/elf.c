#include "elf.h"
#include "plugin_common.h"

const unsigned char SELF_MAGIC[8] = { 0x4F, 0x15, 0x3D, 0x1D, 0x00, 0x01, 0x01, 0x12 };

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

void* parse_pt_dynamic(const char* filename, size_t* size) {
    int fd = open(filename, O_RDONLY);
    if (fd == -1) {
        final_printf("Failed to open file: %s\n", filename);
        return NULL;
    }

    SELFHeader self_header;
    if (read(fd, &self_header, sizeof(SELFHeader)) != sizeof(SELFHeader)) {
        final_printf("Failed to read SELF header\n");
        close(fd);
        return NULL;
    }

    if (memcmp(self_header.magic, SELF_MAGIC, 8) != 0) {
        final_printf("Invalid SELF magic\n");
        close(fd);
        return NULL;
    }

    SELFSegment* self_segments = malloc(self_header.segments_count * sizeof(SELFSegment));
    if (!self_segments) {
        final_printf("Failed to allocate memory for SELF segments\n");
        close(fd);
        return NULL;
    }

    if (read(fd, self_segments, self_header.segments_count * sizeof(SELFSegment)) != self_header.segments_count * sizeof(SELFSegment)) {
        final_printf("Failed to read SELF segments\n");
        free(self_segments);
        close(fd);
        return NULL;
    }

    off_t elf_start_offset = lseek(fd, 0, SEEK_CUR);

    Elf64_Ehdr elf_header;
    if (read(fd, &elf_header, sizeof(Elf64_Ehdr)) != sizeof(Elf64_Ehdr)) {
        final_printf("Failed to read ELF header\n");
        free(self_segments);
        close(fd);
        return NULL;
    }

    Elf64_Phdr* phdrs = malloc(elf_header.e_phnum * sizeof(Elf64_Phdr));
    if (!phdrs) {
        final_printf("Failed to allocate memory for program headers\n");
        free(self_segments);
        close(fd);
        return NULL;
    }

    if (lseek(fd, elf_start_offset + elf_header.e_phoff, SEEK_SET) == -1 ||
        read(fd, phdrs, elf_header.e_phnum * sizeof(Elf64_Phdr)) != elf_header.e_phnum * sizeof(Elf64_Phdr)) {
        final_printf("Failed to read program headers\n");
        free(phdrs);
        free(self_segments);
        close(fd);
        return NULL;
    }

    SELFSegment* dynamic_segment = NULL;
    Elf64_Phdr* dynamic_phdr = NULL;
    for (int i = 0; i < self_header.segments_count; i++) {
        uint32_t program_header_id = (self_segments[i].flags >> 20) & 0xFFF;
        if (program_header_id < elf_header.e_phnum && phdrs[program_header_id].p_type == PT_DYNAMIC) {
            dynamic_segment = &self_segments[i];
            dynamic_phdr = &phdrs[program_header_id];
            break;
        }
    }

    if (!dynamic_segment || !dynamic_phdr) {
        final_printf("PT_DYNAMIC segment not found\n");
        free(phdrs);
        free(self_segments);
        close(fd);
        return NULL;
    }

    void* segment_data = malloc(dynamic_segment->decrypted_decompressed_size);
    if (!segment_data) {
        final_printf("Failed to allocate memory for segment data\n");
        free(phdrs);
        free(self_segments);
        close(fd);
        return NULL;
    }

    if (lseek(fd, dynamic_segment->offset, SEEK_SET) == -1 ||
        read(fd, segment_data, dynamic_segment->decrypted_decompressed_size) != dynamic_segment->decrypted_decompressed_size) {
        final_printf("Failed to read segment data\n");
        free(segment_data);
        free(phdrs);
        free(self_segments);
        close(fd);
        return NULL;
    }

    *size = dynamic_segment->decrypted_decompressed_size;
    free(phdrs);
    free(self_segments);
    close(fd);
    return segment_data;
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

DynamicInfo parse_dynamic_section(const uint8_t* data, size_t size) {
    DynamicInfo info = {
        .rela_ent = sizeof(Elf64_Rela),
        .sym_ent = sizeof(Elf64_Sym),

    };
    uint64_t* dynamic = (uint64_t*)data;
    
    init_resizable_list(&info.modules, 10);
    init_resizable_list(&info.libraries, 10);

    for (size_t i = 0; ; i++) {
        uint64_t tag = dynamic[i * 2];
        uint64_t value = dynamic[i * 2 + 1];

        if (tag == DT_NULL) {
            break;  // Stop parsing when we encounter DT_NULL
        }

        final_printf("tag: %lx\n", tag);

        switch (tag) {
            case DT_SCE_RELA:
                info.rela_offset = value;
                break;
            case DT_SCE_RELASZ:
                info.rela_size = value;
                break;
            case DT_SCE_RELAENT:
                info.rela_ent = value;
                if (!(value == sizeof(Elf64_Rela))) {
                    final_printf("SCE_RELAENT invalid: %lu\n", value);
                };
                break;
            case DT_SCE_STRTAB:
                info.strtab_offset = value;
                break;
            case DT_SCE_STRSZ:
                info.strtab_size = value;
                break;
            case DT_SCE_SYMTAB:
                info.symtab_offset = value;
                break;
            case DT_SCE_SYMTABSZ:
                info.symtab_size = value;
                break;
            case DT_SCE_SYMENT:
                info.sym_ent = value;
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

    // Set up relocations
    info.rela_entries = (const Elf64_Rela*)(data + info.rela_offset);
    info.rela_count = info.rela_size / info.rela_ent;

    // Set up string table
    info.strtab = (const char*)(data + info.strtab_offset);

    // Parse symbols
    info.symbol_count = info.symtab_size / info.sym_ent;
    SymbolInfo* symbols = malloc(info.symbol_count * sizeof(SymbolInfo));
    const Elf64_Sym* symtab = (const Elf64_Sym*)(data + info.symtab_offset);

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

void print_relocations(uint8_t* dynamic_data, size_t dynamic_size, const DynamicInfo* info) {
    const Elf64_Sym* symtab = (const Elf64_Sym*)(dynamic_data + info->symtab_offset);

    final_printf("Relocations:\n");
    for (size_t i = 0; i < info->rela_count; i++) {
        const Elf64_Rela* rela = &info->rela_entries[i];
        uint32_t sym_index = ELF64_R_SYM(rela->r_info);
        uint32_t rela_type = ELF64_R_TYPE(rela->r_info);
        const SymbolInfo* sym = &info->symbols[sym_index];
        const Elf64_Sym* elf_sym = &symtab[sym_index];

        const char* library_name = "N/A";
        const char* module_name = "N/A";
        const char* symbol_name = "N/A";

        if (sym->type == SYMBOL_PARSED) {
            library_name = find_library_name(info, sym->data.parsed.library_id);
            module_name = find_module_name(info, sym->data.parsed.module_id);
            symbol_name = sym->data.parsed.name;
        } else {
            symbol_name = sym->data.raw;
        }

        final_printf("Relocation %zu:\n", i);
        final_printf("  Offset: 0x%lx\n", rela->r_offset);
        final_printf("  Symbol: %s\n", symbol_name);
        final_printf("  Symbol Type: %s\n", get_symbol_type_name(elf_sym->st_info));
        final_printf("  Library: %s\n", library_name);
        final_printf("  Module: %s\n", module_name);
        final_printf("  Addend: %ld\n", rela->r_addend);
        final_printf("  Relocation Type: %u\n\n", rela_type);
    }
}