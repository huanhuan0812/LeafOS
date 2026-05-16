use uefi::mem::memory_map::MemoryType;
use core::ptr::copy_nonoverlapping;
use core::mem::size_of;
use uefi::system::with_config_table;
use uefi::table::cfg::ConfigTableEntry;
use acpi::rsdp::Rsdp;

// ACPI 表头部结构
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct AcpiHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: [u8; 4],
    pub creator_revision: u32,
}

// 修改：存储物理地址而不是虚拟地址
#[repr(C)]
pub struct AcpiCopiedData {
    pub rsdp_phys: u64,                    // RSDP 物理地址
    pub xsdt_phys: u64,                    // XSDT 物理地址
    pub table_count: u32,                  // 表的数量
    pub table_phys_addrs: [u64; 32],       // 各表的物理地址
    pub table_sizes: [u32; 32],            // 各表的大小
    pub rsdp_size: u32,                    // RSDP 大小
    pub xsdt_size: u32,                    // XSDT 大小
}

impl Default for AcpiCopiedData {
    fn default() -> Self {
        Self {
            rsdp_phys: 0,
            xsdt_phys: 0,
            table_count: 0,
            table_phys_addrs: [0; 32],
            table_sizes: [0; 32],
            rsdp_size: 0,
            xsdt_size: 0,
        }
    }
}

// 查找 RSDP 物理地址
fn find_rsdp() -> Option<u64> {
    with_config_table(|slice| {
        for i in slice {
            if i.guid == ConfigTableEntry::ACPI2_GUID {
                return Some(i.address as u64);
            }
        }
        None
    })
}

// 计算校验和
fn checksum(data: *const u8, len: usize) -> u8 {
    let mut sum: u8 = 0;
    unsafe {
        for i in 0..len {
            sum = sum.wrapping_add(*data.add(i));
        }
    }
    sum
}

/// 方案1：直接使用物理地址（推荐）
/// 优点：不需要拷贝，退出 boot services 后仍然有效
pub fn get_acpi_physical_addresses() -> Result<AcpiCopiedData, &'static str> {
    // 1. 找到 RSDP
    let rsdp_phys = find_rsdp().ok_or("RSDP not found")?;
    uefi::println!("RSDP found at physical address: 0x{:x}", rsdp_phys);
    
    // 2. 验证 RSDP
    let rsdp_ptr = rsdp_phys as *const Rsdp;
    if unsafe { &(*rsdp_ptr).signature } != b"RSD PTR " {
        return Err("Invalid RSDP signature");
    }
    
    // 验证校验和
    if checksum(rsdp_phys as *const u8, 20) != 0 {
        return Err("RSDP checksum failed");
    }
    
    // 3. 确定使用 XSDT（需要 ACPI 2.0+）
    let revision = unsafe { (*rsdp_ptr).revision };
    if revision < 2 {
        return Err("Only ACPI 2.0+ supported");
    }
    
    let xsdt_phys = unsafe { (*rsdp_ptr).xsdt_address };
    uefi::println!("XSDT found at physical address: 0x{:x}", xsdt_phys);
    
    // 4. 读取 XSDT 头部获取长度和表数量
    let xsdt_header = unsafe { &*(xsdt_phys as *const AcpiHeader) };
    let xsdt_size = xsdt_header.length as usize;
    let entry_count = (xsdt_size - size_of::<AcpiHeader>()) / 8;
    uefi::println!("XSDT size: {} bytes, {} tables", xsdt_size, entry_count);
    
    if entry_count > 32 {
        uefi::println!("Warning: Too many tables ({}), limiting to 32", entry_count);
    }
    
    let mut result = AcpiCopiedData {
        rsdp_phys,
        xsdt_phys,
        table_count: entry_count.min(32) as u32,
        rsdp_size: size_of::<Rsdp>() as u32,
        xsdt_size: xsdt_size as u32,
        ..Default::default()
    };
    
    // 5. 直接从物理地址读取各表的地址和大小
    for i in 0..entry_count.min(32) {
        // 从 XSDT 中读取表的物理地址
        let table_phys = unsafe {
            let entry_ptr = (xsdt_phys + size_of::<AcpiHeader>() as u64) as *const u64;
            *entry_ptr.add(i)
        };
        
        // 读取表头部获取大小
        let header = unsafe { &*(table_phys as *const AcpiHeader) };
        let table_size = header.length as usize;
        
        // 获取签名用于调试
        let sig = unsafe { core::str::from_utf8_unchecked(&header.signature) };
        uefi::println!("Table {}: {:4?} ({} bytes, phys 0x{:x})", 
            i, sig, table_size, table_phys);
        
        result.table_phys_addrs[i] = table_phys;
        result.table_sizes[i] = table_size as u32;
    }
    
    uefi::println!("ACPI tables info collected! Total {} tables", result.table_count);
    Ok(result)
}

