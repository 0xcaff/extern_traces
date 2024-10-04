use crate::mapped_memory::{MappedMemory, Protection};
use crate::println;
use anyhow::{bail, format_err};
use core::arch::asm;
use core::ptr;

extern "C" {
    fn emit_span_start();
    fn emit_span_end();
}

const HOOK_FN_SIZE: usize = 0xcd;

#[naked]
pub extern "C" fn hook_template() {
    unsafe {
        asm!(
            // backup argument registers
            "push r9",
            "push r8",
            "push rcx",
            "push rdx",
            "push rsi",
            "push rdi",

            "movdqu [rsp - 0x80], xmm0",
            "movdqu [rsp - 0x70], xmm1",
            "movdqu [rsp - 0x60], xmm2",
            "movdqu [rsp - 0x50], xmm3",
            "movdqu [rsp - 0x40], xmm4",
            "movdqu [rsp - 0x30], xmm5",
            "movdqu [rsp - 0x20], xmm6",
            "movdqu [rsp - 0x10], xmm7",
            "sub rsp, 0x88",

            // call logger
            "mov edi, fs:[-32]",
            "mov rsi, fs:[-8]",
            "lea rdx, [rsp + 0x8]",
            "call {emit_span_start}",
            "nop",

            // restore argument registers
            "add rsp, 0x88",
            "movdqu xmm0, [rsp - 0x80]",
            "movdqu xmm1, [rsp - 0x70]",
            "movdqu xmm2, [rsp - 0x60]",
            "movdqu xmm3, [rsp - 0x50]",
            "movdqu xmm4, [rsp - 0x40]",
            "movdqu xmm5, [rsp - 0x30]",
            "movdqu xmm6, [rsp - 0x20]",
            "movdqu xmm7, [rsp - 0x10]",

            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "pop r8",
            "pop r9",

            // backup return address
            // use r10 as a scratch register. it is a caller saved register and not
            // an argument register.
            "pop r10",

            // store the value in thread local storage slot. there are no registers
            // which we can use in this context which will both
            // 1. not need to be backed up
            // 2. saved across call boundary
            // callee saved registers are saved across the call boundary but we
            // need to backup for our caller.
            // caller saved registers will not be saved across the call boundary
            // but do not need to be backed up prior to usage
            "mov fs:[-16], r10",

            // execute original function
            "mov rax, fs:[-24]",
            "call rax",

            // restore return address
            "mov r10, fs:[-16]",
            "push r10",

            // backup rax (also happens to align stack for call)
            "push rax",

            // call end logger
            "mov rdi, fs:[-8]",
            "call {emit_span_end}",

            // extra nop instruction to patch call
            "nop",

            // restore rax (ignore the return value of end logger)
            "pop rax",

            "ret",
            emit_span_start = sym emit_span_start,
            emit_span_end = sym emit_span_end,
            options(noreturn)
        );
    }
}

unsafe fn build_hook_fn(static_tls_base: u16) -> Result<MappedMemory, anyhow::Error> {
    let required_size = HOOK_FN_SIZE + 16;

    let mut memory =
        MappedMemory::allocate(required_size).map_err(|_| format_err!("allocation failed"))?;
    println!("hook fn block: {:#x}", memory.base as u64);

    ptr::copy_nonoverlapping(
        hook_template as *const u8,
        memory.base as *mut u8,
        HOOK_FN_SIZE,
    );

    for (idx, (offset, expected_value)) in [
        (0x3f + 4, -32),
        (0x47 + 5, -8),
        (0x9c + 5, -16),
        (0xa5 + 5, -24),
        (0xb0 + 5, -16),
        (0xbc + 5, -8),
    ]
    .iter()
    .enumerate()
    {
        let target_ptr = (memory.base as *mut u8).add(*offset) as *mut i32;
        let existing_value = *target_ptr;
        if existing_value != *expected_value {
            bail!(
                "failed to patch @ idx = {}, unexpected value {}, expected {:#x}",
                idx,
                existing_value,
                *expected_value
            );
        }

        *target_ptr = existing_value - static_tls_base as i32;
    }

    // patch emit_span_start and emit_span_end jumps
    {
        let patch: [u8; 6] = [0xFF, 0x15, 0x72, 0x00, 0x00, 0x00];
        ptr::copy_nonoverlapping(
            patch.as_ptr(),
            (memory.base as *mut u8).add(0x55),
            patch.len(),
        );
    }

    {
        let patch: [u8; 6] = [0xFF, 0x15, 0x0A, 0x00, 0x00, 0x00];
        ptr::copy_nonoverlapping(
            patch.as_ptr(),
            (memory.base as *mut u8).add(0xC5),
            patch.len(),
        );
    }

    *((memory.base as *mut u8).add(HOOK_FN_SIZE) as *mut *const libc::c_void) =
        emit_span_start as *const libc::c_void;

    *((memory.base as *mut u8).add(HOOK_FN_SIZE + 8) as *mut *const libc::c_void) =
        emit_span_end as *const libc::c_void;

    // set memory protection
    memory
        .protect(Protection::Read | Protection::Exec)
        .map_err(|_| format_err!("failed to protect"))?;

    Ok(memory)
}

pub unsafe fn register_hooks(
    relocs: &[u64],
    static_tls_base: u16,
) -> Result<MappedMemory, anyhow::Error> {
    let hook_fn = build_hook_fn(static_tls_base)?;

    let template_code = [
        // mov dword ptr fs:-32, <immediate_value>
        0x64, 0xC7, 0x04, 0x25, 0xE0, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        // mov r11, qword ptr [rip + 0xF]
        0x4C, 0x8B, 0x1D, 0x0F, 0x00, 0x00, 0x00, // mov qword ptr fs:-24, r11
        0x64, 0x4C, 0x89, 0x1C, 0x25, 0xE8, 0xFF, 0xFF, 0xFF, // jmp [rip + 0x8]
        0xFF, 0x25, 0x08, 0x00, 0x00, 0x00,
        // <placeholder for address_value> (mov instruction)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // <placeholder for target_function address> (jump instruction)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let bytes_needed = template_code.len() * relocs.len();

    let mut hooks_slab =
        MappedMemory::allocate(bytes_needed).map_err(|_| format_err!("failed to map"))?;

    println!(
        "jump relocation trampolines block: {:#x}",
        hooks_slab.base as u64
    );

    println!("setting up trampolines for {} items", relocs.len());

    for label_idx in 0..relocs.len() {
        let reloc = &relocs[label_idx];
        let func_mem = (hooks_slab.base as *mut u8).add(label_idx * template_code.len());
        ptr::copy_nonoverlapping(template_code.as_ptr(), func_mem, template_code.len());

        let function_ptr = (*reloc + 0x0000000000400000) as *mut *mut libc::c_void;

        libc::mprotect(
            function_ptr as *mut libc::c_void,
            size_of::<u64>(),
            libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
        );

        println!(
            "label_id = {}, offset = {:#x}, addr = {:#x}",
            label_idx, *reloc, *function_ptr as u64,
        );

        let immediate_value_ptr = func_mem.add(8) as *mut u32;
        *immediate_value_ptr = label_idx as u32;

        let target_func_ptr = func_mem.add(34) as *mut *mut libc::c_void;
        *target_func_ptr = *function_ptr;

        let hook_func_ptr = func_mem.add(42) as *mut *mut libc::c_void;
        *hook_func_ptr = hook_fn.base;

        *function_ptr = func_mem as *mut libc::c_void;
    }

    println!("trampolines installed");

    hooks_slab.protect(Protection::Read | Protection::Exec)?;

    Ok(hooks_slab)
}
