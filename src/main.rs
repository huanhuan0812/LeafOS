#![no_std]
#![no_main]

use uefi::{prelude::*, println};
// use uefi::system::*;

mod boot;
mod kernel;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    //println!("Hello, world! -- from Rust UEFI Kernel!");
    println!("UEFI Version: {}.{}", uefi::system::uefi_revision().major(), uefi::system::uefi_revision().minor());
    println!("Firmware Vendor: {}", uefi::system::firmware_vendor());
    println!("Firmware Revision: {}", uefi::system::firmware_revision());
    println!("-------------------------");
    // 获取内存信息
    let memory_info = boot::memory::MemoryInfo::new();
    let memory_grams= match memory_info {
        Ok(info) => {
            info.print_info();
            info.get_boot_params()
        },
        Err(e) => {
            println!("Failed to get memory information: {:?}", e);
            // Return a default MemoryParams instance or handle the error appropriately
            boot::memory::MemoryParams {
                magic: 0,
                memory_map_phys: 0,
                memory_map_entries: 0,
                memory_map_size: 0,
                usable_ram_size: 0,
            }
        }
    };
    println!("-------------------------");

    if memory_grams.magic != 0x4C656146 { // 如果魔数不匹配，说明内存信息获取失败；退出
        println!("Invalid boot parameters magic: {:X}", memory_grams.magic);
        return Status::LOAD_ERROR;
    }

    let acpi_data = match boot::acpi::get_acpi_physical_addresses() {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to get ACPI physical addresses: {:?}", e);
            // 继续执行，但 ACPI 信息为空
            boot::acpi::AcpiCopiedData::default()
        }
    };
    
    
    uefi::println!("Exiting UEFI boot services...");

    let _ = unsafe {
        uefi::boot::exit_boot_services(None);
    };

    

    // 进入内核引导阶段

    // 进入内核
    kernel::kernel::kernel_main(&memory_grams, &acpi_data);

    Status::SUCCESS
}