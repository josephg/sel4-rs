//! From include/plat/pc99/plat/machine/devices.h.
//!
//! I'm not sure why this is different from hardware or machine. I might combine all these files
//! in a future version.

use crate::arch::constants::PAGE_BITS;
use crate::arch::hardware::KDEV_BASE;
use crate::config::CONFIG_MAX_NUM_IOAPIC;
use crate::const_assert;
use crate::utils::bit_usize;

// pub const PPTR_APIC: usize = KDEV_BASE;
//
// pub const PPTR_IOAPIC_START: usize = PPTR_APIC + bit_usize(PAGE_BITS);
// pub const PPTR_DRHU_START: usize = PPTR_IOAPIC_START + bit_usize(PAGE_BITS) * CONFIG_MAX_NUM_IOAPIC;

// pub const MAX_NUM_DRHU: usize = PPTR_DRHU_START.wrapping_neg() >> PAGE_BITS;

/// Most hardware has just 1-3 iommus. This should be pretty generous in practice.
// DEPARTURE: SeL4 just allows as many as would fit in memory. But because I'm putting BootState on
// the stack, doing that blows out my stack space and I get a triple fault.
pub const MAX_NUM_DRHU: usize = 8;