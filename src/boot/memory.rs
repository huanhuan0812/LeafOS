use uefi::boot::PAGE_SIZE;
use uefi::prelude::*;
use uefi::mem::memory_map::{MemoryMap, MemoryMapKey, MemoryType};

/*
 * 内存信息获取
 * 目的：获取物理内存信息，并传入系统内核
 * 识别类型：总内存、可用内存、保留内存等
 * 传递方式：通过 BootParams 结构体传递给内核
 */

 // 内存信息结构体
#[repr(C)]
pub struct MemoryParams {
    pub magic: u64,                      // 魔数验证
    pub memory_map_phys: u64,            // 内存映射地址
    pub memory_map_entries: u64,         // 条目数量
    pub memory_map_size: u64,            // 总大小
    pub usable_ram_size: u64,            // 可用 RAM 总大小
}

 // 内存区域类型
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

#[warn(unused)]
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

impl Default for MemoryRegionEntry {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            size: 0,
            page_count: 0,
            region_type: MemoryRegionType::Reserved,
            type_str: "Unknown",
        }
    }
}

// 动态内存列表（使用 UEFI 分配的内存）
pub struct DynamicMemoryList {
    entries: *mut MemoryRegionEntry,
    capacity: usize,
    count: usize,
}

impl DynamicMemoryList {
    // 从 UEFI 分配内存创建新列表
    pub fn new(capacity: usize) -> Result<Self, &'static str> {
        let entry_size = core::mem::size_of::<MemoryRegionEntry>();
        let total_size = capacity * entry_size;
        let pages = (total_size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;

        // 从 UEFI 分配内存
        let addr = match boot::allocate_pages(
            boot::AllocateType::AnyPages,
            boot::MemoryType::LOADER_DATA,
            pages,
        ) {
            Ok(addr) => addr,
            Err(_) => return Err("Failed to allocate memory for memory list"),
        };

        // addr is likely NonNull<u8> or similar, so use as_ptr()
        unsafe {
            core::ptr::write_bytes(addr.as_ptr(), 0, total_size);
        }

        Ok(Self {
            entries: addr.as_ptr() as *mut MemoryRegionEntry,
            capacity,
            count: 0,
        })
    }

    // 添加内存区域条目
    pub fn push(&mut self, entry: MemoryRegionEntry) -> Result<(), &'static str> {
        if self.count >= self.capacity {
            return Err("DynamicMemoryList capacity exceeded");
        }
        
        unsafe {
            self.entries.add(self.count).write(entry);
        }
        self.count += 1;
        Ok(())
    }

    pub fn insert(&mut self, index: usize, entry: MemoryRegionEntry) -> Result<(), &'static str> {
        if self.count >= self.capacity {
            return Err("DynamicMemoryList capacity exceeded");
        }
        if index > self.count {
            return Err("DynamicMemoryList insert index out of bounds");
        }

        unsafe {
            for i in (index..self.count).rev() {
                let src = self.entries.add(i);
                let dst = self.entries.add(i + 1);
                dst.write(*src);
            }
            self.entries.add(index).write(entry);
        }

        self.count += 1;
        Ok(())
    }

    // 获取条目（只读）
    pub fn get(&self, index: usize) -> Option<&MemoryRegionEntry> {
        if index >= self.count {
            None
        } else {
            unsafe { Some(&*self.entries.add(index)) }
        }
    }

    // 获取可变条目
    pub fn get_mut(&mut self, index: usize) -> Option<&mut MemoryRegionEntry> {
        if index >= self.count {
            None
        } else {
            unsafe { Some(&mut *self.entries.add(index)) }
        }
    }

    // 迭代器
    pub fn iter(&self) -> DynamicIterator {
        DynamicIterator {
            entries: self.entries,
            count: self.count,
            index: 0,
        }
    }
    // 获取当前数量
    pub fn len(&self) -> usize {
        self.count
    }
    
    // 是否为空
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
    // 获取容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // 获取内存映射地址（物理地址）
    pub fn get_entries_addr(&self) -> u64 {
        self.entries as u64
    }

    // 可选：获取原始指针（用于高级操作）
    pub fn as_ptr(&self) -> *mut MemoryRegionEntry {
        self.entries
    }

    // 清理（如果需要提前释放）
    pub fn free(&mut self) {
        if !self.entries.is_null() {
            let total_size = self.capacity * core::mem::size_of::<MemoryRegionEntry>();
            let pages = (total_size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
            // Convert entries pointer to NonNull<u8>
            let addr = unsafe { core::ptr::NonNull::new_unchecked(self.entries as *mut u8) };
            unsafe {
                let _ = boot::free_pages(addr, pages);
            }
            self.entries = core::ptr::null_mut();
            self.count = 0;
            self.capacity = 0;
        }
    }

    // 合并相邻且类型相同的内存区域
    pub fn merge_adjacent(&mut self) {
        if self.count < 2 {
            return;
        }
        
        let mut write_idx = 0;
        for read_idx in 0..self.count {
            if write_idx == 0 {
                write_idx += 1;
                continue;
            }
            
            unsafe {
                let prev = &*self.entries.add(write_idx - 1);
                let curr = &*self.entries.add(read_idx);
                
                if prev.region_type == curr.region_type && prev.end + 1 == curr.start {
                    // 合并
                    let merged = MemoryRegionEntry {
                        end: curr.end,
                        size: curr.end - prev.start + 1,
                        page_count: (curr.end - prev.start + 1) / PAGE_SIZE as u64,
                        ..*prev
                    };
                    self.entries.add(write_idx - 1).write(merged);
                } else {
                    if write_idx != read_idx {
                        self.entries.add(write_idx).write(curr.clone());
                    }
                    write_idx += 1;
                }
            }
        }
        self.count = write_idx;
    }

}

