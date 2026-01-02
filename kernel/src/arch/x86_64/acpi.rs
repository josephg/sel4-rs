//! This file contains the ACPI boot time probe and we do a minimal parse of the ACPI data.
//!
//! This is not sufficient for a full operating system. Actually interacting with ACPI to set up
//! power management and load device drivers is left as an exercise for the root task.
//!
//! This file is ported from SeL4:
//! - include/plat/pc99/plat/machine/acpi.h
//! - src/plat/pc99/machine/acpi.c

use core::slice;
use ufmt::derive::uDebug;
use crate::arch::U32Ptr;
use crate::arch::x86_64::machine::{BIOS_PADDR_END, BIOS_PADDR_START};
use crate::{const_assert, kprintln};
use crate::basic_types::Paddr;
use crate::utils::fixedarr::FixedArr;
use super::devices::MAX_NUM_DRHU;

const ACPI_V1_SIZE: usize = 20;
const ACPI_V2_SIZE: usize = 36;

/// Generic System Descriptor Table Header.
///
/// Not guaranteed to be aligned in memory.
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct AcpiHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: [u8; 4],
    creator_revision: u32,
}

/// ACPI Root System Descriptor Pointer. Note this structure IS NOT stored in a way that's memory
/// aligned.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub(crate) struct AcpiRsdp {
    /// "RSD PTR "
    signature: [u8; 8],
    /// Number chosen such that the first 20 bytes of the table add to 0
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    /// Given we're 64 bit only, we can use the xsdt instead of rsdt. But the rsdt is still valid
    /// and gives us everything we need to set up the kernel.
    rsdt_address: u32,

    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,

    _reserved: [u8; 3],
}

const_assert!(size_of::<AcpiRsdp>() == ACPI_V2_SIZE);

#[repr(C, packed)]
pub(crate) struct AcpiRsdt {
    header: AcpiHeader,
    /// The RSDT is variable sized, and contains some number of entries which we can calculate from
    /// the header length field.
    // TODO: Replace this u32 with a U32Ptr.
    entries: [u32; 0],
}

/// Fixed ACPI description table (FADT). Partial as we only need flags.
#[repr(C)]
struct AcpiFadt {
    header: AcpiHeader,
    _reserved: [u8; 76],
    flags: u32,
}

#[unsafe(link_section = ".boot.text")]
fn acpi_calc_checksum<T>(obj: &T) -> u8 {
    let slice = unsafe {
        slice::from_raw_parts(obj as *const _ as *const u8, size_of::<T>())
    };

    let mut checksum: u8 = 0;
    for m in slice {
        checksum = checksum.wrapping_add(*m);
    }
    checksum
}

#[unsafe(link_section = ".boot.text")]
fn checksum_valid<T>(obj: &T) -> bool {
    acpi_calc_checksum(obj) == 0
}

/* workaround because string literals are not supported by C parser */
// const char acpi_str_fadt[] = {'F', 'A', 'C', 'P', 0};
// const char acpi_str_apic[] = {'A', 'P', 'I', 'C', 0};
// const char acpi_str_dmar[] = {'D', 'M', 'A', 'R', 0};

const RSDP_SIGNATURE: [u8; 8] = *b"RSD PTR ";



