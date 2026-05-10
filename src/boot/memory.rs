use uefi::prelude::*;
use uefi::mem::memory_map::{MemoryMap, MemoryType};

/*
 * 内存信息获取
 * 目的：获取物理内存信息，并传入系统内核
 * 识别类型：总内存、可用内存、保留内存等
 * 传递方式：通过内核引导参数结构体传递给内核
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MemoryRegionType {
    Usable = 0,
    RECLAIMABLE = 1,
    Reserved = 2,
    Runtime = 3,
    Acpi = 4,
    MMIO = 5
}

pub struct MemoryRegion {
    pub start: usize,
    pub size: usize,
    pub region_type: MemoryRegionType,
    pub type_str: &'static str
}

// 单个内存区域条目
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryRegionEntry {
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub page_count: u64,
    pub region_type: MemoryRegionType,
    pub type_str: &'static str,
}

pub fn memory_init() -> [MemoryRegion; 6] {
    let memory_map = boot::memory_map(MemoryType::LOADER_DATA).unwrap();

    let mut memory_rigions : [MemoryRegion;6]= [
        MemoryRegion { start: 0, size: 0, region_type: MemoryRegionType::Usable, type_str: "Usable" },
        MemoryRegion { start: 0, size: 0, region_type: MemoryRegionType::Reserved, type_str: "Reserved" },
        MemoryRegion { start: 0, size: 0, region_type: MemoryRegionType::RECLAIMABLE, type_str: "RECLAIMABLE" },
        MemoryRegion { start: 0, size: 0, region_type: MemoryRegionType::Runtime, type_str: "Runtime" },
        MemoryRegion { start: 0, size: 0, region_type: MemoryRegionType::Acpi, type_str: "ACPI" },
        MemoryRegion { start: 0, size: 0, region_type: MemoryRegionType::MMIO, type_str: "MMIO" },
    ]; 

    // 计算内存区域数量和总大小

    let mut index : usize = 0;
    for ( _, _descriptor) in memory_map.entries().enumerate() {index+=1}

    let sizes = index * core::mem::size_of::<MemoryRegion>();
    let pages = (sizes + 4095) / 4096;

    let addr = boot::allocate_pages(
            boot::AllocateType::AnyPages,
            boot::MemoryType::LOADER_DATA,
            pages
        ).unwrap();
    

    // 将内存区域信息写入分配的内存中

    for ( _, descriptor) in memory_map.entries().enumerate() {
        // uefi::println!("Entry {}: {:?}", i, descriptor);
        // uefi::println!("  物理地址: {:#x}", descriptor.phys_start);
        // uefi::println!("  页数: {}", descriptor.page_count);
        // uefi::println!("  大小: {} KB", (descriptor.page_count * uefi::boot::PAGE_SIZE as u64) / 1024);
        // uefi::println!("  类型: {:?}", descriptor.ty);
        // uefi::println!("  属性: {:?}", descriptor.att);
        if descriptor.ty == MemoryType::CONVENTIONAL { // 可用内存
            memory_rigions[0].size+= (descriptor.page_count * uefi::boot::PAGE_SIZE as u64) as usize;
        }
        else if descriptor.ty == MemoryType::RESERVED || descriptor.ty == MemoryType::LOADER_CODE || descriptor.ty == MemoryType::LOADER_DATA {
            // 保留内存
            memory_rigions[1].size+= (descriptor.page_count * uefi::boot::PAGE_SIZE as u64) as usize;
        }
        else if descriptor.ty == MemoryType::BOOT_SERVICES_CODE || descriptor.ty == MemoryType::BOOT_SERVICES_DATA {
            // 可回收内存
            memory_rigions[2].size+= (descriptor.page_count * uefi::boot::PAGE_SIZE as u64) as usize;
        }
        else if  descriptor.ty == MemoryType::RUNTIME_SERVICES_DATA ||  descriptor.ty == MemoryType::RUNTIME_SERVICES_CODE  {
            // 运行时内存
            memory_rigions[3].size+= (descriptor.page_count * uefi::boot::PAGE_SIZE as u64) as usize;
        }
        else if descriptor.ty == MemoryType::ACPI_RECLAIM || descriptor.ty == MemoryType::ACPI_NON_VOLATILE {
            // ACPI内存
            memory_rigions[4].size+= (descriptor.page_count * uefi::boot::PAGE_SIZE as u64) as usize;
        }
        else if descriptor.ty == MemoryType::MMIO {
            // MMIO内存
            memory_rigions[5].size+= (descriptor.page_count * uefi::boot::PAGE_SIZE as u64) as usize;
        }
        
    }
    return memory_rigions;
}
