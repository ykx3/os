use core::fmt;
use x86_64::instructions::port::{PortReadOnly,PortWriteOnly};
extern crate bitflags;
use bitflags::bitflags;

bitflags! {
    pub struct LineControlRegister: u8 {
        const DATA_BITS_5 = 0b00;
        const DATA_BITS_6 = 0b01;
        const DATA_BITS_7 = 0b10;
        const DATA_BITS_8 = 0b11;
        const STOP_BITS_1 = 0b0 << 2;
        const STOP_BITS_2 = 0b1 << 2;
        const PARITY_NONE = 0b0 << 3;
        const PARITY_ODD = 0b1 << 3;
        const PARITY_EVEN = 0b1 << 4;
        const STICK_PARITY = 0b1 << 5;
        const ENABLE_INTERRUPT = 0b1 << 6;
        const DLAB = 0b1 << 7;
    }
}
/// A port-mapped UART 16550 serial interface.
pub struct SerialPort{
    base:u16,
}
impl SerialPort {
    pub const fn new(port: u16) -> Self {
        SerialPort { base: port }
    }
    fn outb(port: u16, data: u8){
        let mut pipline = PortWriteOnly::new(port);
        unsafe {
            pipline.write(data);
        }
    }
    fn inb(port: u16)->u8{
        let mut data = PortReadOnly::new(port);
        unsafe{
            data.read()
        }
    }
    fn set_lcr(port: u16, flags: LineControlRegister) {
        let mut lcr = PortWriteOnly::new(port + 3);
        unsafe {
            lcr.write(flags.bits());
        }
    }
    /// Initializes the serial port.
    pub fn init(&self) {
        // FIXME: Initialize the serial port
        SerialPort::outb(self.base + 1, 0x00);    // Disable all interrupts
        SerialPort::set_lcr(self.base, LineControlRegister::DLAB);    // Enable DLAB (set baud rate divisor)
        SerialPort::outb(self.base + 0, 0x03);    // Set divisor to 3 (lo byte) 38400 baud
        SerialPort::outb(self.base + 1, 0x00);    //                  (hi byte)
        SerialPort::set_lcr(self.base, LineControlRegister::DATA_BITS_8 | LineControlRegister::PARITY_NONE | LineControlRegister::STOP_BITS_1);    // 8 bits, no parity, one stop bit
        SerialPort::outb(self.base + 2, 0xC7);    // Enable FIFO, clear them, with 14-byte threshold
        SerialPort::outb(self.base + 4, 0x0B);    // IRQs enabled, RTS/DSR set
        SerialPort::outb(self.base + 4, 0x1E);    // Set in loopback mode, test the serial chip
     
        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        SerialPort::outb(self.base + 4, 0x0F);
    }
    fn is_transmit_empty(&self)->bool {
        SerialPort::inb(self.base + 5) & 0x20 != 0
    }
    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        // FIXME: Send a byte on the serial port
        while (self.is_transmit_empty() == false){}
        SerialPort::outb(self.base,data);
    }

    fn serial_received(&self)->bool {
        SerialPort::inb(self.base + 5) & 1 != 0
    }
    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        while (self.serial_received() == false){}

        Some(SerialPort::inb(self.base))
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
