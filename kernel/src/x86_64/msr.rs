use core::arch::asm;

/// Reads value from MSR
///
/// Uses `rdmsr` instruction.
///
/// # Safety
///
/// Caller must provide valid MSR
pub unsafe fn rdmsr(msr: u32) -> u64 {
    let hi: u32;
    let lo: u32;

    asm!("rdmsr", in("ecx") msr, out("edx") hi, out("eax") lo, options(preserves_flags));

    (hi as u64) << 32 | lo as u64
}

/// Writes value to MSR
///
/// Uses `wrmsr` instruction.
///
/// # Safety
///
/// Caller must provide valid MSR and try to not write anything destructible
pub unsafe fn wrmsr(msr: u32, value: u64) {
    let hi = (value >> 32) as u32;
    let lo = value as u32;

    asm!("wrmsr", in("ecx") msr, in("edx") hi, in("eax") lo, options(preserves_flags));
}
