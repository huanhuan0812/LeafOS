; ARM64 UEFI Boot Loader
; 调用C++内核的main函数

.section .text
.global _start
_start:
; ARM64 UEFI调用约定:
; X0 = ImageHandle
; X1 = SystemTable指针

; 保存UEFI参数
ldr x2, =gImageHandle
str x0, [x2]
ldr x2, =gSystemTable
str x1, [x2]

; 设置栈指针
ldr x2, =stack_top
mov sp, x2

; 清空栈帧指针
mov x29, 0
mov x30, 0

; 调用C++初始化
bl _init

; 调用内核main函数
mov x0, x1        ; 第一个参数: SystemTable指针
ldr x1, =gImageHandle
ldr x1, [x1]      ; 第二个参数: ImageHandle
bl main

; main返回后，回到UEFI
; aarch64 UEFI直接返回
ret

.section .bss
.align 16
stack_bottom:
.skip 65536                ; 64KB栈空间
stack_top:

.section .rodata
.align 3
gImageHandle:
.quad 0

gSystemTable:
.quad 0