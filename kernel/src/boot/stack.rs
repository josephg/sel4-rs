use crate::config::CONFIG_MAX_NUM_NODES;
use crate::racycell::RacyCell;

/// This describes the log2 size of the kernel stack. Great care should be taken as
/// there is no guard below the stack so setting this too small will cause random
/// memory corruption
pub(crate) const KERNEL_STACK_BITS: usize = 12;

#[repr(align(16))]
#[allow(unused)]
pub(crate) struct KernelStack([[u8; 1 << KERNEL_STACK_BITS]; CONFIG_MAX_NUM_NODES]);

pub(crate) static KERNEL_STACK: RacyCell<KernelStack> = RacyCell::new(KernelStack([[0; _]; _]));