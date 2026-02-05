/* aarch64 UEFI内核链接脚本 */
ENTRY(_start)

SECTIONS
{
    /* ARM64 UEFI加载地址 */
    . = 0x40080000;
    
    .text ALIGN(4K) : 
    {
        *(.text)
        *(.text.*)
        *(.gnu.linkonce.t*)
    }
    
    .rodata ALIGN(4K) : 
    {
        *(.rodata)
        *(.rodata.*)
        *(.gnu.linkonce.r*)
    }
    
    /* 全局构造函数数组 */
    .init_array ALIGN(8) : 
    {
        __init_array_start = .;
        KEEP(*(SORT_BY_INIT_PRIORITY(.init_array.*)))
        KEEP(*(.init_array))
        __init_array_end = .;
    }
    
    /* 全局析构函数数组 */
    .fini_array ALIGN(8) : 
    {
        __fini_array_start = .;
        KEEP(*(SORT_BY_INIT_PRIORITY(.fini_array.*)))
        KEEP(*(.fini_array))
        __fini_array_end = .;
    }
    
    .data ALIGN(4K) : 
    {
        *(.data)
        *(.data.*)
        *(.gnu.linkonce.d*)
    }
    
    .bss ALIGN(4K) : 
    {
        *(COMMON)
        *(.bss)
        *(.bss.*)
        *(.gnu.linkonce.b*)
    }
    
    . = ALIGN(4096);
    kernel_end = .;
}