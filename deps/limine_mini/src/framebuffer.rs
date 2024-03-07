#[repr(C)]
#[derive(Debug)]
pub struct Framebuffer {
    pub addr: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    reserved: [u8; 7],
    pub edid_size: u64,
    pub edid: *const u8,
    pub mode_count: u64,
    pub modes: *const *const VideoMode,
}

#[repr(C)]
#[derive(Debug)]
pub struct VideoMode {
    pub pitch: u64,
    pub width: u64,
    pub height: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
}

impl Framebuffer {
    /// Returns the size of the framebuffer.
    pub fn size(&self) -> usize {
        self.pitch as usize * self.height as usize * (self.bpp as usize / 8)
    }

    pub fn modes(&self) -> impl Iterator<Item = &VideoMode> {
        crate::utils::iter(self.modes, self.mode_count)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Response {
    revision: u64,
    framebuffer_count: u64,
    framebuffers: *const *const Framebuffer,
}

impl Response {
    pub fn framebuffers(&self) -> impl Iterator<Item = &Framebuffer> {
        crate::utils::iter(self.framebuffers, self.framebuffer_count)
    }
}

crate::make_struct!(
    /// Omitting this request will cause the bootloader to not initialise the framebuffer
     struct Request: [ 0x9d5827dcd881dd75, 0xa3148604f6fab11b] => Response {}
);
