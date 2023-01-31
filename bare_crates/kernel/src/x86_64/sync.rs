use core::{
    arch::asm,
    ops::{Deref, DerefMut},
};

pub fn hlt() {
    unsafe { asm!("hlt") }
}

pub fn pause() {
    unsafe { asm!("pause") }
}

/// Disables interrupts using `cli` instruction
pub fn disable_interrupts() {
    unsafe {
        asm!("cli");
    }
}

/// Enables interrupts using `sti` instruction
pub fn enable_interrupts() {
    unsafe {
        asm!("sti");
    }
}

/// Checks if interrupts are enabled by reading value of interrupt flag from `RFLAGS` register
pub fn are_interrupts_enabled() -> bool {
    Rflags::load().contains(Rflags::INTERRUPT_ENABLE)
}

/// Calls given closure with disabled interrupts
pub fn without_interrupts<F, R>(call: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = WithoutInterruptsGuard::enter();
    call()
}

/// Disables interrupts for the length of its lifetime
///
/// Restores previous interrupt state on `drop`
pub struct WithoutInterruptsGuard {
    were_enabled: bool,
}

impl WithoutInterruptsGuard {
    pub fn enter() -> Self {
        let are_enabled = are_interrupts_enabled();

        if are_enabled {
            disable_interrupts();
        }

        Self {
            were_enabled: are_enabled,
        }
    }

    pub fn were_enabled(&self) -> bool {
        self.were_enabled
    }
}

impl Drop for WithoutInterruptsGuard {
    fn drop(&mut self) {
        if self.were_enabled {
            enable_interrupts();
        }
    }
}

pub struct MutexGuard<'a, T> {
    guard: spin::MutexGuard<'a, T>,
    _without_interrupts: Option<WithoutInterruptsGuard>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

pub struct Mutex<T> {
    inner: spin::Mutex<T>,
}

impl<T> Mutex<T> {
    #[inline(always)]
    pub const fn new(val: T) -> Self {
        Self {
            inner: spin::Mutex::new(val),
        }
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        MutexGuard {
            guard: self.inner.lock(),
            _without_interrupts: None,
        }
    }

    pub fn lock_disabling_interrupts(&self) -> MutexGuard<'_, T> {
        MutexGuard {
            guard: self.inner.lock(),
            _without_interrupts: Some(WithoutInterruptsGuard::enter()),
        }
    }
}

bitflags::bitflags! {
    pub struct Rflags: u64 {
        const CARRY = 0x0001;
        const RESERVED_1 = 0x0002;
        const PARITY = 0x0004;
        const ADJUST = 0x0010;
        const ZERO = 0x0040;
        const SIGN = 0x0080;
        const TRAP = 0x0100;
        const INTERRUPT_ENABLE = 0x0200;
        const DIRECTION = 0x0400;
        const OVERFLOW = 0x0800;
        const IOPL = 0x3000;
        const NESTED_TASK = 0x3000;
        const RESUME = 0x0001_0000;
        const VIRTUAL8086 = 0x0002_0000;
        const ALIGNMENT_CHECK = 0x0004_0000;
        const VIRTUAL_INTERRUPT = 0x0008_0000;
        const VIRTUAL_INTERRUPT_PENDING = 0x0010_0000;
        const CPUID_AVAIL = 0x0020_0000;
    }
}

impl Rflags {
    pub fn load() -> Self {
        let rflags: u64;
        unsafe { asm!("pushfq; pop {}", out(reg) rflags, options(preserves_flags)) };
        Self::from_bits_truncate(rflags)
    }
}
