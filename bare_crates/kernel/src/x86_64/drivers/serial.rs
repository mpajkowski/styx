use core::fmt::Write;

use crate::x86_64::ioport;

const COM1: u16 = 0x3f8;

pub struct Serial {
    port: u16,
}

impl Serial {
    pub fn init_com1() -> Result<Self, &'static str> {
        unsafe { init(COM1)? };

        Ok(Self { port: COM1 })
    }

    #[inline(always)]
    pub fn read(&self) -> u8 {
        unsafe {
            while !self.received_data() {}
            ioport::read_u8(self.port)
        }
    }

    #[inline(always)]
    pub fn try_read(&self) -> Option<u8> {
        unsafe { self.received_data().then(|| ioport::read_u8(self.port)) }
    }

    #[inline(always)]
    pub fn write(&self, byte: u8) {
        unsafe {
            while !self.transmit_empty() {}
            ioport::write_u8(self.port, byte)
        }
    }

    #[inline(always)]
    unsafe fn received_data(&self) -> bool {
        ioport::read_u8(self.port + 5) & 1 != 0
    }

    #[inline(always)]
    unsafe fn transmit_empty(&self) -> bool {
        ioport::read_u8(self.port + 5) & 0x20 != 0
    }
}

#[inline(always)]
unsafe fn init(port: u16) -> Result<(), &'static str> {
    ioport::write_u8(port + 1, 0x00); // disable all interrupts
    ioport::write_u8(port + 3, 0x80); // enable DLAB
    ioport::write_u8(port + 0, 0x03); // 38400 baud
    ioport::write_u8(port + 1, 0x00); //
    ioport::write_u8(port + 3, 0x03); // 8 bits, no parity, one stop bit
    ioport::write_u8(port + 2, 0xc7); // enable fifo, clear them, with 14-byte threshold
    ioport::write_u8(port + 4, 0x1e); // set in loopback mode, test the serial chip

    let test = 0x42;
    ioport::write_u8(port + 0, test);

    if ioport::read_u8(port) != test {
        return Err("Failed to initialize port");
    }

    ioport::write_u8(port + 4, 0x0f);

    Ok(())
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();

        bytes.iter().for_each(|byte| self.write(*byte));

        Ok(())
    }
}
