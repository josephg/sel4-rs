#![no_std]
#![no_main]

mod multiboot;
mod boot;
mod racycell;
mod serial;

use core::arch::asm;
use core::ffi::c_void;
use core::fmt::Write;
use core::panic::PanicInfo;
use crate::racycell::RacyCell;
use crate::serial::SerialPort;
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
pub(crate) extern "C" fn boot_sys(multiboot_magic: u32, mbi: *mut c_void) -> ! {
    let mut port = unsafe { SerialPort::init() };

    port.write_str("hi from boot_sys!\n").unwrap();
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
