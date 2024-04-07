use super::consts::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::idt::InterruptStackFrame;
pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_handler);
}

pub extern "x86-interrupt" fn clock_handler(_sf: InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // if inc_counter() % 0x10000 == 0 {
        //     info!("Tick! @{}", read_counter());
        // }
        inc_counter();
        super::ack();
    });
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn read_counter() -> u64 {
    COUNTER.load(Ordering::Relaxed)
}

#[inline]
pub fn inc_counter() -> u64 {
    COUNTER.fetch_add(1, Ordering::Relaxed) + 1
}