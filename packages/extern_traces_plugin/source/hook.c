#include "hook.h"
#include "plugin_common.h"
#include "tracing.h"

#include <stdlib.h>
#include <orbis/libkernel.h>

__attribute__((naked)) void hook()
{
    asm volatile(
        // backup argument registers
        "push %r9\n\t"
        "push %r8\n\t"
        "push %rcx\n\t"
        "push %rdx\n\t"
        "push %rsi\n\t"
        "push %rdi\n\t"

        "movdqu %xmm0, -0x80(%rsp)\n\t"
        "movdqu %xmm1, -0x70(%rsp)\n\t"
        "movdqu %xmm2, -0x60(%rsp)\n\t"
        "movdqu %xmm3, -0x50(%rsp)\n\t"
        "movdqu %xmm4, -0x40(%rsp)\n\t"
        "movdqu %xmm5, -0x30(%rsp)\n\t"
        "movdqu %xmm6, -0x20(%rsp)\n\t"
        "movdqu %xmm7, -0x10(%rsp)\n\t"
        "sub $0x88, %rsp\n\t"

        // call logger
        "movl %fs:-32, %edi\n\t"
        "movq %fs:-8, %rsi\n\t"
        "lea 0x8(%rsp), %rdx\n\t"
        "call emit_span_start\n\t"
        "nop\n\t"

        // restore argument registers
        "add $0x88, %rsp\n\t"
        "movdqu -0x80(%rsp), %xmm0\n\t"
        "movdqu -0x70(%rsp), %xmm1\n\t"
        "movdqu -0x60(%rsp), %xmm2\n\t"
        "movdqu -0x50(%rsp), %xmm3\n\t"
        "movdqu -0x40(%rsp), %xmm4\n\t"
        "movdqu -0x30(%rsp), %xmm5\n\t"
        "movdqu -0x20(%rsp), %xmm6\n\t"
        "movdqu -0x10(%rsp), %xmm7\n\t"

        "pop %rdi\n\t"
        "pop %rsi\n\t"
        "pop %rdx\n\t"
        "pop %rcx\n\t"
        "pop %r8\n\t"
        "pop %r9\n\t"

        // backup return address
        // use r10 as a scratch register. it is a caller saved register and not
        // an argument register.
        "pop %r10\n\t"

        // store the value in thread local storage slot. there are no registers
        // which we can use in this context which will both
        // 1. not need to be backed up
        // 2. saved across call boundary
        // callee saved registers are saved across the call boundary but we
        // need to backup for our caller.
        // caller saved registers will not be saved across the call boundary
        // but do not need to be backed up prior to usage
        "movq %r10, %fs:-16\n\t"

        // execute original function
        "movq %fs:-24, %rax\n\t"
        "call *%rax\n\t"

        // restore return address
        "movq %fs:-16, %r10\n\t"
        "push %r10\n\t"

        // backup rax (also happens to align stack for call)
        "push %rax\n\t"

        // call end logger
        "movq %fs:-8, %rdi\n\t"
        "movq %rax, %rsi\n\t"
        "call emit_span_end\n\t"

        // extra nop instruction to patch call
        "nop\n\t"

        // restore rax (ignore the return value of end logger)
        "pop %rax\n\t"

        "ret\n\t"
    );
}

#define PAGE_SIZE 4096
#define HOOK_FN_SIZE 0xd0

