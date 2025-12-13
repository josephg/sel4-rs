#![no_std]
#![no_main]

mod multiboot;
mod boot;
mod racycell;
mod console;

use crate::console::init_serial;
use crate::racycell::RacyCell;
use core::arch::asm;
use core::ffi::c_void;
use core::panic::PanicInfo;
// use no_panic::no_panic;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


const MAX_NUM_NODES: usize = 32;

/// This describes the log2 size of the kernel stack. Great care should be taken as
/// there is no guard below the stack so setting this too small will cause random
/// memory corruption
pub(crate) const KERNEL_STACK_BITS: usize = 12;

#[repr(align(16))]
#[allow(unused)]
struct KernelStack([[u8; 1 << KERNEL_STACK_BITS]; MAX_NUM_NODES]);

pub(crate) static KERNEL_STACK: RacyCell<KernelStack> = RacyCell::new(KernelStack([[0; _]; _]));

#[unsafe(link_section = ".boot.text")]
#[unsafe(no_mangle)]
pub(crate) extern "C" fn boot_sys(_multiboot_magic: u32, _mbi: *mut c_void) -> ! {
    unsafe { init_serial() };


    kprintln!("Hi from the kernel! {}", "oooohhhhh");

    // write!(port, "Multiboot {:x}\n", multiboot_magic).unwrap();

    panic!("oh nooo");

    // port.write_str("hi from boot_sys!\n").unwrap();
    unsafe { asm!("hlt"); }
    loop {
    }
}




// global_asm!(r#"
//   .section .phys.text
//   .code32
//   .globl _start
//   .align 4
//         mov edi, eax
//         mov esi, ebx
//
// _start:
// "#);
