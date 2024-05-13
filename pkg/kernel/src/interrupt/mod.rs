mod apic;
mod consts;
pub mod clock;
mod serial;
mod exceptions;
pub mod syscall;
use x86_64::registers::model_specific::Msr;
use apic::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::memory::physical_to_virtual;
use x86::msr::IA32_APIC_BASE;
use crate::interrupt::consts::Irq;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            exceptions::register_idt(&mut idt);
            clock::register_idt(&mut idt);
            serial::register_idt(&mut idt);
            syscall::register_idt(&mut idt);
        }
        idt
    };
}

/// init interrupts system
pub fn init() {
    IDT.load();

    // FIXME: check and init APIC
    if XApic::support() {
        let apic_base = unsafe { Msr::new(IA32_APIC_BASE).read() };
        let apic_addr = (apic_base & 0xffff_ffff_0000) as u64;

        // Map APIC physical address to virtual address
        let apic_virt_addr = physical_to_virtual(apic_addr);
        let mut apic = unsafe { XApic::new(apic_virt_addr) };
        apic.cpu_init();

        info!("APIC Initialized.");
    } else {
        panic!("APIC not found!");
    }
    // FIXME: enable serial irq with IO APIC (use enable_irq)
    enable_irq(Irq::Serial0 as u8, 0); // enable IRQ4 for CPU0
    info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8, cpuid: u8) {
    let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
    ioapic.enable(irq, cpuid);
}

#[inline(always)]
pub fn ack() {
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
