// Alright so:

// SeL4 officially supports both 32 and 64 bit modes of operation. But I don't really care about
// 32 bit support, because all modern x86 chips support running in 64 bit mode anyway.

// As a result, I've simplified a lot of this


// Constants are here:
// libsel4/sel4_arch_include/x86_64/sel4/sel4_arch/constants.h

/* for x86-64, the large page size is 2 MiB and huge page size is 1 GiB */

// Aliased to seL4_PageDirIndexBits in sel4.
pub const PD_INDEX_BITS: usize = 9; // seL4_PageDirIndexBits
pub const PAGE_BITS: usize = 12; // seL4_PageBits

pub const LARGE_PAGE_BITS: usize = 21; // seL4_LargePageBits
pub const PD_ENTRIES: usize = 1 << PD_INDEX_BITS;

// From sel4 include/plat/pc99/plat/64/plat_mode/machine/hardware.h:

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

/* WARNING: some of these constants are also defined in linker.lds
 * These constants are written out in full instead of using bit arithmetic
 * because they need to defined like this in linker.lds
 */

/* Define USER_TOP to be 1 before the last address before sign extension occurs.
 * This ensures that
 *  1. user addresses never needed to be sign extended to be valid canonical addresses
 *  2. the user cannot map the last page before addresses need sign extension. This prevents
 *     the user doing a syscall as the very last instruction and the CPU calculated PC + 2
 *     from being an invalid (non sign extended) address
 */
pub const USER_TOP: usize = 0x7FFF_FFFF_FFFF;

/* The first physical address to map into the kernel's physical memory
 * window */
pub const PADDR_BASE: usize = 0;

/* The base address in virtual memory to use for the 1:1 physical memory
 * mapping. Our kernel window is 2^39 bits (2^9 * 1gb) and the virtual
 * address range is 48 bits. Therefore our base is 2^48 - 2^39 */
pub const PPTR_BASE: usize = 0xffff_ff80_0000_0000;


/* Below the main kernel window we have any slots for the TLB bitmap */
const TLBBITMAP_PML4_RESERVED: usize = 0; // TODO: Needed for SMP!
const TLBBITMAP_PPTR: usize = 0;

// #define TLBBITMAP_PML4_RESERVED (TLBBITMAP_ROOT_ENTRIES * BIT(PML4_INDEX_OFFSET))
// #define TLBBITMAP_PPTR (PPTR_BASE - TLBBITMAP_PML4_RESERVED)



// Page directory entry.
#[derive(Copy, Clone, Default)]
struct Pde(u64);

impl Pde {
    const fn new(base_addr: u32, pat: bool, avl: u8, global: bool, dirty: bool, accessed: bool, cache_disabled: bool, write_through: bool, super_user: bool, rw: bool, present: bool) -> Self {
        const PDE_LARGE: bool = true;

        Self(
            (base_addr & 0xffc00000) as u64
            | (pat as u64) << 12
            | (avl as u64) << 9
            | (global as u64) << 8
            | (PDE_LARGE as u64) << 7
            | (dirty as u64) << 6
            | (accessed as u64) << 5
            | (cache_disabled as u64) << 4
            | (write_through as u64) << 3
            | (super_user as u64) << 2
            | (rw as u64) << 1
            | (present as u64) << 0
        )
    }
}

// 1<<12 = 4096
#[repr(C, align(4096))]
struct BootPD([Pde; PD_ENTRIES]);

impl BootPD {
    const fn new_default() -> Self {
        let mut pd = [Pde(0); _];

        let mid = PPTR_BASE >> LARGE_PAGE_BITS;
        for i in 0..mid {
            pd[i] = Pde::new((i << LARGE_PAGE_BITS) as u32, false, 0, true,
                             false, false, false, false,
                             false, true, true
            );
        }

        for i in mid..pd.len() {
            pd[i] = Pde::new(((i - mid) << LARGE_PAGE_BITS + ) as u32, false, 0, true,
                             false, false, false, false,
                             false, true, true
            );
        }

        Self(pd)
    }
}

#[unsafe(link_section = ".phys_bss")]
pub static mut BOOT_PD: BootPD = BootPD::new_default();
