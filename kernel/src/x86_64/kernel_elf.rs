use super::limine::Limine;

pub fn from_boot_info(boot_info: &Limine) {
    let slice = boot_info.kernel.file().as_slice();
    crate::kernel_elf::store(slice).expect("failed to load kernel");
}
