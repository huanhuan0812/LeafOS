#![no_std]
#![no_main]

use uefi::{prelude::*, println};

mod boot;
mod kernel;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    println!("UEFI Version: {}.{}", uefi::system::uefi_revision().major(), uefi::system::uefi_revision().minor());
    println!("Firmware Vendor: {}", uefi::system::firmware_vendor());
    println!("Firmware Revision: {}", uefi::system::firmware_revision());
    println!("-------------------------");
    
    let memory_info = boot::memory::MemoryInfo::new();
    let memory_grams = match memory_info {
        Ok(info) => {
            info.print_info();
            info.get_boot_params()
        },
        Err(e) => {
            println!("Failed to get memory information: {:?}", e);
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

    if memory_grams.magic != 0x4C656146 {
        println!("Invalid boot parameters magic: {:X}", memory_grams.magic);
        return Status::LOAD_ERROR;
    }

    let acpi_data = match boot::acpi::get_acpi_physical_addresses() {
        Ok(data) => {
            println!("✓ ACPI/SMBIOS data collected");
            boot::acpi::print_tables_info(&data);
            data
        },
        Err(e) => {
            println!("✗ Failed to get ACPI: {}", e);
            boot::acpi::AcpiCopiedData::default()
        }
    };

    // 获取随机数种子
    let seed =  match boot::rng::get_random_seed() {
        Ok(seed) => {
            println!("✓ Random seed obtained");
            seed
        },
        Err(e) => {
            println!("✗ Failed to get random seed: {:?}", e);
            [0u8; 32] // 返回默认种子
        }
    };
    
    println!("-------------------------");
    uefi::println!("Exiting UEFI boot services...");

    let _ = unsafe {
        uefi::boot::exit_boot_services(None);
    };

    kernel::kernel::kernel_main(&memory_grams, &acpi_data, &seed);

    Status::SUCCESS
}