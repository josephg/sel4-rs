

pub const PML4_ENTRY_BITS: usize = 3;
pub const PML4_INDEX_BITS: usize = 9;

pub const PML3_ENTRY_BITS: usize = 3;
pub const PML3_INDEX_BITS: usize = 9;

// pub const PML4_ENTRIES: usize = 1 << PML4_INDEX_BITS;
const PAGE_BITS: usize = 12;

/// Page table level 4 entry
#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct Pml4e(pub u64);

/// Sel4 calls this pdpte. ("Page directory page table entry")
#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct Pml3e(pub u64);

impl Pml4e {
    // const fn new(base_addr: u32, pat: bool, avl: u8, global: bool, dirty: bool, accessed: bool, cache_disabled: bool, write_through: bool, super_user: bool, rw: bool, present: bool) -> Self {
    //     const PDE_LARGE: bool = true;
    //
    //     Self(
    //         (base_addr & 0xffc00000) as u64
    //             | (pat as u64) << 12
    //             | (avl as u64) << 9
    //             | (global as u64) << 8
    //             | (PDE_LARGE as u64) << 7
    //             | (dirty as u64) << 6
    //             | (accessed as u64) << 5
    //             | (cache_disabled as u64) << 4
    //             | (write_through as u64) << 3
    //             | (super_user as u64) << 2
    //             | (rw as u64) << 1
    //             | (present as u64) << 0
    //     )
    // }

}

// align(1 << PAGE_BITS) - which is 4096.
#[repr(C, align(4096))]
pub(crate) struct BootPML4(pub [Pml4e; 1 << PML4_INDEX_BITS]);

#[repr(C, align(4096))]
pub(crate) struct BootPML3(pub [Pml3e; 1 << PML3_INDEX_BITS]);


/* For the boot code we create two windows into the physical address space
 * One is at the same location as the kernel window, and is placed up high
 * The other is a 1-to-1 mapping of the first 512gb of memory. The purpose
 * of this is to have a 1-to-1 mapping for the low parts of memory, so that
 * when we switch paging on, and are still running at physical addresses,
 * we don't explode. Then we also want the high mappings so we can start
 * running at proper kernel virtual addresses */

// Zero initializing for now. We'll fill this in at runtime.
//
// Note: we could fill this in here at const (compile time), but doing it this way allows the
// binary to be slightly smaller. Though only by 512 bits, so maybe whatever.

#[unsafe(link_section = ".phys_bss")]
pub static mut BOOT_PML4: BootPML4 = BootPML4([Pml4e(0); _]);

#[unsafe(link_section = ".phys_bss")]
pub static mut BOOT_PML3: BootPML3 = BootPML3([Pml3e(0); _]);
