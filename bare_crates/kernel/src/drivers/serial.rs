use core::fmt::Write;

use super::{ioport::read_u8, write_u8};

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
            while !received_data(self.port) {}
            read_u8(self.port)
        }
    }

    #[inline(always)]
    pub fn try_read(&self) -> Option<u8> {
        unsafe { received_data(self.port).then(|| read_u8(self.port)) }
    }

    #[inline(always)]
    pub fn write(&self, byte: u8) {
        unsafe {
            while !transmit_empty(self.port) {}
            write_u8(self.port, byte)
        }
    }
}

#[inline(always)]
unsafe fn init(port: u16) -> Result<(), &'static str> {
    write_u8(port + 1, 0x00); // disable all interrupts
    write_u8(port + 3, 0x80); // enable DLAB
    write_u8(port + 0, 0x03); // 38400 baud
    write_u8(port + 1, 0x00); //
    write_u8(port + 3, 0x03); // 8 bits, no parity, one stop bit
    write_u8(port + 2, 0xc7); // enable fifo, clear them, with 14-byte threshold
    write_u8(port + 4, 0x1e); // set in loopback mode, test the serial chip

    let test = 0x42;
    write_u8(port + 0, test);

    if read_u8(port) != test {
        return Err("Failed to initialize port");
    }

    write_u8(port + 4, 0x0f);

    Ok(())
}

#[inline(always)]
unsafe fn received_data(port: u16) -> bool {
    read_u8(port + 5) & 1 != 0
}

#[inline(always)]
unsafe fn transmit_empty(port: u16) -> bool {
    read_u8(port + 5) & 0x20 != 0
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();

        bytes.iter().for_each(|byte| self.write(*byte));

        Ok(())
    }
}
