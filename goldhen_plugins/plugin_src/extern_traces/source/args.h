#define SAVE_ARGS_STATE()               \
    asm volatile(                       \
        "push %rdi\n\t"                 \
        "push %rsi\n\t"                 \
        "push %rdx\n\t"                 \
        "push %rcx\n\t"                 \
        "push %r8\n\t"                  \
        "push %r9\n\t"                  \
        "movdqu %xmm0, -0x10(%rsp)\n\t" \
        "movdqu %xmm1, -0x20(%rsp)\n\t" \
        "movdqu %xmm2, -0x30(%rsp)\n\t" \
        "movdqu %xmm3, -0x40(%rsp)\n\t" \
        "movdqu %xmm4, -0x50(%rsp)\n\t" \
        "movdqu %xmm5, -0x60(%rsp)\n\t" \
        "movdqu %xmm6, -0x70(%rsp)\n\t" \
        "movdqu %xmm7, -0x80(%rsp)\n\t" \
        "sub $0x80, %rsp\n\t")

#define RESTORE_ARGS_STATE()            \
    asm volatile(                       \
        "add $0x80, %rsp\n\t"           \
        "movdqu -0x10(%rsp), %xmm0\n\t" \
        "movdqu -0x20(%rsp), %xmm1\n\t" \
        "movdqu -0x30(%rsp), %xmm2\n\t" \
        "movdqu -0x40(%rsp), %xmm3\n\t" \
        "movdqu -0x50(%rsp), %xmm4\n\t" \
        "movdqu -0x60(%rsp), %xmm5\n\t" \
        "movdqu -0x70(%rsp), %xmm6\n\t" \
        "movdqu -0x80(%rsp), %xmm7\n\t" \
        "pop %r9\n\t"                   \
        "pop %r8\n\t"                   \
        "pop %rcx\n\t"                  \
        "pop %rdx\n\t"                  \
        "pop %rsi\n\t"                  \
        "pop %rdi\n\t")
