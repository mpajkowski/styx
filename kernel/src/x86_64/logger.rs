//! Kernel logger

use core::fmt::Write;

use config::Config;
use log::{Log, Record};
use spin::Once;

use crate::Terminal;

use super::cpulocal::CpuLocal;
use super::drivers::Serial;
use super::sync::Mutex;

static LOGGER: Once<Logger> = Once::new();

#[allow(unused)]
struct Logger {
    level: log::LevelFilter,
    writers: Mutex<Writers>,
}

struct Writers {
    serial: Option<Serial>,
    terminal: Terminal,
}

impl Writers {
    pub fn new(serial: Serial, terminal: Terminal, config: &Config) -> Self {
        let serial = config.com1.then_some(serial);

        Self { serial, terminal }
    }

    pub fn log(&mut self, logger: &Logger, record: &log::Record) {
        if let Some(serial) = self.serial.as_mut() {
            logger.log(serial, record);
        }

        logger.log(&mut self.terminal, record);
    }
}

impl Logger {
    fn log(&self, writer: &mut impl Write, record: &Record) {
        let cpu = CpuLocal::obtain()
            .map(|cpuinfo| cpuinfo.info.lapic_id)
            .unwrap_or(0);

        let _ = write!(writer, "[CPU {cpu:2}] ");

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
            let mut writers = self.writers.lock_disabling_interrupts();

            writers.log(self, record);
        }
    }

    fn flush(&self) {}
}

/// Initializes logger
pub fn initialize(serial: Serial, terminal: Terminal, config: &Config) {
    let logger = Logger {
        level: log::LevelFilter::Trace,
        writers: Mutex::new(Writers::new(serial, terminal, config)),
    };

    let logger = LOGGER.call_once(|| logger);

    log::set_logger(logger)
        .map(|_| log::set_max_level(config.log))
        .unwrap();
}