void* build_hook_fn(uint16_t static_tls_base) {
    size_t required_size = HOOK_FN_SIZE + 16;

    size_t total_size = (required_size + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1);
    
    void* new_mem = mmap(NULL, total_size, PROT_READ | PROT_WRITE, 
                         MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    
    if (new_mem == MAP_FAILED) {
        final_printf("failed to map memory for patched hook function\n");
        return NULL;
    }

    final_printf("hook fn block: 0x%lx\n", (uint64_t)new_mem);

    memcpy(new_mem, (void*)hook, HOOK_FN_SIZE);

    typedef struct {
        uint64_t offset;
        int32_t expected_value;
    } ThreadLocalStoragePatches;

    ThreadLocalStoragePatches patches[] = {
        {0x3f + 4, -32},
        {0x47 + 5, -8},
        {0x9c + 5, -16},
        {0xa5 + 5, -24},
        {0xb0 + 5, -16},
        {0xbc + 5, -8},
    };
    size_t num_patches = sizeof(patches) / sizeof(ThreadLocalStoragePatches);

    for (size_t i = 0; i < num_patches; i++) {
        int32_t* target_ptr = (int32_t*)((char*)new_mem + patches[i].offset);

        int32_t existing_value = *target_ptr;
        if (existing_value != patches[i].expected_value) {
            final_printf("failed to patch @ idx = %zu, unexpected value, %x, expected: %x\n", i, existing_value, patches[i].expected_value);
            munmap(new_mem, total_size);
            return NULL;
        }

        *target_ptr = existing_value - (int32_t)static_tls_base;
    }

    {
        uint64_t instruction_start = 0x55;
        uint32_t displacement = (HOOK_FN_SIZE) - (instruction_start + 6);
        final_printf("displacement 1: %d\n", displacement);
        unsigned char patch[] = {0xFF, 0x15, 0x00, 0x00, 0x00, 0x00};
        memcpy(&patch[2], &displacement, sizeof(displacement));
        memcpy(new_mem + instruction_start, patch, sizeof(patch));
    }

    {
        uint64_t instruction_start = 0xc8;
        uint32_t displacement = (HOOK_FN_SIZE + 8) - (instruction_start + 6);
        final_printf("displacement 2: %d\n", displacement);
        unsigned char patch[] = {0xFF, 0x15, 0x00, 0x00, 0x00, 0x00};
        memcpy(&patch[2], &displacement, sizeof(displacement));
        memcpy(new_mem + instruction_start, patch, sizeof(patch));
    }

    *(void**)((char*)new_mem + HOOK_FN_SIZE) = emit_span_start;
    *(void**)((char*)new_mem + HOOK_FN_SIZE + 8) = emit_span_end;

    if (sceKernelMprotect(new_mem, total_size, VM_PROT_READ | VM_PROT_EXECUTE) != 0) {
        final_printf("failed to set memory protection for patched hook function\n");
        munmap(new_mem, total_size);
        return NULL;
    }

    // hex_dump(new_mem, total_size);

    return new_mem;
}

unsigned char template_code[] = {
    // mov dword ptr fs:-32, <immediate_value>
    0x64, 0xC7, 0x04, 0x25, 0xE0, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,

    // mov r11, qword ptr [rip + 0xF]
    0x4C, 0x8B, 0x1D, 0x0F, 0x00, 0x00, 0x00,

    // mov qword ptr fs:-24, r11
    0x64, 0x4C, 0x89, 0x1C, 0x25, 0xE8, 0xFF, 0xFF, 0xFF,

    // jmp [rip + 0x8]
    0xFF, 0x25, 0x08, 0x00, 0x00, 0x00,

    // <placeholder for address_value> (mov instruction)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

    // <placeholder for target_function address> (jump instruction)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
};

static JumpSlotRelocationList* saved_relocs;
static void* saved_hooks;

void register_hooks_impl(JumpSlotRelocationList* relocs, void* hooks) {
    size_t total_size = relocs->count * sizeof(template_code);
    sceKernelMprotect((void *)hooks, total_size, VM_PROT_READ | VM_PROT_EXECUTE | VM_PROT_WRITE);
    for (unsigned int label_idx = 0; label_idx < relocs->count; label_idx++) {
        JumpSlotRelocation* reloc = &relocs->items[label_idx];

        void* func_mem = (char*)hooks + label_idx * sizeof(template_code);

        void** function_ptr = (void**)(reloc->relocation_offset + 0x0000000000400000);
        sceKernelMprotect((void *)function_ptr, sizeof(uint64_t), VM_PROT_ALL);
        void* initial_function_ptr_value = *function_ptr;
        if ((uint64_t)initial_function_ptr_value >= 0xeffffffe00000000) {
            continue;
        }

        if (initial_function_ptr_value >= hooks && initial_function_ptr_value < (hooks + total_size)) {
            continue;
        }

        final_printf("label_id = %u, offset = %lx, addr = %lx, symbol = %s\n", label_idx, reloc->relocation_offset, (uint64_t)*function_ptr, reloc->symbol_info->data.parsed.name);

        *(void**)((char*)func_mem + 34) = initial_function_ptr_value;

        *function_ptr = (void*)func_mem;
    }

    sceKernelMprotect((void *)hooks, total_size, VM_PROT_READ | VM_PROT_EXECUTE);

    final_printf("trampolines installed\n");
}

bool register_hooks(JumpSlotRelocationList* relocs, uint16_t static_tls_base) {
    void* hook = build_hook_fn(static_tls_base);
    if (!hook) {
        return false;
    }

    *(int32_t*)((char*)template_code + 4 ) = -(int32_t)static_tls_base - 32;
    *(int32_t*)((char*)template_code + 24) = -(int32_t)static_tls_base - 24;

    size_t bytes_needed = sizeof(template_code) * relocs->count;
    size_t pages_needed = (bytes_needed + PAGE_SIZE - 1) / PAGE_SIZE;
    size_t total_size = pages_needed * PAGE_SIZE;

    void* mem = mmap(NULL, total_size, PROT_READ | PROT_WRITE | PROT_EXEC, 
                     MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    
    if (mem == MAP_FAILED) {
        final_printf("failed to map memory\n");
        return false;
    }

    final_printf("jump relocation trampolines block: 0x%lx\n", (uint64_t)mem);
    final_printf("setting up tramplines for %lu items\n", relocs->count);

    for (unsigned int label_idx = 0; label_idx < relocs->count; label_idx++) {
        void* func_mem = (char*)mem + label_idx * sizeof(template_code);
        memcpy(func_mem, template_code, sizeof(template_code));

        *(uint32_t*)((char*)func_mem + 8) = label_idx;
        *(void**)((char*)func_mem + 42) = hook;
    }

    register_hooks_impl(relocs, mem);

    saved_relocs = relocs;
    saved_hooks = mem;
    return true;
}

void reregister_hooks() {
    register_hooks_impl(saved_relocs, saved_hooks);
}