/// 方案2：如果需要拷贝到安全的内存区域
/// 在退出 boot services 前拷贝到内核保留的内存区域
pub fn copy_acpi_tables_to_fixed() -> Result<AcpiCopiedData, &'static str> {
    // 定义固定的物理地址区域（例如 2MB 开始，需要与内核约定）
    const ACPI_COPY_BASE: u64 = 0x200000;  // 2MB 地址
    let mut current_offset = 0u64;
    
    // 1. 找到并拷贝 RSDP
    let rsdp_phys = find_rsdp().ok_or("RSDP not found")?;
    let rsdp_size = size_of::<Rsdp>() as u64;
    let rsdp_copy_phys = ACPI_COPY_BASE + current_offset;
    current_offset += rsdp_size;
    
    unsafe {
        copy_nonoverlapping(
            rsdp_phys as *const u8,
            rsdp_copy_phys as *mut u8,
            rsdp_size as usize
        );
    }
    
    // 2. 找到 XSDT
    let rsdp_ptr = rsdp_phys as *const Rsdp;
    let xsdt_phys = unsafe { (*rsdp_ptr).xsdt_address };
    let xsdt_header = unsafe { &*(xsdt_phys as *const AcpiHeader) };
    let xsdt_size = xsdt_header.length as u64;
    let xsdt_copy_phys = ACPI_COPY_BASE + current_offset;
    current_offset += xsdt_size;
    
    unsafe {
        copy_nonoverlapping(
            xsdt_phys as *const u8,
            xsdt_copy_phys as *mut u8,
            xsdt_size as usize
        );
    }
    
    // 3. 拷贝所有表
    let entry_count = (xsdt_size as usize - size_of::<AcpiHeader>()) / 8;
    let mut result = AcpiCopiedData {
        rsdp_phys: rsdp_copy_phys,
        xsdt_phys: xsdt_copy_phys,
        table_count: entry_count.min(32) as u32,
        rsdp_size: rsdp_size as u32,
        xsdt_size: xsdt_size as u32,
        ..Default::default()
    };
    
    for i in 0..entry_count.min(32) {
        let table_phys = unsafe {
            let entry_ptr = (xsdt_phys + size_of::<AcpiHeader>() as u64) as *const u64;
            *entry_ptr.add(i)
        };
        
        let header = unsafe { &*(table_phys as *const AcpiHeader) };
        let table_size = header.length as u64;
        let table_copy_phys = ACPI_COPY_BASE + current_offset;
        current_offset += table_size;
        
        unsafe {
            copy_nonoverlapping(
                table_phys as *const u8,
                table_copy_phys as *mut u8,
                table_size as usize
            );
        }
        
        result.table_phys_addrs[i] = table_copy_phys;
        result.table_sizes[i] = table_size as u32;
        
        let sig = unsafe { core::str::from_utf8_unchecked(&header.signature) };
        uefi::println!("Copied table {}: {:4?} to 0x{:x}", i, sig, table_copy_phys);
    }
    
    Ok(result)
}

// 保留原函数但标记为 deprecated
#[deprecated(note = "Use get_acpi_physical_addresses() instead - this allocates memory that becomes invalid after exit_boot_services")]
pub fn copy_acpi_tables() -> Result<AcpiCopiedData, &'static str> {
    get_acpi_physical_addresses()
}

/// 注意：不再需要 free 函数，因为不再分配内存
/// 如果使用方案2，也不需要释放（固定区域由内核管理）

/// 根据签名查找表（使用物理地址）
pub fn find_table_by_signature(data: &AcpiCopiedData, signature: &[u8; 4]) -> Option<u64> {
    for i in 0..data.table_count as usize {
        let table_phys = data.table_phys_addrs[i];
        if table_phys == 0 {
            continue;
        }
        
        let header = unsafe { &*(table_phys as *const AcpiHeader) };
        if &header.signature == signature {
            return Some(table_phys);
        }
    }
    None
}

/// 打印所有表的信息（物理地址）
pub fn print_tables_info(data: &AcpiCopiedData) {
    uefi::println!("=== ACPI Tables Physical Addresses ===");
    uefi::println!("RSDP at phys: 0x{:x} (size: {} bytes)", data.rsdp_phys, data.rsdp_size);
    uefi::println!("XSDT at phys: 0x{:x} (size: {} bytes)", data.xsdt_phys, data.xsdt_size);
    uefi::println!("Total tables: {}", data.table_count);
    
    for i in 0..data.table_count as usize {
        let phys = data.table_phys_addrs[i];
        let size = data.table_sizes[i];
        
        unsafe {
            let header = &*(phys as *const AcpiHeader);
            let sig = core::str::from_utf8_unchecked(&header.signature);
            uefi::println!("  {}: {:4?} at phys 0x{:x} ({} bytes)", 
                i, sig, phys, size);
        }
    }
    uefi::println!("=========================================");
}