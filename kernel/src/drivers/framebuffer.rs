use core::{fmt, mem::MaybeUninit, slice};

use crate::arch::sync::Mutex;

static FRAMEBUFFER: Mutex<MaybeUninit<Framebuffer>> = Mutex::new(MaybeUninit::uninit());

pub struct Framebuffer {
    fb: &'static mut [RgbPixel],
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
}

impl fmt::Debug for Framebuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Framebuffer")
            .field("addr", &self.fb.as_ptr())
            .field("len", &self.fb.len())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pitch", &self.pitch)
            .field("bpp", &self.bpp)
            .finish()
    }
}

impl Framebuffer {
    pub const BACKGROUND: RgbPixel = RgbPixel::from_rgb(0x1d, 0x1f, 0x21);

    pub fn new(slice: &'static mut [u32], width: u64, height: u64, pitch: u64, bpp: u16) -> Self {
        let slice = {
            let ptr = slice.as_mut_ptr() as *mut RgbPixel;
            let len = slice.len();

            // safety: RgbPixel is a newtype wrapper
            unsafe { slice::from_raw_parts_mut(ptr, len) }
        };

        let mut this = Self {
            fb: slice,
            width,
            height,
            pitch,
            bpp,
        };

        this.clear();

        this
    }

    pub fn install(self) {
        FRAMEBUFFER.lock_disabling_interrupts().write(self);
    }

    pub const fn width(&self) -> u64 {
        self.width
    }

    pub const fn height(&self) -> u64 {
        self.height
    }

    pub fn clear(&mut self) {
        self.fb.fill(Self::BACKGROUND);
    }

    pub fn with_handle<T>(mut fun: impl FnMut(&Self) -> T) -> T {
        let this = FRAMEBUFFER.lock_disabling_interrupts();
        let this = unsafe { this.assume_init_ref() };
        fun(this)
    }

    pub fn with_handle_mut<T>(mut fun: impl FnMut(&mut Self) -> T) -> T {
        let mut this = FRAMEBUFFER.lock_disabling_interrupts();
        let this = unsafe { this.assume_init_mut() };
        fun(this)
    }

    pub fn put_pixel_at_point(&mut self, point: Point, color: RgbPixel) {
        let x = (point.x as u64).min(self.width - 1);
        let y = (point.y as u64).min(self.height - 1);
        let target = x + y * self.width;

        self.put_pixel_at_pos(target as usize, color);
    }

    pub fn put_pixel_at_pos(&mut self, pos: usize, color: RgbPixel) {
        self.fb[pos] = color;
    }

    pub fn put_from_point(&mut self, start: Point, pixels: impl Iterator<Item = RgbPixel>) {
        let pos = self.pos(start);
        //todo copy from slice?
        pixels
            .enumerate()
            .for_each(|(offset, pix)| self.put_pixel_at_pos(pos + offset, pix));
    }

    pub fn put_bitmap(&mut self, width: u64, bitmap: &[u8]) {
        let width = width.min(self.width) as usize;

        let mut nw_corner = Point::new(0, 0);

        let pix_lines = bitmap
            .chunks(3 * width)
            .map(|line| line.chunks(3).map(RgbPixel::from_rgb_slice));

        for pix_line in pix_lines {
            if nw_corner.y as u64 == self.height - 1 {
                return;
            }

            nw_corner.y += 1;
            self.put_from_point(nw_corner, pix_line);
        }
    }

    pub fn pos(&self, point: Point) -> usize {
        let x = (point.x as u64).min(self.width - 1);
        let y = (point.y as u64).min(self.height - 1);
        (x + y * self.width) as usize
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct RgbPixel(pub u32);

impl RgbPixel {
    /// Red color
    pub const RED: Self = Self(0x00ff_0000);
    /// Green color
    pub const GREEN: Self = Self(0x0000_ff00);
    /// Blue color
    pub const BLUE: Self = Self(0x0000_00ff);

    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        let color = (red as u32) << 16 | (green as u32) << 8 | blue as u32;

        Self(color)
    }

    pub const fn from_rgb_slice(rgb_slice: &[u8]) -> Self {
        Self::from_rgb(rgb_slice[0], rgb_slice[1], rgb_slice[2])
    }

    pub const fn apply_intensity(self, intensity: u8) -> RgbPixel {
        let mut r = (self.0 >> 16) as u8;
        let mut g = (self.0 >> 8) as u8;
        let mut b = self.0 as u8;

        r = r.saturating_add(intensity);
        g = g.saturating_add(intensity);
        b = b.saturating_add(intensity);

        Self::from_rgb(r, g, b)
    }
}