impl AcpiRsdp {
    /// This function scans the BIOS memory area for a valid RSDP.
    #[unsafe(link_section = ".boot.text")]
    fn find_from_bios() -> Option<U32Ptr<Self>> {
        // The ACPI RSDP is somewhere in memory in the main memory area below 1MB. It will always be
        // aligned on a 16 byte boundary. To find it, we scan the BIOS memory region looking for
        // something that has the signature "RSD PTR " and a valid checksum.
        //
        // Note that we scan each 16 byte aligned position and read 20 bytes forward.

        // Also note that documentation says it may be in the Extended BIOS Data Area instead. This
        // seems to be fine in practice, but its possible we don't find the RSDP using this method.

        // This might be a cleaner way to implement this:
        // let mem_region = unsafe {
        //     slice::from_raw_parts(BIOS_PADDR_START as *const u8, (BIOS_PADDR_END - BIOS_PADDR_START) as usize)
        // };
        //
        // mem_region.windows(20).step_by(16).find_map(|window| {
        //
        // });

        for addr in (BIOS_PADDR_START..BIOS_PADDR_END).step_by(16) {
            // The C code from SeL4 checks the last 16 byte offset address. If the RSDP were there, it
            // would spill past the end of the BIOS section. I'm going to match the behaviour here, but
            // I think its probably a (benign) bug.
            let chunk_ptr = addr as *const [u8; 20];

            // The signature is the first 8 bytes of the chunk.
            let sig = chunk_ptr as *const [u8; 8];

            if unsafe { *sig == RSDP_SIGNATURE } {
                let chunk = unsafe { &*chunk_ptr };
                if checksum_valid(chunk) {
                    return Some(U32Ptr::new(addr));
                }
            }
        }

        None
    }

    #[unsafe(link_section = ".boot.text")]
    pub(super) fn init() -> Result<AcpiRsdp, ()> {
        let Some(rsdp_ptr) = AcpiRsdp::find_from_bios() else {
            kprintln!("BIOS: No ACPI support detected!");
            return Err(());
        };

        let rsdp = unsafe { rsdp_ptr.as_static_ref() };

        kprintln!("ACPI: RSDP paddr=0x{:x} revision={}", rsdp_ptr.0, rsdp.revision);

        if rsdp.revision >= 2 { // DEPARTURE: >=2 instead of >0.
            // Check the extended checksum is also valid.
            if !checksum_valid(rsdp) {
                kprintln!("BIOS: ACPIv2 information corrupt!");
                return Err(());
            }
        }

        // DEPARTURE: SeL4 calls acpi_table_init here to make sure the ACPI table is correctly mapped
        // in to memory. But this is unnecessary in 64 bit mode, where the entire lower 32 bit range
        // is identity mapped already.
        Ok(*rsdp)
    }

    #[unsafe(link_section = ".boot.text")]
    pub(crate) fn get_rsdt(&self) -> &AcpiRsdt {
        let rsdp_addr = if self.revision < 2 {
            // RSDT.
            let rsdt_address_ptr = &raw const self.rsdt_address;
            let rsdt_address = unsafe { rsdt_address_ptr.read_unaligned() };
            assert_ne!(rsdt_address, 0, "RSDT pointer is null");
            rsdt_address as usize
        } else {
            // XSDT.
            // DEPARTURE: SeL4 always uses RSDT, and never XSDT when its available.
            let xsdt_address_ptr = &raw const self.xsdt_address;
            let xsdt_address = unsafe { xsdt_address_ptr.read_unaligned() };
            assert_ne!(xsdt_address, 0, "XSDT pointer is null");
            xsdt_address as usize
        };

        kprintln!("BIOS: RSDT paddr=0x{:x}", rsdp_addr);

        // SAFETY: This depends on the RSDP being setup correctly by BIOS / UEFI.
        let rsdt = unsafe { &*(rsdp_addr as *const AcpiRsdt) };

        rsdt
    }
}

impl AcpiRsdt {
    // DEPARTURE: This doesn't exist in SeL4.
    // TODO: It'd be nice to make an iterator for this.
    #[unsafe(link_section = ".boot.text")]
    pub(crate) fn print_table_entries(&self) {
        assert!(self.header.length as usize >= size_of::<AcpiHeader>());

        // Divide by uint32_t explicitly as this is the size as mandated by the ACPI standard.
        let entries: u32 = (self.header.length - size_of::<AcpiHeader>() as u32) / size_of::<u32>() as u32;
        let base_ptr = &raw const self.entries as *const u32;

        kprintln!("ACPI number of entries: {}", entries);
        // The entry table is misaligned - at least on qemu. Have to handle this carefully.
        for count in 0..entries {
            let entry_ptr_ptr = unsafe { base_ptr.add(count as usize) };
            let entry_ptr = unsafe { entry_ptr_ptr.read_unaligned() };

            let header_ptr = entry_ptr as usize as *const AcpiHeader;
            let header = unsafe { header_ptr.read_unaligned() };
            let sig = core::str::from_utf8(header.signature.as_slice()).unwrap();
            kprintln!("  RSDT entry at 0x{:x} with signature {}", entry_ptr, sig);
        }
    }
}


