use rustc_demangle::Demangle;
use spin::Once;
use xmas_elf::{
    sections::{SectionData, ShType},
    symbol_table::{Entry, Entry64},
    ElfFile,
};

pub type KernelElfRef = &'static KernelElf;

static KERNEL_DATA: Once<KernelElf> = Once::new();

pub fn store(kernel_data: &'static [u8]) -> Result<(), &'static str> {
    let elf = ElfFile::new(kernel_data)?;

    KERNEL_DATA.call_once(|| KernelElf(elf));

    Ok(())
}

pub fn get() -> Option<KernelElfRef> {
    KERNEL_DATA.get()
}

pub struct KernelElf(ElfFile<'static>);

impl KernelElf {
    pub fn symtable(&self) -> Option<&[Entry64]> {
        self.0.section_iter().find_map(|section| {
            let data = match section.get_type() {
                Ok(ShType::SymTab) => section.get_data(&self.0),
                _ => return None,
            };

            match data {
                Ok(SectionData::SymbolTable64(symbol_table)) => Some(symbol_table),
                _ => None,
            }
        })
    }

    pub fn symbol_name_at_addr<'a>(
        &'a self,
        symtable: &'a [Entry64],
        addr: u64,
    ) -> Option<Demangle<'a>> {
        for data in symtable.iter() {
            let start = data.value();
            let end = start + data.size();

            if (start..end).contains(&addr) {
                return data.get_name(&self.0).ok().map(rustc_demangle::demangle);
            }
        }

        None
    }
}
