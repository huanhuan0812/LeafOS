/* x86_64 内核链接脚本 */
ENTRY(_start)

SECTIONS
{
    /* 内核加载地址 */
    . = 0x100000;
    
    .text BLOCK(4K) : ALIGN(4K)
    {
        *(.text)
    }
    
    .rodata BLOCK(4K) : ALIGN(4K)
    {
        *(.rodata)
    }
    
    .data BLOCK(4K) : ALIGN(4K)
    {
        *(.data)
    }
    
    .bss BLOCK(4K) : ALIGN(4K)
    {
        *(COMMON)
        *(.bss)
    }
    
    /* 调试信息 */
    .stab : { *(.stab) }
    .stabstr : { *(.stabstr) }
    
    /DISCARD/ : { *(.comment) *(.eh_frame) }
}