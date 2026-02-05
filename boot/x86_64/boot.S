/* x86_64 UEFI Boot Loader
 * 调用C++内核的main函数
 */

#define EFI_SUCCESS 0

.section .text
.global _start
_start:
    /*
     * UEFI调用约定:
     * RCX = 第一个参数 (SystemTable指针)
     * RDX = ImageHandle
     * R8  = 保留
     * R9  = 保留
     */
    
    /* 保存UEFI参数到非易失寄存器 */
    mov %rcx, %r12    /* 保存SystemTable指针到R12 */
    mov %rdx, %r13    /* 保存ImageHandle到R13 */
    
    /* 设置栈指针 */
    lea stack_top(%rip), %rsp
    
    /* 清空方向标志 */
    cld
    
    /* 调用C++初始化 (全局构造函数) */
    call _init
    
    /* 调用内核main函数 */
    mov %r12, %rdi    /* 第一个参数: SystemTable指针 */
    mov %r13, %rsi    /* 第二个参数: ImageHandle */
    call main
    
    /* main返回后，回到UEFI */
    mov %rax, %rcx    /* 返回值作为EFI_STATUS */
    
    /* 调用UEFI的ExitBootServices */
    mov (%r12), %rax           /* SystemTable->Hdr */
    mov 64(%rax), %rax         /* SystemTable->BootServices */
    mov 216(%rax), %rax        /* BootServices->ExitBootServices */
    
    mov %r13, %rcx             /* ImageHandle */
    mov $0, %rdx               /* MapKey = 0 */
    call *%rax
    
    /* 如果失败，无限循环 */
    jmp halt

halt:
    hlt
    jmp halt

.section .bss
.align 16
stack_bottom:
    .skip 65536                /* 64KB栈空间 */
stack_top:

.section .rodata
gdt64:
    .quad 0x0000000000000000   /* NULL段 */
    .quad 0x00af9a000000ffff   /* 代码段 */
    .quad 0x00cf92000000ffff   /* 数据段 */

gdt64_pointer:
    .word gdt64_pointer - gdt64 - 1
    .quad gdt64

.global gImageHandle
gImageHandle:
    .quad 0

.global gSystemTable
gSystemTable:
    .quad 0