// 迭代器实现
pub struct DynamicIterator {
    entries: *mut MemoryRegionEntry,
    count: usize,
    index: usize,
}

impl Iterator for DynamicIterator {
    type Item = MemoryRegionEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let entry = unsafe { *self.entries.add(self.index) };
            self.index += 1;
            Some(entry)
        }
    }
}

// 内存信息主结构
pub struct MemoryInfo {
    pub all_regions: DynamicMemoryList,
    pub usable_regions: DynamicMemoryList,
    pub reclaimable_regions: DynamicMemoryList,
    pub total_usable: u64,
    pub total_reclaimable: u64,
    pub total_reserved: u64,
    pub total_runtime: u64,
    pub total_acpi: u64,
    pub total_mmio: u64,
    pub map_key: MemoryMapKey,
    pub descriptor_size: usize,
    pub handover_map_phys: u64,
}

// 完善 MemoryInfo 实现
impl MemoryInfo {
    pub fn new() -> Result<Self, &'static str> {
        // 获取 UEFI 内存映射
        let memory_map = boot::memory_map(MemoryType::LOADER_DATA)
            .map_err(|_| "Failed to get memory map")?;
        
        let entry_count = memory_map.entries().len();
        
        // 创建动态列表，容量足够存储所有区域
        let mut all_regions = DynamicMemoryList::new(entry_count + 2)?;
        
        let mut total_usable = 0u64;
        let mut total_reclaimable = 0u64;
        let mut total_reserved = 0u64;
        let mut total_runtime = 0u64;
        let mut total_acpi = 0u64;
        let mut total_mmio = 0u64;
        
        let map_key = memory_map.key();
        //let descriptor_size = memory_map.descriptor_size();
        let descriptor_size = memory_map.entries().size_hint().0 * core::mem::size_of::<MemoryRegionEntry>();
        
        // 遍历内存映射，填充详细信息
        for descriptor in memory_map.entries() {
            let start = descriptor.phys_start;
            let page_count = descriptor.page_count;
            let size = page_count * PAGE_SIZE as u64;
            let end = start + size - 1;
            
            // 确定内存类型
            let (region_type, type_str) = match descriptor.ty {
                MemoryType::CONVENTIONAL => {
                    total_usable += size;
                    (MemoryRegionType::Usable, "Usable")
                },
                MemoryType::BOOT_SERVICES_CODE | MemoryType::BOOT_SERVICES_DATA => {
                    total_reclaimable += size;
                    (MemoryRegionType::RECLAIMABLE, "Reclaimable")
                },
                MemoryType::RESERVED | MemoryType::LOADER_CODE | MemoryType::LOADER_DATA => {
                    total_reserved += size;
                    (MemoryRegionType::Reserved, "Reserved")
                },
                MemoryType::RUNTIME_SERVICES_CODE | MemoryType::RUNTIME_SERVICES_DATA => {
                    total_runtime += size;
                    (MemoryRegionType::Runtime, "Runtime")
                },
                MemoryType::ACPI_RECLAIM | MemoryType::ACPI_NON_VOLATILE => {
                    total_acpi += size;
                    (MemoryRegionType::Acpi, "ACPI")
                },
                MemoryType::MMIO => {
                    total_mmio += size;
                    (MemoryRegionType::MMIO, "MMIO")
                },
                _ => {
                    // 其他类型当作保留内存
                    total_reserved += size;
                    (MemoryRegionType::Reserved, "Unknown")
                }
            };
            
            let entry = MemoryRegionEntry {
                start,
                end,
                size,
                page_count,
                region_type,
                type_str,
            };
            
            // 添加到所有区域列表
            all_regions.push(entry)?;
        }

        let entry_size = core::mem::size_of::<MemoryRegionEntry>() as u64;
        let handover_size = ((all_regions.len() + 1) as u64) * entry_size;
        let handover_map_phys = reserve_handover_region(&mut all_regions, handover_size)?;

        copy_regions_to_phys(&all_regions, handover_map_phys)?;

        let mut usable_regions = DynamicMemoryList::new(all_regions.len())?;
        let mut reclaimable_regions = DynamicMemoryList::new(all_regions.len())?;

