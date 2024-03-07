use crate::file::File;
use crate::make_struct;

#[repr(C)]
#[derive(Debug)]
pub struct Response {
    revision: u64,
    module_count: u64,
    module: *const *const File,
}

impl Response {
    pub fn modules(&self) -> impl Iterator<Item = &File> {
        crate::utils::iter(self.module, self.module_count)
    }
}

make_struct!(
    struct Request: [0x3e7e279702be32af, 0xca1c4f3bd1280cee] => Response {});
