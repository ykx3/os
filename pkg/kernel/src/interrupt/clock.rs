use crate::{memory::gdt, proc::print_process_list};
use crate::proc;
use super::consts::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::idt::InterruptStackFrame;
use crate::proc::ProcessContext;
pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_handler)
        .set_stack_index(gdt::CLOCK_INT_IST_INDEX);
}

pub extern "C" fn clock(mut context: ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        proc::switch(&mut context);
        super::ack();
    })
}

as_handler!(clock);