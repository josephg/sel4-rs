use crate::basic_types::{CpuId, Paddr, PhysRegion};
use crate::boot::multiboot::{MMapEntry, MultibootBootInfo, MultibootInfoFlags, MultibootModule, MultibootPtr, MultibootSlice, MULTIBOOT_BOOTLOADER_MAGIC};
use crate::config::CONFIG_MAX_NUM_NODES;
use crate::console::init_serial;
use crate::kprintln;
use crate::utils::fixedarr::FixedArr;
use crate::utils::halt;
use ufmt::derive::uDebug;

#[unsafe(link_section = ".boot.text")]
fn try_boot_sys() -> Result<(), ()> {

    Ok(())
}


// #[derive(uDebug)]
// struct BootInfo {
//
// }

// #[derive(Error, uDebug)]
// enum MultibootError {
//
// }


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
const MAX_NUM_FREEMEM_REG: usize = 16;

#[derive(uDebug)]
struct BootState {
    /// region of available physical memory on platform
    avail_p_reg: PhysRegion,
    /// region containing the kernel image
    kern_p_reg: PhysRegion,

    // ui_info_t    ui_info;     /* info about userland images */

    /// Number of IOAPICs detected
    num_ioapic: u32,

    // paddr_t      ioapic_paddr[CONFIG_MAX_NUM_IOAPIC];
    // uint32_t     num_drhu; /* number of IOMMUs */
    // paddr_t      drhu_list[MAX_NUM_DRHU]; /* list of physical addresses of the IOMMUs */
    // acpi_rmrr_list_t rmrr_list;
    // acpi_rsdp_t  acpi_rsdp; /* copy of the rsdp */

    /// physical address where boot modules end
    mods_end_paddr: Paddr,
    /// physical address of first boot module
    boot_module_start: Paddr,
    /// number of detected cpus
    num_cpus: u32,

    /// lower memory size for boot code of APs to run in real mode
    mem_lower: u32,

    cpus: [CpuId; CONFIG_MAX_NUM_NODES],

    mem_p_regs: FixedArr<PhysRegion, MAX_NUM_FREEMEM_REG>,

    // mem_p_regs_t mem_p_regs;  /* physical memory regions */
    // seL4_X86_BootInfo_VBE vbe_info; /* Potential VBE information from multiboot */
    // seL4_X86_BootInfo_mmap_t mb_mmap_info; /* memory map information from multiboot */
    // seL4_X86_BootInfo_fb_t fb_info; /* framebuffer information as set by bootloader */
}

#[unsafe(link_section = ".boot.text")]
fn parse_mem_map(mmaps: &[MMapEntry]) -> Result<(), ()> {
    kprintln!("Parsing GRUB physical memory maps...");
    for m in mmaps {

    }

    Ok(())
}

#[unsafe(link_section = ".boot.text")]
fn try_boot_sys_mbi1(mbi: &MultibootBootInfo) -> Result<BootState, ()> {
    // TODO: Boot command line. Not sure if I ever want to support this, but its certainly in sel4.
    // (src/arch/x86/kernel/cmdline.c)
    // Could be nice for configuring debugging port when booting on real hardware.

    // I could return a proper result, but we're going to halt immediately if any error happens.
    // In this case, its simpler to just print out the error we get here and return Err(()) to bail.
    if mbi.flags & (MultibootInfoFlags::Memory as u32) == 0 {
        kprintln!("Boot loader did not provide information about physical memory size");
        return Err(());
    }

    if mbi.flags & (MultibootInfoFlags::Mods as u32) == 0 {
        kprintln!("Boot loader did not provide information about physical memory size");
        return Err(());
    }

    if mbi.mods.len < 1 {
        kprintln!("Expected at least 1 boot module (passed as initrd) for root process");
        return Err(());
    }

    kprintln!("Detected {} boot module(s)", mbi.mods.len);

    let mut mods_end_paddr = 0;

    let modules = unsafe { mbi.mods.to_slice(mbi) };
    // kprintln!("modules: {:?}", modules);
    for m in modules {
        let name = unsafe { m.name.as_cstr(mbi) };
        kprintln!("Mod {}: {:?}", name.unwrap().to_str().unwrap(), m);

        if m.mod_end < m.mod_start {
            kprintln!("Invalid boot module size!");
            return Err(());
        }
        // kprintln!("Mod {}: {:?}", "asdf", m);

        mods_end_paddr = mods_end_paddr.max(m.mod_end as Paddr);
    }

    // initialize the memory. We track two kinds of memory regions. Physical memory
    // that we will use for the kernel, and physical memory regions that we must
    // not give to the user. Memory regions that must not be given to the user
    // include all the physical memory in the kernel window, but also includes any
    // important or kernel devices.
    let mut mem_p_regs: FixedArr<PhysRegion, MAX_NUM_FREEMEM_REG> = FixedArr::new();

    if mbi.flags & (MultibootInfoFlags::MemMap as u32) != 0 {
        parse_mem_map(unsafe { mbi.mmap.to_slice(&mbi) })?;
    } else {
        todo!("old way")
    }


    // todo!()
    Ok(BootState {
        avail_p_reg: Default::default(),
        kern_p_reg: Default::default(),
        num_ioapic: 0,
        mods_end_paddr,
        boot_module_start: 0,
        num_cpus: 0,
        mem_lower: 0,
        cpus: Default::default(),
        mem_p_regs,
    })

}


// This is called from entry_64 in boot0.
#[unsafe(link_section = ".boot.text")]
#[unsafe(no_mangle)]
pub extern "C" fn boot_sys(multiboot_magic: u32, mbi: MultibootPtr) -> ! {
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
        kprintln!("Booting via multiboot v1 {:x}", mbi);

        let ptr = mbi as *const MultibootBootInfo;
        // The multiboot info struct is at mbi, which will be in the lower linear memory segment.
        let mbi: &'static MultibootBootInfo = unsafe { &*ptr };

        // Just this alone adds 35kb to the binary!
        // kprintln!("Multiboot info: {:#?}", mbi);

        if let Err(()) = try_boot_sys_mbi1(mbi) {
            kprintln!("Multiboot returned unexpected or unusable data.");
            halt();
        }

    }

    try_boot_sys().unwrap();

    kprintln!("END OF LINE ------ BEEEEEEPPPP");
    halt();
}
