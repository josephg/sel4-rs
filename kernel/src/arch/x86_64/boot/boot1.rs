//! This file is largely based on kernel/src/arch/x86/kernel/boot_sys.c
//!
//! It is the second part of the boot process (after boot0). This reads multiboot state to prepare
//! for the next stage of booting.
//!
//! Notably, [boot_sys] in this file is jumped to directly from the assembly code in boot0 after
//! setting up the kernel stack.

use crate::basic_types::{Paddr, PhysRegion};
use crate::arch::x86_64::boot::multiboot::{MMapEntry, MMapType, MultibootBootInfo, MultibootInfoFlags, MULTIBOOT_BOOTLOADER_MAGIC};
use crate::console::init_serial;
use crate::{kpanic, kprintln};
use crate::utils::{halt, NumUtils};
use crate::arch::constants::PAGE_BITS;
use crate::arch::x86_64::acpi::acpi_init;
use crate::arch::x86_64::boot::bootinfo::{BootState, MemPRegs, MAX_NUM_FREEMEM_REG};
use crate::arch::x86_64::U32Ptr;
use crate::hardware::PADDR_TOP;

const SEL4_MULTIBOOT_MAX_MMAP_ENTRIES: usize = 50;

const HIGHMEM_PADDR: usize = 0x100000;

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


/// Add the passed physical memory region to mem_p_regs. mem_p_regs is fixed size. This function
/// returns an error if we run out of room in the array - though this should be vanishingly rare.
#[unsafe(link_section = ".boot.text")]
fn add_mem_phys_regs(mem_p_regs: &mut MemPRegs, mut reg: PhysRegion) -> Result<(), ()> {
    if reg.start == reg.end {
        // This nonsensical comment from SeL4:
        // > Return true here if asked to add an empty region.
        // > Some of the callers round down the end address to
        return Ok(())
    }

    if reg.end > PADDR_TOP && reg.start > PADDR_TOP {
        // it's not an error for there to exist memory outside the kernel window,
        // we're just going to ignore it and leave it to be given out as device memory.
        return Ok(())
    }

    if reg.end > PADDR_TOP {
        assert!(reg.start <= PADDR_TOP); // Should be guaranteed from above.
        // Clamp a region to the top of the kernel window if it extends beyond.
        reg.end = PADDR_TOP;
    }

    match mem_p_regs.try_push(reg) {
        Ok(()) => {
            kprintln!("Added physical memory region 0x{:x} - 0x{:x}", reg.start, reg.end);
            Ok(())
        }
        Err(_) => {
            kprintln!("Warning: Dropping memory region 0x{:x} - 0x{:x}. Try increasing MAX_NUM_FREEMEM_REG", reg.start, reg.end);
            Err(())
        }
    }
}

/// SAFETY: We're going to do a bunch of raw memory reads based on the passed multiboot pointers.
/// This function is only correct if these pointers are valid.
///
/// We're relying on GRUB providing correct information about the physical memory regions here.
///
/// Returns Ok if all memory regions populated. Or Err if we ran out of space for regions in
/// mem_p_regs.
#[unsafe(link_section = ".boot.text")]
unsafe fn parse_mem_map(mem_p_regs: &mut MemPRegs, bytelen: u32, base_addr: U32Ptr<MMapEntry>) -> Result<(), ()> {
    // Annoyingly, the mmap table is technically a table of dynamically sized elements. In practice,
    // qemu and grub both seem to only produce items of exactly 20 bytes. But for correctness, I'm
    // going to walk the table in a way thats actually correct (according to the spec) here.
    //
    // Things are about to get *unsafe*.
    kprintln!("Parsing GRUB physical memory map...");
    let mut addr = base_addr;
    while addr.0 < base_addr.0 + bytelen {
        let ptr = addr.as_ptr();
        let m = unsafe { *ptr };

        let mem_start = m.base_addr;
        let mem_len = m.len;
        let m_type = m.mtype;

        // The SeL4 code at this location has this check:
        //         if (mem_start != (uint64_t)(word_t)mem_start) { ... }
        // But this is impossible to trip in 64 bit mode. (And the compiler agrees and compiles it
        // out). Given I don't plan to add 32 bit support here, I'm leaving this check out.

        kprintln!("\tPhysical memory region from {:x} size {:x} type {}", mem_start, mem_len, m_type);

        if m_type == MMapType::Usable as _
            && mem_start as usize >= HIGHMEM_PADDR
            && mem_len >= u64::bit(PAGE_BITS)
        {
            let reg = PhysRegion {
                start: mem_start.round_up(PAGE_BITS) as _,
                end: (mem_start + mem_len).round_down(PAGE_BITS) as _,
            };
            add_mem_phys_regs(mem_p_regs, reg)?;
        }

        // Advance the loop.
        addr.0 += m.size + size_of::<u32>() as u32;
    }

    Ok(())
}

