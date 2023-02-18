#![no_std]

mod bitmap;

pub use bitmap::BitmapAlloc;

/// Allocates physical frames
///
/// FRAME_SIZE has to be the power of two
pub trait FrameAlloc<const FRAME_SIZE: usize, A: PhysAddr> {
    // Returns physical address of physical allocation.
    // Address is guaranteed to by `FRAME_SIZE` aligned
    fn alloc(&mut self, size: usize) -> Option<A>;
    fn free(&mut self, phys_pointer: A);
}

pub trait PhysAddr: From<u64> {}
