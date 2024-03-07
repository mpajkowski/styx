pub use limine_mini::framebuffer::Response as FramebufferResponse;
pub use limine_mini::kernel::Response as KernelResponse;
pub use limine_mini::memmap::Response as MemmapResponse;
pub use limine_mini::module::Response as ModuleResponse;
pub use limine_mini::rsdp::Response as RsdpResponse;

use crate::Framebuffer;

pub struct Limine {
    pub framebuffer: &'static FramebufferResponse,
    pub memmap: &'static MemmapResponse,
    pub rsdp: &'static RsdpResponse,
    pub kernel: &'static KernelResponse,
    pub module: &'static ModuleResponse,
}

impl Limine {
    pub fn gather() -> Self {
        Self {
            framebuffer: req::FRAMEBUFFER.response().unwrap(),
            memmap: req::MEMMAP.response().unwrap(),
            rsdp: req::RSDP.response().unwrap(),
            kernel: req::KERNEL.response().unwrap(),
            module: req::MODULES.response().unwrap(),
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

    pub static MEMMAP: limine_mini::memmap::Request = limine_mini::memmap::Request::new(0);

    pub static RSDP: limine_mini::rsdp::Request = limine_mini::rsdp::Request::new(0);

    pub static KERNEL: limine_mini::kernel::Request = limine_mini::kernel::Request::new(0);

    pub static MODULES: limine_mini::module::Request = limine_mini::module::Request::new(0);
}
