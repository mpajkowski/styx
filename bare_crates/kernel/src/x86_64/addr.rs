use core::{fmt::Debug, ops::Add};

use easybit::{align_down, align_up, BitManipulate};

/// Represents physical address
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// Creates new physical address aligned at desired boundary
    ///
    /// TODO make static assert?
    pub const fn new_aligned<const A: u64>(addr: u64) -> Self {
        Self(align_down!(addr, A))
    }

    /// Creates unchecked physical address
    pub const unsafe fn new_unchecked(addr: u64) -> Self {
        Self(addr)
    }

    /// Checks whether address is aligned
    pub const fn is_aligned(self, align: u64) -> bool {
        align_down!(self.0, align) == self.0
    }

    /// Aligns address down
    pub const fn align_down(self, align: u64) -> Self {
        Self(align_down!(self.0, align))
    }

    /// Convert address to `u64`
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl Add<u64> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl core::fmt::Pointer for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Pointer::fmt(&(self.0 as *const u64), f)
    }
}

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PhysAddr")
            .field(&(self.0 as *const u64))
            .finish()
    }
}

/// Contains invalid virtual address
#[derive(Debug)]
#[repr(transparent)]
pub struct VirtAddrInvalid(u64);

impl From<u64> for VirtAddrInvalid {
    fn from(addr: u64) -> Self {
        Self(addr)
    }
}

impl core::fmt::Pointer for VirtAddrInvalid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Pointer::fmt(&(self.0 as *const u64), f)
    }
}

impl core::fmt::Display for VirtAddrInvalid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <VirtAddrInvalid as core::fmt::Pointer>::fmt(&self, f)
    }
}

/// Represents virtual address
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VirtAddr")
            .field(&(self.0 as *const u64))
            .finish()
    }
}

impl VirtAddr {
    /// Creates new virtual address
    ///
    /// Panics if given input cannot represent cannonical address
    pub fn new(addr: u64) -> Self {
        Self::try_new(addr).expect("bits 48-64 are not empty")
    }

    /// Creates new virtual address
    pub fn try_new(addr: u64) -> Result<Self, VirtAddrInvalid> {
        match addr.read_range(47..64) {
            0 | 0x1ffff => Ok(VirtAddr(addr)),
            1 => Ok(VirtAddr::new_truncate(addr)),
            _ => Err(VirtAddrInvalid(addr)),
        }
    }

    /// Creates new virtual address
    ///
    /// Performs sign extending if needed
    pub const fn new_truncate(addr: u64) -> Self {
        VirtAddr(((addr << 16) as i64 >> 16) as u64)
    }

    /// Creates new unchecked address
    ///
    /// ## Safety
    ///
    /// Caller must guarantee that bits 64..48 are 0 or set to 1 if sign extension is used
    pub const unsafe fn new_unchecked(addr: u64) -> Self {
        Self(addr)
    }

    /// Creates NULL virtual address
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Returns `Some(self)` if not null, `None` otherwise
    pub fn nonzero(self) -> Option<Self> {
        (self != VirtAddr::zero()).then(|| self)
    }

    /// Tests for NULL
    #[inline(always)]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Checks whether address is aligned at given boundary
    pub const fn is_aligned(self, align: u64) -> bool {
        align_down!(self.0, align) == self.0
    }

    /// Aligns up to given boundary
    pub const fn align_up(self, align: u64) -> Self {
        Self(align_up!(self.0, align))
    }

    /// Aligns down to given boundary
    pub const fn align_down(self, align: u64) -> Self {
        Self(align_down!(self.0, align))
    }

    /// Returns P1 table index (aka Table)
    pub const fn p1_index(self) -> usize {
        ((self.0 >> 12) as u16 % 512) as usize
    }

    /// Returns P2 table index (aka Directory)
    pub const fn p2_index(self) -> usize {
        ((self.0 >> 12 >> 9) as u16 % 512) as usize
    }

    /// Returns P3 table index (aka Directory Pointer)
    pub const fn p3_index(self) -> usize {
        ((self.0 >> 12 >> 9 >> 9) as u16 % 512) as usize
    }

    /// Returns P4 table index (aka PML4)
    pub const fn p4_index(self) -> usize {
        ((self.0 >> 12 >> 9 >> 9 >> 9) as u16 % 512) as usize
    }

    /// Returns address offset (last 12 bits)
    pub const fn offset(self) -> u16 {
        (self.0 as u16) % (1 << 12)
    }

    /// Convert address to `u64`
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Convert address to ptr
    pub const fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    /// Convert address to mut ptr
    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
}

impl core::fmt::Pointer for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Pointer::fmt(&(self.0 as *const u64), f)
    }
}

impl Add<u64> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}
