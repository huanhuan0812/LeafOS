use core::ptr::copy_nonoverlapping;
use core::mem::size_of;
use uefi::system::with_config_table;
use uefi::table::cfg::{ConfigTableEntry, SMBIOS_GUID, SMBIOS3_GUID};
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

// SMBIOS Entry Point 结构 (32位版本)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct SmbiosEp {
    pub signature: [u8; 4],
    pub checksum: u8,
    pub length: u8,
    pub major_version: u8,
    pub minor_version: u8,
    pub max_struct_size: u16,
    pub revision: u8,
    pub formatted_area: [u8; 5],
    pub intermediate_checksum: u8,
    pub table_length: u16,
    pub table_address: u32,
    pub struct_count: u16,
    pub bcd_revision: u8,
}

// SMBIOS 3.0 Entry Point 结构
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Smbios3Ep {
    pub signature: [u8; 5],
    pub checksum: u8,
    pub length: u8,
    pub major_version: u8,
    pub minor_version: u8,
    pub docrev: u8,
    pub revision: u8,
    pub reserved: u8,
    pub table_length: u32,
    pub table_address: u64,
}

#[repr(C)]
pub struct AcpiCopiedData {
    pub rsdp_phys: u64,
    pub xsdt_phys: u64,
    pub table_count: u32,
    pub table_phys_addrs: [u64; 32],
    pub table_sizes: [u32; 32],
    pub rsdp_size: u32,
    pub xsdt_size: u32,
    pub smbios_phys: u64,
    pub smbios_length: u32,
    pub smbios_version: u32,
    pub smbios_table_count: u16,
    pub smbios_type: u8,
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
            smbios_phys: 0,
            smbios_length: 0,
            smbios_version: 0,
            smbios_table_count: 0,
            smbios_type: 0,
        }
    }
}

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

fn checksum(data: *const u8, len: usize) -> u8 {
    let mut sum: u8 = 0;
    unsafe {
        for i in 0..len {
            sum = sum.wrapping_add(*data.add(i));
        }
    }
    sum
}

fn find_smbios_entry() -> Option<(u64, usize, u32, u16)> {
    let mut result = None;
    
    with_config_table(|slice| {
        for entry in slice {
            if entry.guid == SMBIOS_GUID {
                let ep = entry.address as *const SmbiosEp;
                unsafe {
                    if &(*ep).signature == b"_SM_" {
                        let len = (*ep).length as usize;
                        if checksum(entry.address as *const u8, len) == 0 {
                            let version = ((*ep).major_version as u32) << 16 | (*ep).minor_version as u32;
                            result = Some(((*ep).table_address as u64, 
                                          (*ep).table_length as usize,
                                          version,
                                          (*ep).struct_count));
                            return;
                        }
                    }
                }
            } else if entry.guid == SMBIOS3_GUID {
                let ep = entry.address as *const Smbios3Ep;
                unsafe {
                    if &(*ep).signature == b"_SM3_" {
                        let len = (*ep).length as usize;
                        if checksum(entry.address as *const u8, len) == 0 {
                            let version = ((*ep).major_version as u32) << 16 | (*ep).minor_version as u32;
                            result = Some(((*ep).table_address, 
                                          (*ep).table_length as usize,
                                          version,
                                          0));
                            return;
                        }
                    }
                }
            }
        }
    });
    
    if let Some(res) = result {
        return Some(res);
    }
    
    // 扫描 BIOS 区域
    let bios_start = 0xF0000u64;
    let bios_end = 0x100000u64;
    let mut current = bios_start;
    
    while current + 32 <= bios_end {
        unsafe {
            let sig = *(current as *const u32);
            if sig == 0x5f4d535f {
                let ep = current as *const SmbiosEp;
                let length = (*ep).length as usize;
                if current + length as u64 <= bios_end {
                    if checksum(current as *const u8, length) == 0 {
                        let version = ((*ep).major_version as u32) << 16 | (*ep).minor_version as u32;
                        return Some(((*ep).table_address as u64, 
                                   (*ep).table_length as usize,
                                   version,
                                   (*ep).struct_count));
                    }
                }
            }
            
            let sig5 = *(current as *const [u8; 5]);
            if &sig5 == b"_SM3_" {
                let ep = current as *const Smbios3Ep;
                let length = (*ep).length as usize;
                if current + length as u64 <= bios_end {
                    if checksum(current as *const u8, length) == 0 {
                        let version = ((*ep).major_version as u32) << 16 | (*ep).minor_version as u32;
                        return Some(((*ep).table_address, 
                                   (*ep).table_length as usize,
                                   version,
                                   0));
                    }
                }
            }
        }
        current += 16;
    }
    
    None
}

