#[repr(C)]
#[derive(Debug)]
pub struct Framebuffer {
    address: *const u8,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
    memory_model: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
    reserved: [u8; 7],
    edid_size: u64,
    edid: *const u8,
}

impl Framebuffer {
    /// Returns the size of the framebuffer.
    pub fn size(&self) -> usize {
        self.pitch as usize * self.height as usize * (self.bpp as usize / 8)
    }
}
