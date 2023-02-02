//! Kernel logger

use core::fmt::Write;

use log::{LevelFilter, Log, Record};
use spin::Once;

use super::drivers::Serial;
use super::sync::Mutex;

static LOGGER: Once<Logger> = Once::new();

#[allow(unused)]
struct Logger {
    level: log::LevelFilter,
    serial: Mutex<Serial>,
}

impl Logger {
    fn log(&self, writer: &mut impl Write, record: &Record) {
        if let Some((file, line)) = record.file().zip(record.line()) {
            let _ = write!(writer, "{file}:{line} ");
        }
        let _ = write!(writer, "{:5} ", record.level().as_str());
        let _ = writer.write_fmt(*record.args());
        let _ = writer.write_char('\n');
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.level >= metadata.level()
    }

    fn log(&self, record: &log::Record) {
        let metadata = record.metadata();

        if self.enabled(metadata) {
            let mut serial = self.serial.lock_disabling_interrupts();

            self.log(&mut *serial, record);
        }
    }

    fn flush(&self) {}
}

/// Initializes logger
pub fn initialize(serial: Serial) {
    let logger = Logger {
        level: log::LevelFilter::Trace,
        serial: Mutex::new(serial),
    };

    let logger = LOGGER.call_once(|| logger);

    log::set_logger(logger)
        .map(|_| log::set_max_level(LevelFilter::Trace))
        .unwrap();
}
