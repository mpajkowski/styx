use core::fmt::Write;

use limine_mini::terminal;

use crate::x86_64::limine::Limine;

pub struct Terminal {
    resp: &'static terminal::Response,
}

impl Terminal {
    pub fn from_boot_info(boot_info: &Limine) -> Self {
        Self {
            resp: boot_info.terminal,
        }
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let term = self.resp.terminals().get(0).unwrap();
        let cb = self.resp.write().unwrap();

        cb(term, s);

        Ok(())
    }
}

unsafe impl Send for Terminal {}
