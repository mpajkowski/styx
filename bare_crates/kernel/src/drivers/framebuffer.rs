use core::{fmt, mem::MaybeUninit};

use crate::arch::sync::Mutex;

static FRAMEBUFFER: Mutex<MaybeUninit<Framebuffer>> = Mutex::new(MaybeUninit::uninit());

pub struct Framebuffer {
    slice: &'static mut [u32],
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
}

impl fmt::Debug for Framebuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Framebuffer")
            .field("addr", &self.slice.as_ptr())
            .field("len", &self.slice.len())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pitch", &self.pitch)
            .field("bpp", &self.bpp)
            .finish()
    }
}

impl Framebuffer {
    pub const BACKGROUND: Color = Color::from_rgb(0x1d, 0x1f, 0x21);

    pub fn new(slice: &'static mut [u32], width: u64, height: u64, pitch: u64, bpp: u16) -> Self {
        let mut this = Self {
            slice,
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
        self.slice.fill(Self::BACKGROUND.0);
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

    pub fn put_pixel_at_point(&mut self, point: Point, color: Color) {
        let x = (point.x as u64).min(self.width - 1);
        let y = (point.y as u64).min(self.height - 1);
        let target = x + y * self.width;

        self.put_pixel_at_pos(target as usize, color);
    }

    pub fn put_pixel_at_pos(&mut self, pos: usize, color: Color) {
        self.slice[pos] = color.0;
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
pub struct Color(pub u32);

impl Color {
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

    pub const fn apply_intensity(self, intensity: u8) -> Color {
        let mut r = (self.0 >> 16) as u8;
        let mut g = (self.0 >> 8) as u8;
        let mut b = self.0 as u8;

        r = r.saturating_add(intensity);
        g = g.saturating_add(intensity);
        b = b.saturating_add(intensity);

        Self::from_rgb(r, g, b)
    }
}
