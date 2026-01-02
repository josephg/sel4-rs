//! This file contains SeL4's configuration time parameters
//! Right now these values are all hard coded, but we could totally take them at compile time with
//! some work.
//!
//! Note I don't intend to ever implement the full set of features here that sel4 supports. Mostly,
//! I'm not interested in any features that are only needed or used on legacy chipsets. For example,
//! you can't configure this port to use PIC (it must use APIC). And you can't disable IOMMU.

use crate::const_assert;

/// Max number of CPU cores to boot.
///
/// TODO: In SeL4 this defaults to 1, which seems insufficient.
#[cfg(feature = "smp")]
pub const CONFIG_MAX_NUM_NODES: usize = 32;
#[cfg(not(feature = "smp"))]
pub const CONFIG_MAX_NUM_NODES: usize = 1;

/// Configure the maximum number of IOAPIC controllers that can be supported. SeL4
/// will detect IOAPICs regardless of whether the IOAPIC will actually be used as
/// the final IRQ controller.
pub const CONFIG_MAX_NUM_IOAPIC: usize = 1;

/// This describes the log2 size of the kernel stack. Great care should be taken as
/// there is no guard below the stack so setting this too small will cause random
/// memory corruption
pub(crate) const CONFIG_KERNEL_STACK_BITS: u32 = 12;

pub(crate) enum ConfigGraphicsMode {
    None,
    Text,
    Linear,
}

/// The type of graphics mode to request from the boot loader. This is encoded into the
/// multiboot header and is merely a hint, the boot loader is free to ignore or set some
/// other mode.
pub(crate) const CONFIG_MULTIBOOT_GRAPHICS_MODE: ConfigGraphicsMode = ConfigGraphicsMode::None;

/// Prevent against the Meltdown vulnerability by using a reduced Static Kernel
/// Image and Micro-state window instead of having all kernel state in the kernel window.
/// This only needs to be enabled if deploying to a vulnerable processor.
pub(crate) const CONFIG_KERNEL_SKIM_WINDOW: bool = false;

/// IOMMU support for VT-d enabled chipsets. This is an intel-only feature. AMD chipsets also
/// support IOMMU but use a different IOMMU technology.
/// TODO: This is currently unsupported, since I only have an AMD chipset to test with.
pub(crate) const CONFIG_IOMMU: bool = false;


const_assert!(CONFIG_KERNEL_SKIM_WINDOW == false, "SKIM window not implemented.");