// DEPARTURE: This fadt scan is only to check flags if we have CONFIG_USE_LOGICAL_IDS set.
// But logical IDs aren't supported. So I'm just gonna skip this!

// #[unsafe(link_section = ".boot.text")]
// pub(super) fn acpi_fadt_scan(rsdp: &AcpiRsdp) {
//     let rsdt = unsafe { rsdp.rsdt_address.as_static_ref() };
//
//     assert!(rsdt.header.length as usize >= size_of::<AcpiHeader>());
//
//     // Divide by uint32_t explicitly as this is the size as mandated by the ACPI standard.
//     let entries: u32 = (rsdt.header.length - size_of::<AcpiHeader>() as u32) / size_of::<u32>() as u32;
//     let base_ptr = &raw const rsdt.entries as *const u32;
//
//     // The entry table is misaligned - at least on qemu. Have to handle this carefully.
//     for count in 0..entries {
//         let entry_ptr_ptr = unsafe { base_ptr.add(count as usize) };
//         let entry_ptr = unsafe { entry_ptr_ptr.read_unaligned() };
//
//         // This pointer is probably also misaligned. :p
//         let fadt_ptr = entry_ptr as usize as *const AcpiFadt;
//         // It feels gross reading the whole fadt structure onto the stack. I'm not sure if thats
//         // actually happening here but I kinda hate it.
//         //
//         // Hopefully the compiler can generate some acceptable code here...
//         let fadt = unsafe { fadt_ptr.read_unaligned() };
//
//         if &fadt.header.signature == b"FACP" {
//             if checksum_valid(&fadt) {
//                 kprintln!("ACPI: FADT paddr=0x{:x} flags=0x{:x}", fadt_ptr as usize, fadt.flags);
//             }
//         }
//
//
//         // p.read_unaligned();
//     }
// }

// TODO: This function is disabled, because DMAR only exists on intel chipsets.
// #[unsafe(link_section = ".boot.text")]
// pub fn acpi_dmar_scan(rsdt: &AcpiRsdt, drhu_list: &mut FixedArr<Paddr, MAX_NUM_DRHU>, p3: ()) {
//     assert!(rsdt.header.length as usize >= size_of::<AcpiHeader>());
//
//     // Divide by uint32_t explicitly as this is the size as mandated by the ACPI standard.
//     let entries: u32 = (rsdt.header.length - size_of::<AcpiHeader>() as u32) / size_of::<u32>() as u32;
//     let base_ptr = &raw const rsdt.entries as *const u32;
//
//     kprintln!("Entries: {}", entries);
//     // The entry table is misaligned - at least on qemu. Have to handle this carefully.
//     for count in 0..entries {
//         let entry_ptr_ptr = unsafe { base_ptr.add(count as usize) };
//         let entry_ptr = unsafe { entry_ptr_ptr.read_unaligned() };
//
//         let header_ptr = entry_ptr as usize as *const AcpiHeader;
//         let header = unsafe { header_ptr.read_unaligned() };
//         let sig = core::str::from_utf8(header.signature.as_slice()).unwrap();
//         kprintln!("RSDT entry at 0x{:x} with signature {}", entry_ptr, sig);
//
//         // TODO: DMAR (Device Remapping) is only supported on intel CPUs. Until I have an intel CPU
//         // to test with, I'm skipping IOMMU.
//     }
// }

