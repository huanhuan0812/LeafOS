#![no_main]
#![no_std]

use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    uefi::println!("Hello, world! -- from Rust UEFI Kernel!");
    Status::SUCCESS
}