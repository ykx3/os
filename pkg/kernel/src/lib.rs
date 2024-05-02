#![no_std]
#![allow(dead_code)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(type_alias_impl_trait)]
#![feature(panic_info_message)]
#![feature(map_try_insert)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::result_unit_err)]

extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;
extern crate libm;

#[macro_use]
pub mod utils;
pub use utils::*;

#[macro_use]
pub mod drivers;
pub use drivers::*;

pub mod memory;
pub mod interrupt;

pub use alloc::format;
use boot::BootInfo;
pub mod proc;
pub fn init(boot_info: &'static BootInfo) {
    serial::init(); // init serial output
    logger::init(); // init logger system
    memory::address::init(boot_info);
    memory::gdt::init(); // init gdt
    memory::allocator::init(); // init kernel heap allocator
    proc::init();
    interrupt::init(); // init interrupts
    memory::init(boot_info); // init memory manager

    x86_64::instructions::interrupts::enable();
    info!("Interrupts Enabled.");

    info!("YatSenOS initialized.");
}

pub fn shutdown(boot_info: &'static BootInfo) -> ! {
    info!("YatSenOS shutting down.");
    unsafe {
        boot_info.system_table.runtime_services().reset(
            boot::ResetType::SHUTDOWN,
            boot::UefiStatus::SUCCESS,
            None,
        );
    }
}
fn humanized_size(size: u64) -> (f64, &'static str) {
    let mut num: f64 = size as f64;
    let mut i = 0;
    while num > 1024_f64 {
    num /= 1024_f64;
    i += 1;
    }
    match i {
        0 => (num, "B"),
        1 => (num, "KiB"),
        2 => (num, "MiB"),
        3 => (num, "GiB"),
        _ => (num, "unknow")
    }
   }