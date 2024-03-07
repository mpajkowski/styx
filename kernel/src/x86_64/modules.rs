use alloc::collections::btree_map::BTreeMap;

pub struct Modules {
    modules: BTreeMap<&'static str, &'static [u8]>,
}

impl Modules {
    pub fn from_boot_info(modules: impl Iterator<Item = &'static limine_mini::file::File>) -> Self {
        let modules = modules
            .filter_map(|f| {
                let Some(path) = f.path().and_then(|path| core::str::from_utf8(path).ok()) else {
                    log::info!("path not present");
                    return None;
                };

                let bytes = f.as_slice();

                log::info!("adding module with path {path}");

                Some((path, bytes))
            })
            .collect();

        Self { modules }
    }

    pub fn by_path(&self, path: &str) -> Option<&'static [u8]> {
        self.modules.get(path).copied()
    }
}
