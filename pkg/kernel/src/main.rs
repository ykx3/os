#![no_std]
#![no_main]

use ysos::*;
use ysos_kernel as ysos;
extern crate alloc;
use core::arch::asm;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    // unsafe {
    //     asm!(
    //         "mov rax, 28",
    //         "mov [0xDEADBEEF], rax",
    //         options(nostack)
    //     );
    // }
    loop {
        print!("> ");
        let input = input::get_line();
        match input.trim() {
            "exit" => break,
            _ => {
                println!("You said: {}", input);
                println!("The counter value is {}", interrupt::clock::read_counter());
            }
        }
    }

    ysos::shutdown(boot_info);
}