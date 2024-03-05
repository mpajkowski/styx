use core::{ffi::CStr, mem::MaybeUninit};

use crate::make_struct;

#[repr(C)]
#[derive(Debug)]
pub struct Response {
    revision: u64,
    file: *const File,
}

#[repr(C)]
pub struct File {
    revision: u64,
    addr: *mut u8,
    size: u64,
    path: *const i8,
    cmdline: *const i8,
    media_type: u32,
    _unused: MaybeUninit<u32>,
    tftp_ip: u32,
    tftp_port: u32,
    partition_idx: u32,
    mbr_disk_id: u32,
    gpt_disk_id: Uuid,
    gpt_partition_id: Uuid,
    partition_uuid: Uuid,
}

#[repr(C)]
pub struct Uuid {
    a: u32,
    b: u16,
    c: u16,
    d: [u8; 8],
}

impl Response {
    pub fn path(&self) -> &[u8] {
        unsafe { CStr::from_ptr(self.file().path) }.to_bytes()
    }

    pub fn cmdline(&self) -> &[u8] {
        unsafe { CStr::from_ptr(self.file().cmdline) }.to_bytes()
    }

    pub fn file(&self) -> &'static File {
        unsafe { &*self.file }
    }
}

make_struct!(
    struct Request: [0xad97e90e83f1ed67, 0x31eb5d1c5ff23b69] => Response {};
);
