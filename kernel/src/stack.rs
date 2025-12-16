use crate::config::{CONFIG_KERNEL_STACK_BITS, CONFIG_MAX_NUM_NODES};
use crate::racycell::RacyCell;
use crate::utils::bit_usize;

#[repr(align(16))]
#[allow(unused)]
pub(crate) struct KernelStack([[u8; bit_usize(CONFIG_KERNEL_STACK_BITS)]; CONFIG_MAX_NUM_NODES]);

pub(crate) static KERNEL_STACK: RacyCell<KernelStack> = RacyCell::new(KernelStack([[0; _]; _]));