use uefi::prelude::*;
use uefi::mem::memory_map::MemoryType;

/*
 * 内存信息获取
 * 目的：获取物理内存信息，并传入系统内核
 * 识别类型：总内存、可用内存、保留内存等
 * 传递方式：通过内核引导参数结构体传递给内核
 */

pub struct MemoryRegion {
    pub start: usize,
    pub size: usize,
    pub region_type: MemoryType,
}



pub fn memory_init(){
    let memory_map = 
        boot::memory_map(MemoryType::LOADER_DATA);

    let available_memory_map =
        boot::memory_map(MemoryType::CONVENTIONAL);

    let reserved_memory_map = 
        boot::memory_map(MemoryType::RESERVED);
    
}
