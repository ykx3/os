use super::consts::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::drivers::input::push_key;
use crate::serial::get_serial;
use x86_64::structures::idt::InterruptStackFrame;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8]
        .set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    info!("keyboard interrupt");
    receive();
    super::ack();
}

fn receive() {
    let mut serial_port = get_serial().expect("get serial failed");
    let mut buffer = [0u8; 4];
    let mut len = 0;
    info!("receive");
    while let Some(byte) = serial_port.receive() {
        buffer[len] = byte;
        len += 1;

        if let Ok(s) = core::str::from_utf8(&buffer[..len]) {
            if let Some(ch) = s.chars().next() {
                push_key(ch);
                len = 0; // Reset buffer after successfully decoding a character
            }
        }

        if len == 4 {
            len = 0; // Reset buffer if full but no valid character decoded
        }
    }
}
