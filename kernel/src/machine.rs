//! Based on include/machine.h

use crate::arch::hardware::KERNEL_ELF_BASE;
use crate::basic_types::{Paddr, Pptr};
use crate::hardware::{KERNEL_ELF_BASE_OFFSET, KERNEL_ELF_TOP};

// const fn addr_from_kpptr(ptr: Pptr) -> Paddr {
//     assert!(ptr >= KERNEL_ELF_BASE);
//     assert!(ptr <= KERNEL_ELF_TOP);
//     ptr - KERNEL_ELF_BASE_OFFSET
// }