pub mod fixedarr;
mod panic;

use core::arch::asm;

/// Halt execution immediately.
pub fn halt() -> ! {
    unsafe { asm!("hlt"); }
    loop {}
}

// This is a bit gross, but its also kinda fine. We mostly just deal in usizes here anyway.
// pub const fn bit_usize(n: u32) -> usize { 1 << n }
// pub const fn bit_u32(n: u32) -> u32 { 1 << n }
// pub const fn bit_u64(n: u32) -> u64 { 1 << n }

pub trait NumUtils {
    fn bit(n: u32) -> Self;
    fn round_down(self, b: u32) -> Self;
    fn round_up(self, b: u32) -> Self;
}

impl NumUtils for usize {
    fn bit(n: u32) -> Self { 1 << n }

    fn round_down(self, b: u32) -> Self {
        (self >> b) << b
    }

    fn round_up(self, b: u32) -> Self {
        (((self - 1) >> b) + 1) << b
    }
}

impl NumUtils for u64 {
    fn bit(n: u32) -> Self { 1 << n }

    fn round_down(self, b: u32) -> Self {
        (self >> b) << b
    }

    fn round_up(self, b: u32) -> Self {
        (((self - 1) >> b) + 1) << b
    }
}

impl NumUtils for u32 {
    fn bit(n: u32) -> Self { 1 << n }

    fn round_down(self, b: u32) -> Self {
        (self >> b) << b
    }

    fn round_up(self, b: u32) -> Self {
        (((self - 1) >> b) + 1) << b
    }
}