        total_usable = 0;
        total_reclaimable = 0;
        total_reserved = 0;
        total_runtime = 0;
        total_acpi = 0;
        total_mmio = 0;

        for entry in all_regions.iter() {
            match entry.region_type {
                MemoryRegionType::Usable => {
                    total_usable += entry.size;
                    usable_regions.push(entry)?;
                }
                MemoryRegionType::RECLAIMABLE => {
                    total_reclaimable += entry.size;
                    reclaimable_regions.push(entry)?;
                }
                MemoryRegionType::Reserved => {
                    total_reserved += entry.size;
                }
                MemoryRegionType::Runtime => {
                    total_runtime += entry.size;
                }
                MemoryRegionType::Acpi => {
                    total_acpi += entry.size;
                }
                MemoryRegionType::MMIO => {
                    total_mmio += entry.size;
                }
            }
        }
        
        Ok(Self {
            all_regions,
            usable_regions,
            reclaimable_regions,
            total_usable,
            total_reclaimable,
            total_reserved,
            total_runtime,
            total_acpi,
            total_mmio,
            map_key,
            descriptor_size,
            handover_map_phys,
        })
    }
    
    // 打印内存信息（调试用）
    pub fn print_info(&self) {
        let total_ram = self.total_usable + self.total_reclaimable;
        let total_ram_mb = total_ram / (1024 * 1024);
        let total_mmio_space = self.total_reserved;  // 这些实际上是 MMIO 地址空间
    
        uefi::println!("=== Memory Information ===");
        uefi::println!("Total Physical RAM: {} MB", total_ram_mb);
        uefi::println!("  - Usable: {} MB", self.total_usable / (1024 * 1024));
        uefi::println!("  - Boot Service (reclaimable): {} MB", self.total_reclaimable / (1024 * 1024));
        uefi::println!("MMIO Address Space: {} MB", total_mmio_space / (1024 * 1024));
        uefi::println!("Runtime Services: {} MB", self.total_runtime / (1024 * 1024));
        uefi::println!("ACPI Tables: {} MB", self.total_acpi / (1024 * 1024));
        uefi::println!("==========================");
    }

    pub fn get_boot_params(&self) -> MemoryParams {
        MemoryParams {
            magic: 0x4C656146,  // "Leaf"
            memory_map_phys: self.handover_map_phys,
            memory_map_entries: self.all_regions.len() as u64,
            memory_map_size: (self.all_regions.len() * core::mem::size_of::<MemoryRegionEntry>()) as u64,
            usable_ram_size: self.total_usable + self.total_reclaimable,
        }
    }
}

fn align_up(value: u64, align: u64) -> u64 {
    if value % align == 0 {
        value
    } else {
        value + (align - (value % align))
    }
}

fn reserve_handover_region(
    all_regions: &mut DynamicMemoryList,
    size: u64,
) -> Result<u64, &'static str> {
    let aligned_size = align_up(size, PAGE_SIZE as u64);

    for index in 0..all_regions.len() {
        let entry = match all_regions.get(index) {
            Some(entry) => *entry,
            None => continue,
        };

        if entry.region_type != MemoryRegionType::Usable {
            continue;
        }

        if entry.size < aligned_size {
            continue;
        }

        let reserved_start = entry.start;
        let reserved_end = reserved_start + aligned_size - 1;
        let remaining_size = entry.size - aligned_size;

        let reserved_entry = MemoryRegionEntry {
            start: reserved_start,
            end: reserved_end,
            size: aligned_size,
            page_count: aligned_size / PAGE_SIZE as u64,
            region_type: MemoryRegionType::Reserved,
            type_str: "Reserved(Handover)",
        };

        if remaining_size == 0 {
            if let Some(entry_mut) = all_regions.get_mut(index) {
                *entry_mut = reserved_entry;
            }
        } else {
            let updated_entry = MemoryRegionEntry {
                start: reserved_end + 1,
                end: entry.end,
                size: remaining_size,
                page_count: remaining_size / PAGE_SIZE as u64,
                ..entry
            };

            if let Some(entry_mut) = all_regions.get_mut(index) {
                *entry_mut = updated_entry;
            }

            all_regions.insert(index, reserved_entry)?;
        }

        return Ok(reserved_start);
    }

    Err("Failed to reserve handover memory region")
}

fn copy_regions_to_phys(
    all_regions: &DynamicMemoryList,
    dest_phys: u64,
) -> Result<(), &'static str> {
    let dest = dest_phys as *mut MemoryRegionEntry;
    for index in 0..all_regions.len() {
        let entry = match all_regions.get(index) {
            Some(entry) => *entry,
            None => return Err("Failed to read memory region entry"),
        };
        unsafe {
            dest.add(index).write(entry);
        }
    }

    Ok(())
}

// 修改原有的 memory_init 函数，改为使用新的 MemoryInfo
pub fn memory_init() -> Result<MemoryInfo, &'static str> {
    MemoryInfo::new()
}