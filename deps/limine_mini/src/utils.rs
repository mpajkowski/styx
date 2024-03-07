pub fn iter<'a, T: 'a>(ptr: *const *const T, len: u64) -> impl Iterator<Item = &'a T> + Clone {
    unsafe {
        core::slice::from_raw_parts(ptr, len as usize)
            .iter()
            .map(|x| &**x)
    }
}
