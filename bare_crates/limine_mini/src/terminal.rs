use crate::framebuffer::Framebuffer;

type WriteFn =
    Option<unsafe extern "C" fn(terminal: *const Terminal, string: *const u8, length: u64)>;

#[repr(C)]
pub struct Response {
    revision: u64,

    terminal_count: u64,

    terminals: *mut Terminal,

    write_fn: WriteFn,
}

impl Response {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn terminals(&self) -> &[Terminal] {
        unsafe { core::slice::from_raw_parts(self.terminals, self.terminal_count as usize) }
    }

    pub fn write(&self) -> Option<impl Fn(&Terminal, &str)> {
        let term_func = self.write_fn?;

        Some(move |terminal: &Terminal, txt: &str| unsafe {
            term_func(terminal as *const _, txt.as_ptr(), txt.len() as u64);
        })
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Terminal {
    /// Number of columns provided by the terminal.
    cols: u64,
    /// Number of rows provided by the terminal.
    rows: u64,
    /// The framebuffer associated with this terminal.
    framebuffer: *mut Framebuffer,
}

crate::make_struct!(
    /// Omitting this request will cause the bootloader to not initialise the terminal service.
    struct Request: [0xc8ac59310c2b0844, 0xa68d0c7265d38878] => Response {
        callback: *const () = core::ptr::null()
    };
);
