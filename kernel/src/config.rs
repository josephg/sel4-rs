//! This file contains SeL4's configuration time parameters
//! Right now these values are all hard coded, but we could totally take them at compile time with
//! some work.


/// Max number of CPU cores to boot.
///
/// TODO: In SeL4 this defaults to 1, which seems insufficient.
pub const CONFIG_MAX_NUM_NODES: usize = 32;

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