// This is based on sel4's src/arch/x86/64/head.S. Lots of the code here is 32 bit code - which
// I could either put in a separate crate and awkwardly, manually glue together. Or write in
// inline assembly, which is what I'm doing here.
//
// The downside of this approach is all this code is assembly, not rust. But at least the package
// layout is much simpler!

use core::arch::naked_asm;
use crate::{KERNEL_STACK, KERNEL_STACK_BITS, boot_sys};

#[allow(dead_code)]
unsafe extern "C" {
    static boot_stack_top: u8;
    static boot_stack_bottom: u8;

    // fn init_boot_pd();
}

// pub const PML4_ENTRY_BITS: usize = 3;
pub const PML4_INDEX_BITS: usize = 9;

// pub const PML3_ENTRY_BITS: usize = 3;
pub const PML3_INDEX_BITS: usize = 9;

// pub const PML2_ENTRY_BITS: usize = 3;
// I'm not sure why the boot page directory in sel4 has 2048 entries.
pub const PML2_INDEX_BITS: usize = 11;

#[repr(align(4096))]
struct Align4k<T>(T);

// "PM level 4"
#[unsafe(link_section = ".phys.bss")]
static mut BOOT_PML4: Align4k<[u64; 1 << PML4_INDEX_BITS]> = Align4k([0; _]);

// "Page directory page table"
#[unsafe(link_section = ".phys.bss")]
static mut BOOT_PML3: Align4k<[u64; 1 << PML3_INDEX_BITS]> = Align4k([0; _]);

// "Page directory", which has 2048 entries to cover the whole 4gb of addressable memory in 32 bit
// mode
#[unsafe(link_section = ".phys.bss")]
static mut BOOT_PML2: Align4k<[u64; 1 << PML2_INDEX_BITS]> = Align4k([0; _]);


const fn new_gdt(flags: u8, access_byte: u8) -> u64 {
    // The base and limit are both ignored in long mode, so I'll leave them 0. The only things we
    // need to actually set are the flags and access byte.
    ((flags as u64) << 48) | ((access_byte as u64) << 40)
}


