use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};

use crate::Framebuffer;

use super::framebuffer::Point;

pub struct Terminal {
    chars_in_line: u64,
    lines: u64,
    current_char: u64,
    current_line: u64,
    offset_x: u64,
    offset_y: u64,
}

impl Terminal {
    const RASTER_WIDTH: u64 = get_raster_width(FontWeight::Regular, RasterHeight::Size24) as u64;
    const RASTER_HEIGHT: u64 = RasterHeight::Size24.val() as u64;

    pub fn new(width: u64, height: u64) -> Self {
        Self {
            chars_in_line: width / Self::RASTER_WIDTH,
            lines: height / Self::RASTER_HEIGHT,
            current_char: 0,
            current_line: 0,
            offset_x: 0,
            offset_y: 0,
        }
    }

    pub fn lines(&self) -> u64 {
        self.lines
    }

    pub fn chars_in_line(&self) -> u64 {
        self.chars_in_line
    }

    pub fn new_line(&mut self, fb: &mut Framebuffer) {
        self.current_char = 0;
        self.offset_x = 0;
        self.current_line = if self.current_line < self.lines - 1 {
            self.offset_y += Self::RASTER_HEIGHT;
            self.current_line + 1
        } else {
            self.offset_y = 0;
            fb.clear();
            0
        };
    }

    pub fn put_char(&mut self, fb: &mut Framebuffer, ch: char) {
        let ch = match ch as u8 {
            b'\n' => return self.new_line(fb),
            ch if ch.is_ascii_whitespace() => ' ',
            ch @ 0x20..=0x7e => ch as char,
            _ => ' ',
        };

        if self.current_char == self.chars_in_line {
            self.new_line(fb);
        }

        let raster = get_raster(ch, FontWeight::Regular, RasterHeight::Size24)
            .expect("char sanitization failed")
            .raster();

        for (y, line) in raster.iter().enumerate() {
            for (x, intensity) in line.iter().enumerate() {
                let point = Point::new(
                    (self.offset_x + x as u64) as u16,
                    (self.offset_y + y as u64) as u16,
                );

                let color = Framebuffer::BACKGROUND.apply_intensity(*intensity);

                fb.put_pixel_at_point(point, color);
            }
        }

        self.current_char += 1;
        self.offset_x += Self::RASTER_WIDTH;
    }
}

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Framebuffer::with_handle_mut(|fb| {
            for ch in s.chars() {
                self.put_char(fb, ch);
            }
        });

        Ok(())
    }
}
