//! Kernel logger

use core::fmt::Write;

use log::{LevelFilter, Log, Record};
use spin::Once;

use crate::{
    drivers::{Serial, Terminal},
    sync::Mutex,
};

static LOGGER: Once<Logger> = Once::new();

#[allow(unused)]
struct Logger {
    level: log::LevelFilter,
    term: Mutex<(Terminal, Serial)>,
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
            let mut outputs = self.term.lock_disabling_interrupts();

            self.log(&mut outputs.0, record);
            self.log(&mut outputs.1, record);
        }
    }

    fn flush(&self) {}
}

/// Initializes logger
pub fn initialize(term: Terminal, serial: Serial) {
    let logger = Logger {
        level: log::LevelFilter::Trace,
        term: Mutex::new((term, serial)),
    };

    let logger = LOGGER.call_once(|| logger);

    log::set_logger(logger)
        .map(|_| log::set_max_level(LevelFilter::Trace))
        .unwrap();
}