#[unsafe(naked)]
#[unsafe(link_section = ".phys.text")]
extern "C" fn print_string() {
    naked_asm!(r"
    .code32
    .Lloop:
        mov dx, 0x3f8+5
    .Lwait:
        in al, dx
        test al, 0x20
        jz .Lwait
        sub dx, 5
        mov al, byte ptr [ebx]
        out dx, al
        inc ebx
        dec ecx
        jnz .Lloop
        ret
    ")
}

// 64-bit variant: expects pointer in RDI and length in RSI (SysV ABI).
#[unsafe(naked)]
#[unsafe(link_section = ".phys.text")]
extern "C" fn print_string64() {
    naked_asm!(r"
    .code64
    .Lloop64:
        mov dx, 0x3f8+5
    .Lwait64:
        in al, dx
        test al, 0x20
        jz .Lwait64
        sub dx, 5
        mov al, byte ptr [rdi]
        out dx, al
        inc rdi
        dec rsi
        jnz .Lloop64
        ret
    ")
}

/*
 *          2^64 +-------------------+
 *               | Kernel Page PDPT  | --+
 *   2^64 - 2^39 +-------------------+ PPTR_BASE
 *               |    TLB Bitmaps    |   |
 *               +-------------------+   |
 *               |                   |   |
 *               |     Unmapped      |   |
 *               |                   |   |
 *   2^64 - 2^47 +-------------------+   |
 *               |                   |   |
 *               |   Unaddressable   |   |
 *               |                   |   |
 *          2^47 +-------------------+ USER_TOP
 *               |                   |   |
 *               |       User        |   |
 *               |                   |   |
 *           0x0 +-------------------+   |
 *                                       |
 *                         +-------------+
 *                         |
 *                         v
 *          2^64 +-------------------+
 *               |                   |
 *               |                   |     +------+      +------+
 *               |                   | --> |  PD  | -+-> |  PT  |
 *               |  Kernel Devices   |     +------+  |   +------+
 *               |                   |               |
 *               |                   |               +-> Log Buffer
 *               |                   |
 *   2^64 - 2^30 +-------------------+ KDEV_BASE
 *               |                   |
 *               |                   |     +------+
 *               |    Kernel ELF     | --> |  PD  |
 *               |                   |     +------+
 *               |                   |
 *   2^64 - 2^29 +-------------------+ PPTR_TOP / KERNEL_ELF_BASE
 *               |                   |
 *               |  Physical Memory  |
 *               |       Window      |
 *               |                   |
 *   2^64 - 2^39 +-------------------+ PPTR_BASE
 */


#[unsafe(link_section = ".phys.data")]
static PAGE_ENABLED_MSG: [u8; 16] = *b"Paging enabled!\n";


#[unsafe(naked)]
#[unsafe(link_section = ".phys.text")]
extern "C" fn setup_pagetable() {
    naked_asm!(r"
        .code32
            // TODO: check huge page support

            // Zero pml4 and pml3
            mov edi, offset {boot_pml4}
            mov edx, 0
            mov ecx, 1024
        .Lzero_pml4:
            mov [edi], edx
            add edi, 4
            loop .Lzero_pml4

            mov edi, offset {boot_pml3}
            mov ecx, 1024
        .Lzero_pml3:
            mov [edi], edx
            add edi, 4
            loop .Lzero_pml3

            // Setup the level 4 page table with a single entry.
            mov edi, offset {boot_pml4}
            mov ecx, offset {boot_pml3}
            or ecx, 0x7 // 0x7 = preset, writable, user accessable.
            // (Other bits are zero because of alignment.)

            // 3 copied mappings:
            mov [edi], ecx // Lower (physical)
            mov [edi+0x800], ecx // [256]. Upper half, for kernel's physical map. We jump here!
            mov [edi+4088], ecx // [511]. Map the top page for sign-extended compatibility mode pointers.

            // Setup the level 3 page table (aka PDPT)
            mov ecx, offset {boot_pml2}
            or ecx, 0x7 // same bits - present, writable, user accessible

            mov edi, offset {boot_pml3}
            mov [edi], ecx // 0-1gb
            mov [edi+4080], ecx // [510]. Mapped for sign extension compatibility.
            add ecx, 0x1000
            mov [edi+8], ecx // 1-2gb
            add ecx, 0x1000
            mov [edi+16], ecx // 2-3gb
            add ecx, 0x1000
            mov [edi+24], ecx  // 3-4gb

            // Setup level 2 page tables using large pages (2mb)
            mov edi, offset {boot_pml2}
            mov edx, 0x87 // Flags. Present, writable, user, and large mode.

            // Loop through assigning L2PT entries (PD). 2048 entries * 2mb = Entire 4gb.
            mov ecx, 2048
        .Lmap_pd:
            mov     [edi], edx
            add     edx, 0x200000 // Increment in 2mb physical chunks
            add     edi, 8 // Next entry
            loop    .Lmap_pd



            mov ebx, offset {msg}
            mov ecx, {len}
            call {print_string}


        ret
    ",
        boot_pml4 = sym BOOT_PML4,
        boot_pml3 = sym BOOT_PML3,
        boot_pml2 = sym BOOT_PML2,


        msg = sym PAGE_ENABLED_MSG,
        len = const PAGE_ENABLED_MSG.len(),
        print_string = sym print_string,
    )
}

// Layout matches the GAS table:
//  entry 0: null
//  entry 1: kernel code (access=0x98, flags=0x20, base/limit=0)
//  entry 2: kernel data (access=0x90, flags=0x00, base/limit=0)

// I'm using the natural 8 byte alignment here. SeL4 uses 16 byte alignment, but thats not
// necessary.
#[unsafe(link_section = ".phys.data")]
static GDT64: [u64; 3] = [
    0, // Required NULL GDT segment
    new_gdt(0x20, 0x98), // code. 0x20 = 64 bit code. 0x98 = executable.
    new_gdt(0x00, 0x90), // Data
];

#[repr(C, packed)]
struct GdtPtr {
    limit: u16,
    base: *const u8,
}

// Safe because the pointer is to a read-only static and never mutated.
unsafe impl Sync for GdtPtr {}

#[unsafe(link_section = ".phys.data")]
static GDT64_PTR: GdtPtr = GdtPtr {
    limit: (GDT64.len() * 8 - 1) as u16,
    base: GDT64.as_ptr() as *const u8,
};


const IA32_EFER_MSR: u32 = 0xC0000080;

/// Enable x64 mode on the current CPU.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".phys.text")]
extern "C" fn enable_x64_mode() {
    naked_asm!(r"
        .code32
            // TODO: Check for PCID support, which should be available on all modern CPUs.
            // Save L4PT
            mov eax, offset {boot_pml4}
            mov cr3, eax

            // Enable PAE (bit 5), which is required for 64 bit mode.
            mov eax, cr4
            or  eax, 0x20
            mov cr4, eax

            // Set Long Mode Extension (bit 8) in IA32_EFER (MSR 0xC000_0080).
            mov ecx, {IA32_EFER_MSR}
            rdmsr
            or eax, 0x100
            wrmsr

            // Enable paging (bit 31) in CR0. With LME set, this enters long mode.
            mov eax, cr0
            or  eax, 0x80000000
            mov cr0, eax

            // PCID is only available on modern intel chips. AMD just implements invpcid, which
            // works slightly differently. Given I'm on an amd chip, I'm just going to implement
            // the latter.
            //
            // If we try to enable pcid when its not available we get a GPF.
            // mov eax, cr4
            // or  eax, 0x20000
            // mov cr4, eax

            ret
        ",
        boot_pml4 = sym BOOT_PML4,
        IA32_EFER_MSR = const IA32_EFER_MSR,
    )
}

#[unsafe(naked)]
#[unsafe(link_section = ".phys.text")]
extern "C" fn enable_syscalls() {
    naked_asm!("ret")
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".phys.text")]
extern "C" fn common_init() {
    naked_asm!(r"
        .code32

        // Disable paging
        mov eax, cr0
        and eax, 0x7fffffff
        mov cr0, eax

        // enable fsgsbase

        // Initialize boot PML4 and switch to long mode.
        call {setup_pagetable}
        call {enable_x64_mode}
        lgdt {gdt64_ptr}

        call {enable_syscalls}

        ret
    ",
        setup_pagetable = sym setup_pagetable,
        enable_x64_mode = sym enable_x64_mode,
        enable_syscalls = sym enable_syscalls,
        gdt64_ptr = sym GDT64_PTR,
    )
}


#[unsafe(link_section = ".phys.data")]
static STR: [u8; 9] = *b"hi there\n";


// #[unsafe(naked)]
// #[unsafe(link_section = ".phys.text")]
// extern "C" fn junk() {
//     naked_asm!(r"
//     .code32
//         mov ebx, offset {msg}
//         mov ecx, {len}
//         call {print_string}
//         ret
//     ",
//         msg = sym STR,
//         len = const STR.len(),
//         print_string = sym print_string,
//     )
// }
//
//
// #[unsafe(naked)]
// #[unsafe(link_section = ".phys.text")]
// extern "C" fn junk64() {
//     naked_asm!(r"
//     .code64
//         mov rdi, offset {msg}
//         mov rsi, {len}
//         call {print_string64}
//         ret
//     ",
//         msg = sym STR,
//         len = const STR.len(),
//         print_string64 = sym print_string64,
//     )
// }




// This is called on the BSP. For now I'm assuming multiboot - though ideally it'd be nice to
// actually check what mode the CPU is in and boot this kernel correctly in all cases.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".phys.text")]
pub extern "C" fn _start() -> ! {
    naked_asm!(r"
        .code32
            // Assume we are MultiBooted, e.g. by GRUB.
            // See MultiBoot Specification: www.gnu.org/software/grub/manual/multiboot
            // We'll check the magic number later.
            mov edi, eax // Multiboot magic number
            mov esi, ebx // multiboot info ptr.

            // Kernel boot stack pointer
            lea esp, [{boot_stack_top}]

            // Reset EFLAGS and disable interrupts
            push 0
            popf

            // Preserve args for next call. Pushed as 8 byte values so we can more easily pop later.
            push 0
            push esi // multiboot_info
            push 0
            push edi // multiboot_magic


            // TODO: Check for required features:
            //   - Large pages
            //   - invpcid
            //   - long mode
            //   - syscall
            // TODO: Check / clear CPU state. Make sure we're currently in 32 bit mode.

            call {common_init}

            // This is awkward, but it works around a bug in the llvm linker in intel assembly mode.
            // The at&t syntax doesn't have this issue.
            push 0x8
            mov eax, offset {_start64}
            push eax
            retf
    ",
        boot_stack_top = sym boot_stack_top,
        common_init = sym common_init,
        _start64 = sym _start64,
    )
}



#[unsafe(naked)]
#[unsafe(link_section = ".phys.text")]
extern "C" fn _start64() -> ! {
    // Ok we should now be in 64 bit mode. Bounce to the virtual memory region rather than using
    // the physical memory map.
    naked_asm!(r"
        .code64
            mov rax, offset {_entry_64}
            jmp rax
    ",
        // jmp {_entry_64}
        _entry_64 = sym _entry_64,
    )
}


// Boot section!
#[unsafe(naked)]
#[unsafe(link_section = ".boot.text")]
extern "C" fn _entry_64() -> ! {
    naked_asm!(r"
        .code64
            // Update stack pointer
            mov rax, 0xffffffff80000000
            add rsp, rax
            add rbp, rax

            // Pop the multiboot parameters off
            pop rdi
            pop rsi

            // Load the real kernel stack
            lea rsp, [{kernel_stack} + 1 << {KERNEL_STACK_BITS}]

            // Set restore_user_context() as return EIP, which will start the root task as soon as
            // boot_sys returns. (??)
            // push restore_user_context

            jmp {boot_sys}
    ",
        kernel_stack = sym KERNEL_STACK,
        KERNEL_STACK_BITS = const KERNEL_STACK_BITS,
        boot_sys = sym boot_sys,
        // junk64 = sym junk64,
    )
}

// pub fn _start() -> ! {
//     loop {}
// }
