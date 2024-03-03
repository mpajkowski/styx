use crate::x86_64::ioport;
use crate::x86_64::sync::Mutex;

/// Master PIC vector offset
pub const PIC_1_OFFSET: u8 = 32;
/// Slave PIC vector offset
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// PICs handle
pub static PIC8529PAIR: Mutex<Pic8529Pair> =
    Mutex::new(Pic8529Pair::new(PIC_1_OFFSET, PIC_2_OFFSET));

pub fn init() {
    PIC8529PAIR.lock().init_inner();

    log::info!("Legacy PIC initialized");
}

/// Pair of Pic8259
pub struct Pic8529Pair {
    first: Chip,
    second: Chip,
}

impl Pic8529Pair {
    /// Creates PICs handle from given offsets
    pub const fn new(offset1: u8, offset2: u8) -> Self {
        Self {
            first: Chip {
                offset: offset1,
                command_port: 0x20,
                data_port: 0x21,
            },
            second: Chip {
                offset: offset2,
                command_port: 0xa0,
                data_port: 0xa1,
            },
        }
    }

    fn init_inner(&mut self) {
        let masks = self.read_masks();
        self.write_cmd(Command::Init);
        self.write_data(self.first.offset, self.second.offset);
        self.write_data(0x04, 0x02);
        self.write_cmd(Command::Mode8086);
        self.write_masks(masks);
    }

    fn read_masks(&mut self) -> [u8; 2] {
        let mask1 = self.first.read_mask();
        let mask2 = self.second.read_mask();

        [mask1, mask2]
    }

    /// Notify PICs that the interrupt has ended.
    ///
    /// NOTE First chip will be notified also if the interrupt is handled by a second chip.
    pub fn notify_end_of_interrupt(&mut self, interrupt: u8) {
        let notify_second = self.second.in_interrupt(interrupt);

        if notify_second {
            self.second.notify_end_of_interrupt();
        }

        self.first.notify_end_of_interrupt();
    }

    fn write_masks(&mut self, masks: [u8; 2]) {
        self.first.write_mask(masks[0]);
        self.second.write_mask(masks[1]);
    }

    /// Masks all legacy PICs interrupts
    pub fn disable(&mut self) {
        self.write_masks([0xff; 2]);
    }

    fn write_cmd(&mut self, command: Command) {
        let command = command as u8;
        unsafe {
            ioport::write_u8(self.first.command_port, command);
            ioport::wait();
            ioport::write_u8(self.second.command_port, command);
            ioport::wait();
        }
    }

    fn write_data(&mut self, data1: u8, data2: u8) {
        unsafe {
            ioport::write_u8(self.first.data_port, data1);
            ioport::wait();
            ioport::write_u8(self.second.data_port, data2);
            ioport::wait();
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Init = 0x11,
    EndOfInterrupt = 0x20,
    Mode8086 = 0x01,
}

struct Chip {
    offset: u8,
    command_port: u16,
    data_port: u16,
}

impl Chip {
    const fn in_interrupt(&self, interrupt: u8) -> bool {
        self.offset <= interrupt && interrupt < self.offset + 8
    }

    fn notify_end_of_interrupt(&mut self) {
        unsafe { ioport::write_u8(self.command_port, Command::EndOfInterrupt as u8) }
    }

    fn read_mask(&mut self) -> u8 {
        unsafe { ioport::read_u8(self.data_port) }
    }

    fn write_mask(&mut self, mask: u8) {
        unsafe { ioport::write_u8(self.data_port, mask) }
    }
}
