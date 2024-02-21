#include <stdalign.h>

typedef struct RegisterArgsState
{
    // General purpose registers
    unsigned long rdi, rsi, rdx, rcx, r8, r9;
    // Base pointer and stack pointer
    unsigned long rbp, rsp;
    // SSE registers (XMM0-XMM7 for floating-point arguments)
    // Ensure 16-byte alignment for SSE registers
    alignas(16) __uint128_t xmm0, xmm1, xmm2, xmm3, xmm4, xmm5, xmm6, xmm7;
} RegisterArgsState;

#define SAVE_REG(reg) asm volatile("mov %%" #reg ", %0" : "=m"(state->reg));
#define SAVE_XMM_REG(reg) asm volatile("movdqu %%" #reg ", %0" : "=m"(state->reg));

void save_args_state(RegisterArgsState *state)
{
    SAVE_REG(rdi)
    SAVE_REG(rsi)
    SAVE_REG(rdx)
    SAVE_REG(rcx)
    SAVE_REG(r8)
    SAVE_REG(r9)
    SAVE_REG(rbp)
    SAVE_REG(rsp)

    SAVE_XMM_REG(xmm0)
    SAVE_XMM_REG(xmm1)
    SAVE_XMM_REG(xmm2)
    SAVE_XMM_REG(xmm3)
    SAVE_XMM_REG(xmm4)
    SAVE_XMM_REG(xmm5)
    SAVE_XMM_REG(xmm6)
    SAVE_XMM_REG(xmm7)
}

#define RESTORE_REG(reg) asm volatile("mov %0, %%" #reg : : "m"(state->reg));
#define RESTORE_XMM_REG(reg) asm volatile("movdqu %0, %%" #reg : : "m"(state->reg));

void restore_args_state(RegisterArgsState *state)
{
    RESTORE_REG(rdi)
    RESTORE_REG(rsi)
    RESTORE_REG(rdx)
    RESTORE_REG(rcx)
    RESTORE_REG(r8)
    RESTORE_REG(r9)
    RESTORE_REG(rbp)
    RESTORE_REG(rsp)

    RESTORE_XMM_REG(xmm0)
    RESTORE_XMM_REG(xmm1)
    RESTORE_XMM_REG(xmm2)
    RESTORE_XMM_REG(xmm3)
    RESTORE_XMM_REG(xmm4)
    RESTORE_XMM_REG(xmm5)
    RESTORE_XMM_REG(xmm6)
    RESTORE_XMM_REG(xmm7)
}
