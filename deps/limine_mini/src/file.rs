use core::{ffi::CStr, mem::MaybeUninit};

#[repr(C)]
pub struct File {
    pub(crate) revision: u64,
    pub(crate) addr: *mut u8,
    pub(crate) size: u64,
    pub(crate) path: *const i8,
    pub(crate) cmdline: *const i8,
    pub(crate) media_type: u32,
    pub(crate) _unused: MaybeUninit<u32>,
    pub(crate) tftp_ip: u32,
    pub(crate) tftp_port: u32,
    pub(crate) partition_idx: u32,
    pub(crate) mbr_disk_id: u32,
    pub(crate) gpt_disk_id: Uuid,
    pub(crate) gpt_partition_id: Uuid,
    pub(crate) partition_uuid: Uuid,
}

impl File {
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.addr, self.size as usize) }
    }

    pub fn path(&self) -> Option<&[u8]> {
        if self.path.is_null() {
            return None;
        }

        Some(unsafe { CStr::from_ptr(self.path) }.to_bytes())
    }
}

#[repr(C)]
pub struct Uuid {
    a: u32,
    b: u16,
    c: u16,
    d: [u8; 8],
}
