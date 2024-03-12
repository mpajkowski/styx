use core::{fmt, mem::MaybeUninit, slice};

use crate::arch::sync::Mutex;

static FRAMEBUFFER: Mutex<MaybeUninit<Framebuffer>> = Mutex::new(MaybeUninit::uninit());

pub struct Framebuffer {
    fb: &'static mut [u8],
    width: u64,
    height: u64,
    pitch: u64,
    pixelwidth: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Props {
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub pixelwidth: u64,
}

impl fmt::Debug for Framebuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Framebuffer")
            .field("addr", &self.fb.as_ptr())
            .field("len", &self.fb.len())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pitch", &self.pitch)
            .field("pixelwidth", &self.pixelwidth)
            .finish()
    }
}

impl Framebuffer {
    pub const BACKGROUND: RgbPixel = RgbPixel::from_rgb(0x1d, 0x1f, 0x21);

    pub fn new(
        slice: &'static mut [u8],
        width: u64,
        height: u64,
        pitch: u64,
        pixelwidth: u64,
    ) -> Self {
        let mut this = Self {
            fb: slice,
            width,
            height,
            pitch,
            pixelwidth,
        };

        this.clear();

        this
    }

    pub fn install(self) {
        FRAMEBUFFER.lock_disabling_interrupts().write(self);
    }

    pub const fn props(&self) -> Props {
        Props {
            width: self.width,
            height: self.height,
            pitch: self.pitch,
            pixelwidth: self.pixelwidth,
        }
    }

    pub const fn width(&self) -> u64 {
        self.width
    }

    pub const fn height(&self) -> u64 {
        self.height
    }

    pub fn clear(&mut self) {
        let slice = {
            let ptr = self.fb.as_mut_ptr() as *mut RgbPixel;
            let len = self.fb.len() / self.pixelwidth as usize;

            unsafe { slice::from_raw_parts_mut(ptr, len) }
        };

        slice.fill(Self::BACKGROUND);
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
        let target = (y * self.pitch + x * self.pixelwidth) as usize;

        let color = color.0;

        self.fb[target] = color as u8;
        self.fb[target + 1] = (color >> 8) as u8;
        self.fb[target + 2] = (color >> 16) as u8;
    }

    pub fn put_from_point(&mut self, mut point: Point, pixels: impl Iterator<Item = RgbPixel>) {
        for pix in pixels {
            point.x += 1;
            self.put_pixel_at_point(point, pix)
        }
    }

    pub fn put_bitmap(&mut self, mut point: Point, width: u64, bitmap: &[u8]) {
        let width = width.min(self.width) as usize;

        let pix_lines = bitmap
            .chunks(3 * width)
            .map(|line| line.chunks(3).map(RgbPixel::from_rgb_slice));

        for pix_line in pix_lines {
            if point.y as u64 == self.height - 1 {
                return;
            }

            point.y += 1;
            self.put_from_point(point, pix_line);
        }
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