/// This function creates and populates a BootState object on the stack. In SeL4 this happens in
/// static memory - which might be to reduce stack pressure? Anyway, its certainly cleaner rust code
/// like this.
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

    kprintln!("Detected {} boot module(s)", mbi.mods.len);

    let mut mods_end_paddr = 0;

    let modules = unsafe { mbi.mods.to_slice(mbi) };
    // kprintln!("modules: {:?}", modules);

    let Some(first_module) = modules.first() else {
        kprintln!("Expected at least 1 boot module (passed as initrd) for root process");
        return Err(());
    };

    // This is the entrypoint we jump to after initializing SeL4.
    let boot_module_start = first_module.mod_start as usize;

    for m in modules {
        let name = unsafe { m.name.try_as_cstr(mbi) };
        kprintln!("\tmod {}: {:?}", name.unwrap().to_str().unwrap(), m);

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
    let mut mem_p_regs: MemPRegs = MemPRegs::new();

    if mbi.flags & (MultibootInfoFlags::MemMap as u32) != 0 {
        // This will return an error if we ran out of room to store the list of memory regions.
        let result = unsafe {
            parse_mem_map(&mut mem_p_regs, mbi.mmap_bytelength, mbi.mmap_addr)
        };
        if let Err(()) = result {
            // kprintln!("Warning: Multiboot has reported more memory map entries \
            //        than the max amount that will be passed in the bootinfo, {}. \
            //        These extra regions will still be turned into untyped caps.",
            // MAX_NUM_FREEMEM_REG);
            // TODO: Actually match sel4's behaviour here. The current kernel saves this data and
            // passes it more or less directly into the root process's extra_bi region. But because
            // the might be weird extra stuff in here, I want to think a little bit more before
            // committing to that.

            kprintln!("Warning: Multiboot has reported more memory map entries \
                than the max amount that will be passed in the bootinfo, {}. \
                Extra entries are not available for use.",
                MAX_NUM_FREEMEM_REG
            );
        }
        // TODO: SeL4 also copies map entries into boot_state.mb_mmap_info.mmap.
    } else {
        // "Calculate memory the old way"
        // NOTE: This code has been hand ported from SeL4, but it has not been tested yet. That
        // makes me deeply uneasy.
        let start = HIGHMEM_PADDR;
        let avail = PhysRegion {
            start,
            end: (start + ((mbi.mem_upper as usize) << 10)).round_down(PAGE_BITS)
        };
        add_mem_phys_regs(&mut mem_p_regs, avail)?;
    }

    if mbi.flags & (MultibootInfoFlags::VBEInfo as u32) != 0 {
        // TODO: SeL4 passes VBE info to boot info struct. Since qemu doesn't seem to support any
        // of the multiboot graphics mode stuff, I'm going to ignore it for now.
        kprintln!("Warning: got VBE info from multiboot, but currently ignored.");
    } else {
        kprintln!("Multiboot gave us no video information");
    }

    let acpi_rsdp = acpi_init()?;

    // todo!()
    Ok(BootState {
        avail_p_reg: Default::default(),
        kern_p_reg: Default::default(),
        num_ioapic: 0,
        acpi_rsdp,
        mods_end_paddr,
        boot_module_start,
        num_cpus: 0,
        mem_lower: mbi.mem_lower,
        cpus: Default::default(),
        mem_p_regs,
    })
}


// pub fn vga_write_str(s: &str) {
//     let vga = 0xb8000 as *mut u8;
//     static mut COL: usize = 0;
//     static mut ROW: usize = 0;
//     let width = 80;
//
//     for b in s.bytes() {
//         unsafe {
//             match b {
//                 b'\n' => { ROW += 1; COL = 0; }
//                 _ => {
//                     let i = (ROW * width + COL) * 2;
//                     core::ptr::write_volatile(vga.add(i), b);       // char
//                     core::ptr::write_volatile(vga.add(i + 1), 0x05); // attr: light grey on black
//                     COL += 1;
//                     if COL >= width { ROW += 1; COL = 0; }
//                 }
//             }
//         }
//     }
// }

// This is called from entry_64 in boot0.
#[unsafe(link_section = ".boot.text")]
#[unsafe(no_mangle)]
pub extern "C" fn boot_sys(multiboot_magic: u32, mbi: u32) -> ! {
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
            kpanic!("Multiboot returned unexpected or unusable data.");
        }

    }

    try_boot_sys().unwrap();

    kprintln!("END OF LINE ------ BEEEEEEPPPP");
    halt();
}
