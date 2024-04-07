use super::LocalApic;
use bit_field::BitField;
use core::fmt::{Debug, Error, Formatter};
use core::ptr::{read_volatile, write_volatile};
use x86::cpuid::CpuId;
use crate::interrupt::consts::{Interrupts, Irq};

/// Default physical address of xAPIC
pub const LAPIC_ADDR: u64 = 0xFEE00000;

pub struct XApic {
    addr: u64,
}

impl XApic {
    pub unsafe fn new(addr: u64) -> Self {
        XApic { addr }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        read_volatile((self.addr + reg as u64) as *const u32)
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        write_volatile((self.addr + reg as u64) as *mut u32, value);
        self.read(0x20);
    }
}

bitflags! {
    struct TimerDivide: u32 {
        const BY_1 = 0b1011;
        const BY_2 = 0b0000;
        const BY_4 = 0b0001;
        const BY_8 = 0b0010;
        const BY_16 = 0b0011;
        const BY_32 = 0b1000;
        const BY_64 = 0b1001;
        const BY_128 = 0b1010;
    }
}

impl LocalApic for XApic {
    
    /// If this type APIC is supported
    fn support() -> bool {
        // FIXME: Check CPUID to see if xAPIC is supported.
        CpuId::new().get_feature_info().map(
            |f| f.has_apic()
        ).unwrap_or(false)
    }

    /// Initialize the xAPIC for the current CPU.
    fn cpu_init(&mut self) {
        const SPIV: u32 = 0xF0;
        const TDCR: u32 = 0x3E0;
        const TICR: u32 = 0x380;
        const LVT_TIMER: u32 = 0x320;
        const LVT_LINT0: u32 = 0x350;
        const LVT_LINT1: u32 = 0x360;
        const LVT_PCINT: u32 = 0x340;
        const LVT_ERROR: u32 = 0x370;
        const ERROR_STATUS: u32 = 0x280;
        const EOI: u32 = 0xB0;
        const ICR_LOW: u32 = 0x300;
        const ICR_HIGH: u32 = 0x310;
        const MASK: u32 = 1 << 16;
        unsafe {
            // FIXME: Enable local APIC; set spurious interrupt vector.
            let mut spiv = self.read(SPIV);
            spiv |= 1 << 8; // set EN bit
            // clear and set Vector
            spiv &= !(0xFF);
            spiv |= Interrupts::IrqBase as u32 + Irq::Spurious as u32;
            self.write(SPIV, spiv);
            // FIXME: The timer repeatedly counts down at bus frequency
            let mut lvt_timer = self.read(LVT_TIMER);
            // clear and set Vector
            self.write(TDCR, TimerDivide::BY_1.bits()); // set Timer Divide to 1
            self.write(TICR, 0x20000); // set initial count to 0x20000
            lvt_timer &= !(0xFF);
            lvt_timer |= Interrupts::IrqBase as u32 + Irq::Timer as u32;
            lvt_timer &= !MASK; // clear Mask
            lvt_timer |= 1 << 17; // set Timer Periodic Mode
            self.write(LVT_TIMER, lvt_timer);
            // FIXME: Disable logical interrupt lines (LINT0, LINT1)
            self.write(LVT_LINT0, MASK); // set Mask for LINT0
            self.write(LVT_LINT1, MASK); // set Mask for LINT1
            // FIXME: Disable performance counter overflow interrupts (PCINT)
            self.write(LVT_PCINT, MASK); // set Mask for PCINT
            // FIXME: Map error interrupt to IRQ_ERROR.
            let mut lvt_error = self.read(LVT_ERROR);
            lvt_error &= !(0xFF); // clear Vector
            lvt_error |= Interrupts::IrqBase as u32 + Irq::Error as u32; // set Vector
            self.write(LVT_ERROR, lvt_error);
            // FIXME: Clear error status register (requires back-to-back writes).
            self.write(ERROR_STATUS, 0);
            self.write(ERROR_STATUS, 0);
            
            // FIXME: Ack any outstanding interrupts.
            self.write(EOI, 0);
            // FIXME: Send an Init Level De-Assert to synchronise arbitration ID's.
            self.write(ICR_HIGH, 0); // set ICR 0x310
            const BCAST: u32 = 1 << 19;
            const INIT: u32 = 5 << 8;
            const TMLV: u32 = 1 << 15; // TM = 1, LV = 0
            self.write(ICR_LOW, BCAST | INIT | TMLV); // set ICR 0x300
            const DS: u32 = 1 << 12;
            while self.read(ICR_LOW) & DS != 0 {} // wait for delivery status
            // FIXME: Enable interrupts on the APIC (but not on the processor).
        }

        // NOTE: Try to use bitflags! macro to set the flags.
    }

    fn id(&self) -> u32 {
        // NOTE: Maybe you can handle regs like `0x0300` as a const.
        unsafe { self.read(0x0020) >> 24 }
    }

    fn version(&self) -> u32 {
        unsafe { self.read(0x0030) }
    }

    fn icr(&self) -> u64 {
        unsafe { (self.read(0x0310) as u64) << 32 | self.read(0x0300) as u64 }
    }

    fn set_icr(&mut self, value: u64) {
        unsafe {
            while self.read(0x0300).get_bit(12) {}
            self.write(0x0310, (value >> 32) as u32);
            self.write(0x0300, value as u32);
            while self.read(0x0300).get_bit(12) {}
        }
    }

    fn eoi(&mut self) {
        unsafe {
            self.write(0x00B0, 0);
        }
    }
}

impl Debug for XApic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Xapic")
            .field("id", &self.id())
            .field("version", &self.version())
            .field("icr", &self.icr())
            .finish()
    }
}
