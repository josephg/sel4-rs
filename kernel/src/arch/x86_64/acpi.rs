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

const ACPI_V1_SIZE: usize = 20;
const ACPI_V2_SIZE: usize = 36;

/// Generic System Descriptor Table Header
#[repr(C, packed(4))]
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

/// ACPI Root System Descriptor Pointer
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub(crate) struct AcpiRsdp {
    /// "RSD PTR "
    signature: [u8; 8],
    /// Number chosen such that the first 20 bytes of the table add to 0
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    /// Deprecated since version 2, but still part of the checksum.
    rsdt_address: U32Ptr<AcpiRsdt>,

    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,

    _reserved: [u8; 3],
}

const_assert!(size_of::<AcpiRsdp>() == ACPI_V2_SIZE);

struct AcpiRsdt {
    header: AcpiHeader,
    // TODO: Replace this u32 with a U32Ptr.
    entry: u32,
}

#[unsafe(link_section = ".boot.text")]
fn acpi_calc_checksum<T>(obj: &T) -> u8 {
    let mut checksum: u8 = 0;
    let slice = unsafe {
        slice::from_raw_parts(obj as *const _ as *const u8, size_of::<T>())
    };

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

/// Scan the bios memory for the ACPI RSDP.
#[unsafe(link_section = ".boot.text")]
fn acpi_get_rsdp() -> Option<U32Ptr<AcpiRsdp>> {
    // The ACPI RSDP is somewhere in memory in the main memory area below 1MB. It will always be
    // aligned on a 16 byte boundary. To find it, we scan the BIOS memory region looking for
    // something that has the signature "RSD PTR " and a valid checksum.
    //
    // Note that we scan each 16 byte aligned position and read 20 bytes forward. The core::slice
    // chunking methods unfortunately only have .windows() and .chunks(), but not something that
    // supports this.

    // This is kind of ugly. I'd much rather make a slice across the whole memory range then
    // read it correctly. But this should be correct and fast.
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
fn validate_rsdp(rsdp: &AcpiRsdp) -> Result<(), ()> {
    // DEPARTURE: This function re-checks the base ACPI checksum here. An invalid checksum should be
    // impossible since the pointer is unmodified since the call to acpi_get_rsdp.

    if rsdp.revision > 0 {
        // Check the extended checksum is also valid.
        if !checksum_valid(rsdp) {
            kprintln!("BIOS: ACPIv2 information corrupt!");
            return Err(());
        }
    }

    // Verify the RSDT as well, even though we don't actually use the RSDT within SeL4.
    assert!(rsdp.rsdt_address.not_null(), "RSDT pointer is null");

    kprintln!("BIOS: RSDT paddr=0x{:x}", rsdp.rsdt_address.0);
    let rsdt = unsafe { rsdp.rsdt_address.as_static_ref() };

    if !checksum_valid(rsdt) {
        kprintln!("ACPI: RSDT checksum failure");
        return Err(());
    }

    Ok(())
}

#[unsafe(link_section = ".boot.text")]
pub(super) fn acpi_init() -> Result<AcpiRsdp, ()> {
    let Some(rsdp_ptr) = acpi_get_rsdp() else {
        kprintln!("BIOS: No ACPI support detected!");
        return Err(());
    };

    kprintln!("ACPI: RSDP paddr=0x{:x}", rsdp_ptr.0);

    let rsdp = unsafe { rsdp_ptr.as_static_ref() };
    validate_rsdp(rsdp)?;

    // DEPARTURE: SeL4 calls acpi_table_init here to make sure the ACPI table is correctly mapped
    // in to memory. But this is unnecessary in 64 bit mode, where the entire lower 32 bit range
    // is identity mapped already.

    Ok(*rsdp)
}