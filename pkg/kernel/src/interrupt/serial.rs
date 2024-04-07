use super::consts::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::drivers;
use crate::drivers::serial::get_serial;
use x86_64::structures::idt::InterruptStackFrame;
pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8]
        .set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}
/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    // FIXME: receive character from uart 16550, put it into INPUT_BUFFER
    // println!("keyboard interrupt");
    let mut serial_port = get_serial().expect("get serial failed");
    while let Some(byte) = serial_port.receive() {
        drivers::input::push_key(byte);
    }
}
