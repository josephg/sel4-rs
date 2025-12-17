//! This file implements the needed code in src/plat/pc99/machine/pic.c.
//!
//! DEPARTURE: We don't support using the traditional PIC. This implementation requires the newer
//! IOAPIC.

use core::arch::asm;
use crate::arch::x86_64::asm::out8;

// PIC (i8259) base registers
const PIC1_BASE: u16 = 0x20;
const PIC2_BASE: u16 = 0xa0;

/// Program PIC (i8259) to remap IRQs 0-15 to interrupt vectors starting at 'interrupt'
#[unsafe(link_section = ".boot.text")]
pub fn pic_remap_irqs(interrupt: u8) {
    unsafe {
        out8(PIC1_BASE, 0x11);
        out8(PIC2_BASE, 0x11);
        out8(PIC1_BASE + 1, interrupt);
        out8(PIC2_BASE + 1, interrupt + 8);
        out8(PIC1_BASE + 1, 0x04);
        out8(PIC2_BASE + 1, 0x02);
        out8(PIC1_BASE + 1, 0x01);
        out8(PIC2_BASE + 1, 0x01);
        out8(PIC1_BASE + 1, 0x0);
        out8(PIC2_BASE + 1, 0x0);
    }
}

#[unsafe(link_section = ".boot.text")]
pub unsafe fn pic_disable() {
    // We assume that pic_remap_irqs has already been called and
    // just mask all the irqs */
    unsafe {
        out8(PIC1_BASE + 1, 0xff);
        out8(PIC2_BASE + 1, 0xff);
    }
}
