//! From src/kernel/boot.c

use crate::arch::hardware::KERNEL_ELF_PADDR_BASE;
use crate::basic_types::{Paddr, PhysRegion};
use crate::hardware::{KERNEL_ELF_BASE_OFFSET, KERNEL_ELF_TOP};

/// Returns the physical region of the kernel image.
#[unsafe(link_section = ".boot.text")]
pub fn get_p_reg_kernel_img() -> PhysRegion {

    // In C this expression is defined like this:
    //     .start = kpptr_to_paddr((const void *)KERNEL_ELF_BASE),
    //     .end   = kpptr_to_paddr((const void *)KERNEL_ELF_TOP)

    // If I move to Newtype for Paddr / Ppptr, it might make sense to tack closer to that code.

    PhysRegion {
        // The original expression expands to this:
        start: KERNEL_ELF_PADDR_BASE,
        end: (KERNEL_ELF_TOP as Paddr) - KERNEL_ELF_BASE_OFFSET,
    }
}
