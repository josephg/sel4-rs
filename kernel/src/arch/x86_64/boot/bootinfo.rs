use ufmt::derive::uDebug;
use crate::arch::x86_64::acpi::AcpiRsdp;
use crate::basic_types::{CpuId, Paddr, PhysRegion};
use crate::config::CONFIG_MAX_NUM_NODES;
use crate::utils::fixedarr::FixedArr;

/// The maximum number of reserved regions.
///
/// This is simply set to 16 because thats the value in include/arch/x86/arch. The arm code has
/// more complex logic to calculate this, but 16 is probably fine.
///
/// Here's a comment from the riscv code (which also just arbitrarily picks 16):
///
/// > The value for the max number of free memory region is basically an arbitrary
/// > choice. We could calculate the exact number, but just picking 16 will also
/// > do for now. Increase this value if the boot fails.
pub(super) const MAX_NUM_FREEMEM_REG: usize = 16;

pub type MemPRegs = FixedArr<PhysRegion, MAX_NUM_FREEMEM_REG>;

/// This struct contains the kernel's boot state. In actual sel4 this object is a static global.
// #[derive(uDebug)]
pub(super) struct BootState {
    /// region of available physical memory on platform
    pub avail_p_reg: PhysRegion,
    /// region containing the kernel image
    pub kern_p_reg: PhysRegion,

    // ui_info_t    ui_info;     /* info about userland images */

    /// Number of IOAPICs detected
    pub num_ioapic: u32,

    // paddr_t      ioapic_paddr[CONFIG_MAX_NUM_IOAPIC];
    // uint32_t     num_drhu; /* number of IOMMUs */
    // paddr_t      drhu_list[MAX_NUM_DRHU]; /* list of physical addresses of the IOMMUs */
    // acpi_rmrr_list_t rmrr_list;

    /// A copy of the RSDP
    pub acpi_rsdp: AcpiRsdp,

    /// physical address where boot modules end
    pub mods_end_paddr: Paddr,
    /// physical address of first boot module
    pub boot_module_start: Paddr,
    /// number of detected cpus
    pub num_cpus: u32,

    /// lower memory size for boot code of APs to run in real mode
    pub mem_lower: u32,

    pub cpus: [CpuId; CONFIG_MAX_NUM_NODES],

    pub mem_p_regs: MemPRegs,

    // mem_p_regs_t mem_p_regs;  /* physical memory regions */
    // seL4_X86_BootInfo_VBE vbe_info; /* Potential VBE information from multiboot */
    // seL4_X86_BootInfo_mmap_t mb_mmap_info; /* memory map information from multiboot */
    // seL4_X86_BootInfo_fb_t fb_info; /* framebuffer information as set by bootloader */
}

struct BootStateVBE {

}