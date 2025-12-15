use core::arch::asm;
use core::ffi::c_void;
use ufmt::derive::uDebug;
use crate::boot::multiboot::MULTIBOOT_BOOTLOADER_MAGIC;
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


// #[derive(uDebug)]
// struct BootInfo {
//
// }

// This is called from entry_64 in boot0.
#[unsafe(link_section = ".boot.text")]
#[unsafe(no_mangle)]
pub extern "C" fn boot_sys(multiboot_magic: u32, _mbi: *mut c_void) -> ! {
    // init_serial is called once at the start of the boot process before we use the serial console.
    // This is used for debug messages.
    unsafe { init_serial() };

    // In SeL4, the root process is compiled to an ELF module and passed to the kernel as a
    // multiboot module. This is very convenient during development, because you can compile it
    // directly to an elf file and just pass it through. And we should be able to set up debugging
    // the normal way too.
    //
    // For this kernel I'd like to have some alternate ways of building the kernel with an embedded
    // root process. But I want this implementation to be compatible, so for now I'll stick to
    // sel4's behaviour here.

    // Multiboot 1 and 2 both pass modules to the kernel slightly differently.

    if multiboot_magic == MULTIBOOT_BOOTLOADER_MAGIC {
        kprintln!("Booting via multiboot v1");
    }

    try_boot_sys().unwrap();

    unsafe { asm!("hlt"); }
    loop {
    }
}
