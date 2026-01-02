//! This file contains the ACPI boot time probe and we do a minimal parse of the ACPI data.
//!
//! This is not sufficient for a full operating system. Actually interacting with ACPI to set up
//! power management and load device drivers is left as an exercise for the root task.
//!
//! This file is ported from SeL4:
//! - include/plat/pc99/plat/machine/acpi.h
//! - src/plat/pc99/machine/acpi.c

use core::{ptr, slice};
use core::marker::PhantomData;
use core::mem::offset_of;
use ufmt::derive::uDebug;
use crate::arch::U32Ptr;
use crate::arch::x86_64::machine::{BIOS_PADDR_END, BIOS_PADDR_START};
use crate::{const_assert, kprint, kprintln};
use crate::basic_types::{CpuId, Paddr};
use crate::utils::fixedarr::FixedArr;
use super::devices::MAX_NUM_DRHU;
use crate::config::*;

const ACPI_V1_SIZE: usize = 20;
const ACPI_V2_SIZE: usize = 36;

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

/// The signature for a given table entry. It might be nice to newtype this.
type RsdtSig = [u8; 4];

/// Generic System Descriptor Table Header.
///
/// Not guaranteed to be aligned in memory.
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct AcpiHeader {
    signature: RsdtSig,
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: [u8; 4],
    creator_revision: u32,
}

const_assert!(size_of::<AcpiRsdp>() == ACPI_V2_SIZE);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct AcpiRsdt {
    header: AcpiHeader,
    /// The RSDT is variable sized, and contains some number of entries which we can calculate from
    /// the header length field.
    // TODO: Replace this u32 with a U32Ptr.
    entries: [u32; 0],
}

/// Fixed ACPI description table (FADT). Partial as we only need flags.
#[repr(C)] // Not packed for some reason.
#[derive(Copy, Clone)]
struct AcpiFadt {
    header: AcpiHeader,
    _reserved: [u8; 76],
    flags: u32,
}
const_assert!(size_of::<AcpiFadt>() == size_of::<AcpiHeader>() + 80);

// *** MADT (Multiple APIC Description Table). These describe the list of CPUs available on the
// system and the list of IOAPICs available.

/// Multiple APIC Description Table (MADT). Signature is "APIC".
#[repr(C, packed)]
#[derive(Copy, Clone)]
struct AcpiMadt {
    header: AcpiHeader,
    apic_addr: u32,
    flags: u32,
}
const_assert!(size_of::<AcpiMadt>() == size_of::<AcpiHeader>() + 8);

#[repr(C)]
#[derive(Copy, Clone)]
struct AcpiMadtHeader {
    madt_type: u8,
    length: u8,
}
const_assert!(size_of::<AcpiMadtHeader>() == 2);

#[derive(uDebug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum AcpiMadtStructType {
    /// Represents a single logical processor and its local interrupt controller.
    APIC = 0,
    IOAPIC = 1,
    // /// Interrupt Source Override
    // ISO = 2,
    /// Identical to Local APIC. Only used when the ioapic struct would overflow.
    X2APIC = 9,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct AcpiMadtApic {
    header: AcpiMadtHeader,
    cpu_id: u8,
    apic_id: u8,
    flags: u32,
}
const_assert!(size_of::<AcpiMadtApic>() == size_of::<AcpiMadtHeader>() + 6);

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct AcpiMadtX2Apic {
    header: AcpiMadtHeader,
    _reserved: u16,
    x2apic_id: u32,
    flags: u32,
    acpi_processor_uid: u32,
}
const_assert!(size_of::<AcpiMadtX2Apic>() == size_of::<AcpiMadtHeader>() + 14);

#[repr(C)] // not packed
#[derive(Copy, Clone)]
struct AcpiMadtIOApic {
    header: AcpiMadtHeader,
    ioapic_id: u8,
    _reserved: u8,
    ioapic_addr: u32,
    gsib: u32,
}
const_assert!(size_of::<AcpiMadtIOApic>() == size_of::<AcpiMadtHeader>() + 10);

// ISOs are defined in SeL4 but never actually used.
// #[repr(C)] // not packed
// #[derive(Copy, Clone)]
// struct AcpiMadtIso {
//     header: AcpiMadtHeader,
//     /// Always 0 (ISA)
//     bus: u8,
//     source: u8,
//     gsi: u32,
//     flags: u16,
// }
//
// // We can't assert on the sizeof acpi_madt_iso because it contains trailing
// // padding.
// const_assert!(offset_of!(AcpiMadtIso, flags) == size_of::<AcpiMadtHeader>() + 6);






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

struct AcpiIterator<'a> {
    // Honestly I'd much rather use a slice here, but the ACPI table is often misaligned in memory.
    // It certainly is in QEMU.
    next_ptr: *const u32,
    end_ptr: *const u32,

    phantom: PhantomData<&'a AcpiRsdt>,
}

