use limine_mini::framebuffer::Response as FramebufferResponse;

use crate::Framebuffer;

pub struct Limine {
    pub framebuffer: &'static FramebufferResponse,
}

impl Limine {
    pub fn gather() -> Self {
        Self {
            framebuffer: req::FRAMEBUFFER.response().unwrap(),
        }
    }

    pub fn framebuffer(&self) -> Framebuffer {
        let fb = self.framebuffer.framebuffers().next().unwrap();

        assert_eq!(
            fb.bpp, 32,
            "invalid bpp: {}, expected 32 for this implementation",
            fb.bpp
        );

        Framebuffer::new(
            unsafe { core::slice::from_raw_parts_mut(fb.addr as *mut u32, fb.size() / 4) },
            fb.width,
            fb.height,
            fb.pitch,
            fb.bpp,
        )
    }
}

mod req {
    pub static FRAMEBUFFER: limine_mini::framebuffer::Request =
        limine_mini::framebuffer::Request::new(0);
}
