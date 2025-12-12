#![no_std]

mod boot_pt;

use core::arch::naked_asm;
use core::panic::PanicInfo;
use no_panic::no_panic;
use crate::boot_pt::{Pml3e, Pml4e, BOOT_PML3, BOOT_PML4};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[allow(dead_code)]
unsafe extern "C" {
    static boot_stack_top: u8;
    static boot_stack_bottom: u8;

    // fn init_boot_pd();
}


// pml4e_t boot_pml4[BIT(PML4_INDEX_BITS)] ALIGN(BIT(seL4_PageBits)) VISIBLE PHYS_BSS;
// pdpte_t boot_pdpt[BIT(PDPT_INDEX_BITS)] ALIGN(BIT(seL4_PageBits)) VISIBLE PHYS_BSS;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot32.text")]
#[no_panic]
#[allow(static_mut_refs)]
unsafe fn setup_pml4() {
    // call huge_page_check.

    // Paging should already be disabled by _start.

    // This should be unnecessary.
    unsafe {
        BOOT_PML4.0.fill(Pml4e(0));
        BOOT_PML3.0.fill(Pml3e(0));
    }
}

// This code is based on the equivalent code in SeL4 - from src/arch/x86/64/head.S
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".phys.text")]
pub extern "C" fn _start() -> ! {
    naked_asm!(
        // Assume we are MultiBooted, e.g. by GRUB.
        // See MultiBoot Specification: www.gnu.org/software/grub/manual/multiboot
        // We'll check the magic number later.
        "mov edi, eax", // Multiboot magic number
        "mov esi, ebx", // multiboot info ptr.

        // Kernel boot stack pointer
        "lea esp, [{boot_stack_top}]",

        // Reset EFLAGS and disable interrupts
        "push 0",
        "popf",

        // Preserve args for next call. Pushed as 8 byte values so we can more easily pop later.
        "push 0",
        "push esi",
        "push 0",
        "push edi",

        // SeL4 has 2 code paths: one for "raw" bootstrapping and another for multiboot. For now
        // I'm going to only support multiboot, and inline the common_init code here.
        //"call {common_init}",

        // Disable paging
        "mov eax, cr0",
        "and eax, 0x7fffffff",
        "mov cr0, eax",

        // enable fsgsbase

        // Initialize boot PML4 and switch to long mode.
        "call {setup_pml4}",

        // "call {enable_paging}",

        "pop edi",
        "pop esi",

        // Stop using shared boot stack and get a real stack and move to the top of the stack
        //"lea esp, [{kernel_stack_top}]"

        "push esi",
        "push edi",
        // push restore_user_context
        // jmp boot_sys

        boot_stack_top = sym boot_stack_top,
        setup_pml4 = sym setup_pml4,
    )
}



// // This code is based on the equivalent code in SeL4 - from src/arch/x86/32/head.S
// #[unsafe(naked)]
// #[unsafe(no_mangle)]
// #[unsafe(link_section = ".boot32.text")]
// pub extern "C" fn _start() -> ! {
//     naked_asm!(
//         // Assume we are MultiBooted, e.g. by GRUB.
//         // See MultiBoot Specification: www.gnu.org/software/grub/manual/multiboot
//         "mov edi, eax", // Multiboot magic
//         "mov esi, ebx", // multiboot info ptr.
//
//         // Kernel boot stack pointer
//         "lea esp, [{boot_stack_top}]",
//
//         // Reset EFLAGS and disable interrupts
//         "push 0",
//         "popf",
//
//         // Preserve args for next call
//         "push esi",
//         "push edi",
//
//         // Set up page directory & enable paging
//         "call {init_boot_pd}",
//         // "call {enable_paging}",
//
//         "pop edi",
//         "pop esi",
//
//         // Stop using shared boot stack and get a real stack and move to the top of the stack
//         //"lea esp, [{kernel_stack_top}]"
//
//         "push esi",
//         "push edi",
//         // push restore_user_context
//         // jmp boot_sys
//
//         boot_stack_top = sym boot_stack_top,
//         init_boot_pd = sym init_boot_pd,
//     )
// }
