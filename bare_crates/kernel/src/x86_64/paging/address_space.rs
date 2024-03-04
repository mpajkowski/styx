use crate::{
    arch::{addr::IO_BASE, registers::Cr3},
    x86_64::PhysAddr,
};

use super::{offset_table::OffsetTable, page_table::PageTable};

pub struct AddressSpace {
    addr: PhysAddr,
}

impl AddressSpace {
    pub fn active() -> Self {
        Self {
            addr: Cr3::read().phys_addr(),
        }
    }

    pub fn top_level_page_table(&mut self) -> &mut PageTable {
        unsafe { &mut *self.addr.to_io().as_mut_ptr::<PageTable>() }
    }

    pub fn offset_table(&mut self) -> OffsetTable {
        let top_level_page_table = self.top_level_page_table();

        unsafe { OffsetTable::new(top_level_page_table, IO_BASE) }
    }
}
