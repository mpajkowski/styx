use core::ffi::CStr;

use crate::file::File;
use crate::make_struct;

#[repr(C)]
#[derive(Debug)]
pub struct Response {
    revision: u64,
    file: *const File,
}

impl Response {
    pub fn cmdline(&self) -> &[u8] {
        unsafe { CStr::from_ptr(self.file().cmdline) }.to_bytes()
    }

    pub fn file(&self) -> &'static File {
        unsafe { &*self.file }
    }
}

make_struct!(
    struct Request: [0xad97e90e83f1ed67, 0x31eb5d1c5ff23b69] => Response {}
);
