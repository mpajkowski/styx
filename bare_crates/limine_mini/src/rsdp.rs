use core::ptr::NonNull;

use crate::make_struct;

#[repr(C)]
#[derive(Debug)]
pub struct Response {
    revision: u64,
    address: *const u8,
}

impl Response {
    pub fn address(&self) -> Option<NonNull<u8>> {
        NonNull::new(self.address as *mut _)
    }
}

make_struct!(
    struct Request: [0xc5e77b6b397e7b43, 0x27637845accdcf3c] => Response {};
);
