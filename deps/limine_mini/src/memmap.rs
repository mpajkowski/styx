use core::fmt::Debug;

use crate::make_struct;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EntryKind {
    Usable = 0,
    Reserved = 1,
    AcpiReclaimable = 2,
    AcpiNvs = 3,
    BadMemory = 4,
    BootloaderReclaimable = 5,
    /// The kernel and modules loaded are not marked as usable memory. They are
    /// marked as Kernel/Modules. The entries are guaranteed to be sorted by base
    /// address, lowest to highest. Usable and bootloader reclaimable entries are
    /// guaranteed to be 4096 byte aligned for both base and length. Usable and
    /// bootloader reclaimable entries are guaranteed not to overlap with any
    /// other entry. To the contrary, all non-usable entries (including kernel/modules)
    /// are not guaranteed any alignment, nor is it guaranteed that they do not
    /// overlap other entries.
    KernelAndModules = 6,
    Framebuffer = 7,
}

#[repr(C)]
pub struct Entry {
    pub base: u64,
    pub len: u64,
    pub kind: EntryKind,
}

impl Debug for Entry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Entry")
            .field("base", &(self.base as *const u8))
            .field("len", &(self.len / 0x1000))
            .field("kind", &self.kind)
            .finish()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Response {
    pub revision: u64,
    /// How many memory map entries are present.
    pub entry_count: u64,
    /// Pointer to an array of `entry_count` pointers to struct [`LimineMemmapEntry`] structures.
    pub entries: *const *const Entry,
}

impl Response {
    pub fn entries(&self) -> impl Iterator<Item = &Entry> + Clone {
        crate::utils::iter(self.entries, self.entry_count)
    }
}

make_struct!(
    struct Request: [0x67cf3d9d378a806f, 0xe304acdfc50c3c62] => Response {}
);
