#include "hook.h"
#include "plugin_common.h"
#include "tracing.h"

#include <stdlib.h>
#include <orbis/libkernel.h>

uint64_t read_call_label()
{
    uint64_t old_value;

    __asm__ volatile(
        "movq %%fs:-32, %0;"
        : "=r"(old_value)
        :
        : "memory");

    return old_value;
}

void start_logger() {
    emit_span_start(read_call_label());
}

void end_logger() {
    emit_span_end();
}

__attribute__((naked)) void hook()
{
    asm volatile(
        // backup argument registers
        "push %rdi\n\t"
        "push %rsi\n\t"
        "push %rdx\n\t"
        "push %rcx\n\t"
        "push %r8\n\t"
        "push %r9\n\t"
        "movdqu %xmm0, -0x10(%rsp)\n\t"
        "movdqu %xmm1, -0x20(%rsp)\n\t"
        "movdqu %xmm2, -0x30(%rsp)\n\t"
        "movdqu %xmm3, -0x40(%rsp)\n\t"
        "movdqu %xmm4, -0x50(%rsp)\n\t"
        "movdqu %xmm5, -0x60(%rsp)\n\t"
        "movdqu %xmm6, -0x70(%rsp)\n\t"
        "movdqu %xmm7, -0x80(%rsp)\n\t"
        "sub $0x88, %rsp\n\t"

        // call logger
        "call start_logger\n\t"

        // restore argument registers
        "add $0x88, %rsp\n\t"
        "movdqu -0x10(%rsp), %xmm0\n\t"
        "movdqu -0x20(%rsp), %xmm1\n\t"
        "movdqu -0x30(%rsp), %xmm2\n\t"
        "movdqu -0x40(%rsp), %xmm3\n\t"
        "movdqu -0x50(%rsp), %xmm4\n\t"
        "movdqu -0x60(%rsp), %xmm5\n\t"
        "movdqu -0x70(%rsp), %xmm6\n\t"
        "movdqu -0x80(%rsp), %xmm7\n\t"
        "pop %r9\n\t"
        "pop %r8\n\t"
        "pop %rcx\n\t"
        "pop %rdx\n\t"
        "pop %rsi\n\t"
        "pop %rdi\n\t"

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
        "call end_logger\n\t"

        // restore rax (ignore the return value of end logger)
        "pop %rax\n\t"

        "ret\n\t"
    );
}

#define PAGE_SIZE 4096

void register_hooks(JumpSlotRelocationList* relocs) {
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

    size_t bytes_needed = sizeof(template_code) * relocs->count;
    size_t pages_needed = (bytes_needed + PAGE_SIZE - 1) / PAGE_SIZE;
    size_t total_size = pages_needed * PAGE_SIZE;

    void* mem = mmap(NULL, total_size, PROT_READ | PROT_WRITE | PROT_EXEC, 
                     MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    
    if (mem == MAP_FAILED) {
        final_printf("failed to map memory\n");
        return;
    }

    final_printf("jump relocation trampolines block: 0x%lx\n", (uint64_t)mem);
    final_printf("setting up tramplines for %lu items\n", relocs->count);

    for (unsigned int label_idx = 0; label_idx < relocs->count; label_idx++) {
        JumpSlotRelocation* reloc = &relocs->items[label_idx];

        void* func_mem = (char*)mem + label_idx * sizeof(template_code);
        memcpy(func_mem, template_code, sizeof(template_code));

        void** function_ptr = (void**)(reloc->relocation_offset + 0x0000000000400000);
        sceKernelMprotect((void *)function_ptr, sizeof(uint64_t), VM_PROT_ALL);
        final_printf("offset = %lx, addr = %lx, symbol = %s\n", reloc->relocation_offset, (uint64_t)*function_ptr, reloc->symbol_info->data.parsed.name);

        *(uint32_t*)((char*)func_mem + 8) = label_idx;
        *(void**)((char*)func_mem + 34) = *function_ptr;
        *(void**)((char*)func_mem + 42) = (void*)hook;

        *function_ptr = (void*)func_mem;
    }
    
    final_printf("trampolines installed\n");

    sceKernelMprotect((void *)mem, total_size, VM_PROT_READ | VM_PROT_EXECUTE);
}