pub fn get_acpi_physical_addresses() -> Result<AcpiCopiedData, &'static str> {
    let rsdp_phys = find_rsdp().ok_or("RSDP not found")?;
    uefi::println!("RSDP found at physical address: 0x{:x}", rsdp_phys);
    
    let rsdp_ptr = rsdp_phys as *const Rsdp;
    if unsafe { &(*rsdp_ptr).signature } != b"RSD PTR " {
        return Err("Invalid RSDP signature");
    }
    
    if checksum(rsdp_phys as *const u8, 20) != 0 {
        return Err("RSDP checksum failed");
    }
    
    let revision = unsafe { (*rsdp_ptr).revision };
    if revision < 2 {
        return Err("Only ACPI 2.0+ supported");
    }
    
    let xsdt_phys = unsafe { (*rsdp_ptr).xsdt_address };
    uefi::println!("XSDT found at physical address: 0x{:x}", xsdt_phys);
    
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
    
    for i in 0..entry_count.min(32) {
        let table_phys = unsafe {
            let entry_ptr = (xsdt_phys + size_of::<AcpiHeader>() as u64) as *const u64;
            *entry_ptr.add(i)
        };
        
        let header = unsafe { &*(table_phys as *const AcpiHeader) };
        let table_size = header.length as usize;
        let sig = unsafe { core::str::from_utf8_unchecked(&header.signature) };
        uefi::println!("Table {}: {:4?} ({} bytes, phys 0x{:x})", 
            i, sig, table_size, table_phys);
        
        result.table_phys_addrs[i] = table_phys;
        result.table_sizes[i] = table_size as u32;
    }
    
    // 精简的 SMBIOS 检测
    if let Some((phys, len, version, struct_count)) = find_smbios_entry() {
        result.smbios_phys = phys;
        result.smbios_length = len as u32;
        result.smbios_version = version;
        result.smbios_table_count = struct_count;
        result.smbios_type = if (version >> 16) >= 3 { 1 } else { 0 };
        
        let major = version >> 16;
        let minor = version & 0xFFFF;
        uefi::println!("SMBIOS {}.{} at phys 0x{:x} ({} bytes, {} structures)",
            major, minor, phys, len, struct_count);
    } else {
        uefi::println!("SMBIOS not found");
    }
    
    uefi::println!("ACPI tables info collected! Total {} tables", result.table_count);
    Ok(result)
}

pub fn copy_acpi_tables_to_fixed() -> Result<AcpiCopiedData, &'static str> {
    const ACPI_COPY_BASE: u64 = 0x200000;
    let mut current_offset = 0u64;
    
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
    
    if let Some((smbios_phys, smbios_len, version, struct_count)) = find_smbios_entry() {
        let smbios_copy_phys = ACPI_COPY_BASE + current_offset;
        current_offset += smbios_len as u64;
        
        unsafe {
            copy_nonoverlapping(
                smbios_phys as *const u8,
                smbios_copy_phys as *mut u8,
                smbios_len
            );
        }
        
        result.smbios_phys = smbios_copy_phys;
        result.smbios_length = smbios_len as u32;
        result.smbios_version = version;
        result.smbios_table_count = struct_count;
        result.smbios_type = if (version >> 16) >= 3 { 1 } else { 0 };
        
        let major = version >> 16;
        let minor = version & 0xFFFF;
        uefi::println!("Copied SMBIOS table to 0x{:x} (version {}.{}, length: {} bytes)",
            smbios_copy_phys, major, minor, smbios_len);
    }
    
    Ok(result)
}

#[deprecated(note = "Use get_acpi_physical_addresses() instead")]
pub fn copy_acpi_tables() -> Result<AcpiCopiedData, &'static str> {
    get_acpi_physical_addresses()
}

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

pub fn get_smbios_info(data: &AcpiCopiedData) -> Option<(u64, u32, u32, u16, u8)> {
    if data.smbios_phys == 0 {
        None
    } else {
        Some((data.smbios_phys, data.smbios_length, data.smbios_version, 
              data.smbios_table_count, data.smbios_type))
    }
}

pub fn print_tables_info(data: &AcpiCopiedData) {
    uefi::println!("=========================================");
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
    
    uefi::println!("\n=== SMBIOS Information ===");
    if data.smbios_phys != 0 {
        let major = data.smbios_version >> 16;
        let minor = data.smbios_version & 0xFFFF;
        uefi::println!("SMBIOS version {}.{}", major, minor);
        uefi::println!("SMBIOS table at phys: 0x{:x} (size: {} bytes)", 
            data.smbios_phys, data.smbios_length);
        if data.smbios_table_count > 0 {
            uefi::println!("Total structures: {}", data.smbios_table_count);
        }
    } else {
        uefi::println!("SMBIOS not available");
    }
    uefi::println!("=========================================");
}