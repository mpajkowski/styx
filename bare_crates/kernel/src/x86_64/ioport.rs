#![allow(unused)]

use core::arch::asm;

/// Writes u8 `value` to `port`.
#[inline]
pub unsafe fn write_u8(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

/// Reads u8 value from `port`.
#[inline]
pub unsafe fn read_u8(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

/// Writes u16 `value` to `port`.
#[inline]
pub unsafe fn write_u16(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
}

/// Reads u16 value from `port`.
#[inline]
pub unsafe fn read_u16(port: u16) -> u16 {
    let value: u16;
    asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

/// Writes u32 `value` to `port`.
#[inline]
pub unsafe fn write_u32(port: u32, value: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
}

/// Reads u32 value from `port`.
#[inline]
pub unsafe fn read_u32(port: u32) -> u32 {
    let value: u32;
    asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}
