use core::arch::asm;
use core::ffi::c_void;
use crate::boot::multiboot::MULTIBOOT_INFO_MAGIC;
use crate::console::init_serial;
use crate::kprintln;
use crate::racycell::RacyCell;

const MAX_NUM_NODES: usize = 32;
/// This describes the log2 size of the kernel stack. Great care should be taken as
/// there is no guard below the stack so setting this too small will cause random
/// memory corruption
pub(crate) const KERNEL_STACK_BITS: usize = 12;

#[repr(align(16))]
#[allow(unused)]
pub(crate) struct KernelStack([[u8; 1 << KERNEL_STACK_BITS]; MAX_NUM_NODES]);

pub(crate) static KERNEL_STACK: RacyCell<KernelStack> = RacyCell::new(KernelStack([[0; _]; _]));

#[unsafe(link_section = ".boot.text")]
fn try_boot_sys() -> Result<(), ()> {

    Ok(())
}

// This is called from entry_64 in boot0.
#[unsafe(link_section = ".boot.text")]
#[unsafe(no_mangle)]
pub extern "C" fn boot_sys(multiboot_magic: u32, _mbi: *mut c_void) -> ! {
    unsafe { init_serial() };

    if multiboot_magic == MULTIBOOT_INFO_MAGIC {
        kprintln!("Booting via multiboot v1");
    }

    try_boot_sys().unwrap();

    unsafe { asm!("hlt"); }
    loop {
    }
}
