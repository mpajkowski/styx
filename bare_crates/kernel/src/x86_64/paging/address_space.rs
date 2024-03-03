use core::arch::asm;

use crate::{
    arch::{phys_to_io, FRAME_SIZE, IO_BASE},
    x86_64::PhysAddr,
};

use super::{offset_table::OffsetTable, page_table::PageTable};

pub struct AddressSpace {
    addr: PhysAddr,
}

impl AddressSpace {
    pub fn active() -> Self {
        Self {
            addr: read_cr3_addr(),
        }
    }

    pub fn top_level_page_table(&mut self) -> &mut PageTable {
        let virt = phys_to_io(self.addr);

        unsafe { &mut *virt.as_mut_ptr::<PageTable>() }
    }

    pub fn offset_table(&mut self) -> OffsetTable {
        let top_level_page_table = self.top_level_page_table();

        unsafe { OffsetTable::new(top_level_page_table, IO_BASE) }
    }
}

pub fn read_cr3_addr() -> PhysAddr {
    let value: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
    }

    PhysAddr::new_aligned::<FRAME_SIZE>(value & 0x_000f_ffff_ffff_f000)
}
