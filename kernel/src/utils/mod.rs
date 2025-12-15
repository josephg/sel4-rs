pub mod fixedarr;
mod panic;

use core::arch::asm;

/// Halt execution immediately.
pub fn halt() -> ! {
    unsafe { asm!("hlt"); }
    loop {}
}