/// Iterate over all the ACPI table entries.
impl<'a> Iterator for AcpiIterator<'a> {
    // Each item is actually a pointer to a whole ACPI table entry, starting with a header.
    type Item = (RsdtSig, U32Ptr<AcpiHeader>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_ptr >= self.end_ptr { return None; }

        // The entry pointers are misaligned. Have to handle this carefully.
        let entry_ptr = unsafe { self.next_ptr.read_unaligned() };

        let header_ptr = entry_ptr as usize as *const AcpiHeader;
        let sig_ptr = unsafe { ptr::addr_of!((*header_ptr).signature) };
        let sig = unsafe { *sig_ptr };

        self.next_ptr = unsafe { self.next_ptr.add(1) };

        Some((sig, U32Ptr::new(entry_ptr)))
    }
}

impl AcpiRsdt {
    fn iter(&self) -> AcpiIterator<'_> {
        assert!(self.header.length as usize >= size_of::<AcpiHeader>());

        // Divide by uint32_t explicitly as this is the size as mandated by the ACPI standard.
        let entries: u32 = (self.header.length - size_of::<AcpiHeader>() as u32) / size_of::<u32>() as u32;
        let base_ptr = &raw const self.entries as *const u32;

        AcpiIterator {
            next_ptr: base_ptr,
            end_ptr: unsafe { base_ptr.add(entries as usize) },
            phantom: PhantomData,
        }
    }

    // DEPARTURE: This doesn't exist in SeL4.
    // TODO: It'd be nice to make an iterator for this.
    #[unsafe(link_section = ".boot.text")]
    pub(crate) fn print_table_entries(&self) {
        for (sig, ptr) in self.iter() {
            let sig = core::str::from_utf8(sig.as_slice()).unwrap();
            kprintln!("RSDT entry at 0x{:x} with signature {}", ptr.0, sig);
        }
    }

    #[unsafe(link_section = ".boot.text")]
    pub(crate) fn madt_scan(&self) -> (FixedArr<Paddr, CONFIG_MAX_NUM_IOAPIC>, FixedArr<CpuId, CONFIG_MAX_NUM_NODES>) {
        let mut cpus = FixedArr::new();
        let mut ioapics = FixedArr::new();

        for (sig, ptr) in self.iter() {
            if sig != *b"APIC" { continue; }

            let madt_ptr = ptr.0 as *const AcpiMadt;

            /// The byte length of the entire madt region.
            let len = unsafe { madt_ptr.read_unaligned().header.length };

            kprintln!("ACPI: MADT paddr=0x{:x}", madt_ptr as usize);
            let apic_addr = unsafe { madt_ptr.read_unaligned().apic_addr };

            kprintln!("ACPI: apic_addr=0x{:x}", apic_addr);
            let flags = unsafe { madt_ptr.read_unaligned().flags };
            kprintln!("ACPI: flags=0x{:x}", flags);

            // The madt data is in a series of chunks starting right after the the addr and flags.
            let mut madt_entry_ptr = unsafe { madt_ptr.add(1) } as *const AcpiMadtHeader;
            let end_ptr = unsafe { madt_ptr.byte_offset(len as _) } as *const AcpiMadtHeader;

            while madt_entry_ptr < end_ptr {
                let madt_type = unsafe { *madt_entry_ptr }.madt_type;

                // kprintln!("MADT type {}", madt_type);

                match madt_type {
                    t if t == AcpiMadtStructType::APIC as u8 => {
                        // what Intel calls apic_id is what is called cpu_id in seL4!
                        // let cpu_id =
                        let apic_ptr = madt_entry_ptr as *const AcpiMadtApic;
                        let cpu_id = unsafe { (*apic_ptr).apic_id };
                        let flags = unsafe { (*apic_ptr).flags };
                        if flags == 1 {
                            kprintln!("ACPI: MADT_APIC apic_id=0x{:x}", cpu_id);

                            let result = cpus.try_push(cpu_id as CpuId);

                            if let Err(_) = result {
                                kprintln!("ACPI: Not recording this CPU (via APIC). Only configured to support {} cpus", cpus.len());
                            }
                        }
                    },

                    t if t == AcpiMadtStructType::X2APIC as u8 => {
                        // TODO! This doesn't show up in qemu, so I'm skipping it for now.
                        unimplemented!();
                    },

                    t if t == AcpiMadtStructType::IOAPIC as u8 => {
                        let ioapic_ptr = madt_entry_ptr as *const AcpiMadtIOApic;

                        let ioapic_id = unsafe { ioapic_ptr.read_unaligned().ioapic_id };
                        let ioapic_addr = unsafe { ioapic_ptr.read_unaligned().ioapic_addr };
                        let gsib = unsafe { ioapic_ptr.read_unaligned().gsib };

                        kprintln!("ACPI: MADT_IOAPIC ioapic_id={} ioapic_addr=0x{:x} gsib={}", ioapic_id, ioapic_addr, gsib);

                        let result = ioapics.try_push(ioapic_addr as usize);

                        if let Err(_) = result {
                            kprintln!("ACPI: Not recording this IOAPIC, only support {}", ioapics.len());
                        }
                    },

                    // Departure: We ignore ISOs. They're printed out in SeL4 but not used.

                    _ => {},
                }

                madt_entry_ptr = unsafe {
                    madt_entry_ptr.byte_offset((*madt_entry_ptr).length as isize)
                };
            }
        }

        (ioapics, cpus)
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

