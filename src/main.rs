#![no_std]
#![no_main]

use core::arch::asm;

use uefi::{prelude::*, println};
// use uefi::system::*;

mod boot;
mod kernel;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    println!("Hello, world! -- from Rust UEFI Kernel!");
    println!("UEFI Version: {}.{}", uefi::system::uefi_revision().major(), uefi::system::uefi_revision().minor());
    println!("Firmware Vendor: {}", uefi::system::firmware_vendor());
    println!("Firmware Revision: {}", uefi::system::firmware_revision());
    println!("-------------------------");
    let memory_info =  boot::memory::memory_init();
    for region in memory_info {
        println!("Region: {}, Size: {}", region.type_str, region.size);
    }
    println!("-------------------------");
    boot::exit::exit_uefi();

    // 进入内核

    // 这里是简化测试，直接进入死循环，等待内核接管
    loop{
        unsafe { asm!("hlt") };
    }

    Status::SUCCESS
}