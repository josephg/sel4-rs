use core::arch::asm;

pub unsafe fn out8(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al",
            in("dx") port,
            in("al") value,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub unsafe fn out16(port: u16, value: u16) {
    unsafe {
        asm!("out dx, ax",
            in("dx") port,
            in("ax") value,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub unsafe fn out32(port: u16, value: u32) {
    unsafe {
        asm!("out dx, eax",
            in("dx") port,
            in("eax") value,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub unsafe fn in8(port: u16) -> u8 {
    let value: u8;

    unsafe {
        asm!("in al, dx",
            in("dx") port,
            lateout("al") value,
            options(nostack, preserves_flags),
        );
    }
    value
}

#[inline(always)]
pub unsafe fn in16(port: u16) -> u16 {
    let value: u16;
    unsafe {
        asm!("in ax, dx",
            in("dx") port,
            lateout("ax") value,
            options(nostack, preserves_flags),
        );
    }
    value
}

#[inline(always)]
pub unsafe fn in32(port: u16) -> u32 {
    let value: u32;
    unsafe {
        asm!("in eax, dx",
            in("dx") port,
            lateout("eax") value,
            options(nostack, preserves_flags),
        );
    }
    value
}
