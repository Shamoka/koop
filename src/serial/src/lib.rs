#![no_std]

use asm::x86_64::mmio;

#[repr(u16)]
#[derive(Copy, Clone)]
pub enum ComAddr {
    Com1 = 0x3F8
}

pub struct Port {
    data: mmio::Port,
    irq_enable: mmio::Port,
    irq_id: mmio::Port,
    line_control: mmio::Port,
    modem_control: mmio::Port,
    line_status: mmio::Port,
    _modem_status: mmio::Port,
    _scratch: mmio::Port
}

impl Port {
    pub unsafe fn new(port: ComAddr) -> Port {
        let serial = Port {
            data: mmio::Port::new(port as usize),
            irq_enable: mmio::Port::new(port as usize + 1),
            irq_id: mmio::Port::new(port as usize + 2),
            line_control: mmio::Port::new(port as usize + 3),
            modem_control: mmio::Port::new(port as usize + 4),
            line_status: mmio::Port::new(port as usize + 5),
            _modem_status: mmio::Port::new(port as usize + 6),
            _scratch: mmio::Port::new(port as usize + 7)
        };
        serial.init();
        serial
    }

    unsafe fn init(&self) {
        self.irq_enable.write(0x00 as u8);
        self.line_control.write(0x80 as u8);
        self.data.write(0x03 as u8);
        self.irq_enable.write(0x00 as u8);
        self.line_control.write(0x03 as u8);
        self.irq_id.write(0xC7 as u8);
        self.modem_control.write(0x0B as u8);
    }

    pub unsafe fn read(&self) -> u8 {
        while self.line_status.read() & 0x01 == 0 {}
        self.data.read()
    }

    pub unsafe fn write(&self, byte: u8) {
        while self.line_status.read() & 0x20 == 0 {}
        self.data.write(byte);
    }

    pub unsafe fn write_str(&self, s: &str) {
        for byte in s.bytes() {
            self.write(byte);
        }
    }
}
