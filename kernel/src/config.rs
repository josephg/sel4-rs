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