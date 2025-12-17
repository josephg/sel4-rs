//! From SeL4 cpu_identification.c.

use core::arch::asm;
use core::arch::x86_64::__cpuid;
use core::slice;
use ufmt::derive::uDebug;
use crate::{const_assert, kpanic, kprintln, kwarnln};
use crate::config::CONFIG_KERNEL_SKIM_WINDOW;

#[derive(uDebug, Copy, Clone, Eq, PartialEq)]
pub enum CpuVendor {
    Intel,
    Amd,
    Other,
}

struct CpuInfo {
    // vendor: CpuVendor,

}

/// VendorInfo is a 12 byte long string stored in ebx, edx, ecx after performing cpuid(0).
#[repr(C)]
struct VendorInfo(u32, u32, u32);
const_assert!(size_of::<VendorInfo>() == 12);

impl VendorInfo {
    fn new() -> Self {
        let vendor_info = unsafe { __cpuid(0) };
        Self(vendor_info.ebx, vendor_info.edx, vendor_info.ecx)
    }

    fn as_slice(&self) -> &[u8] {
        let ptr = self as *const _ as *const u8;
        unsafe {
            slice::from_raw_parts(ptr, size_of::<VendorInfo>())
        }
    }

    fn as_str(&self) -> &str {
        let Ok(s) = (unsafe { core::str::from_utf8(self.as_slice()) }) else {
            kpanic!("CPU vendor string does not contain valid bytes");
        };
        s
    }

    // fn as_vendor(&self) -> CpuVendor {
    //     match self.as_slice() {
    //         b"GenuineIntel" => CpuVendor::Intel,
    //         b"AuthenticAMD" => CpuVendor::Amd,
    //         _ => CpuVendor::Other,
    //     }
    // }
}

/// Get the IA32_ARCH_CAPABILITIES MSR. In SeL4 this code is generated and typesafe, which is nice!
#[unsafe(link_section = ".boot.text")]
fn cpuid_007h_edx_get_ia32_arch_cap_msr(cpuid_007_edx: u32) -> bool {
    // The CAP_MSR is bit 29.
    let mask = 1u32 << 29;

    (cpuid_007_edx & mask) != 0
}

/// RDCL_NO: The processor is not susceptible to Rogue Data Cache Load (RDCL).
///
/// TODO: Make capabilities MSR into a NewType and add functions like this to read the bits out.
#[unsafe(link_section = ".boot.text")]
pub fn ia32_arch_caps_msr_get_rdcl_no(ia32_arch_caps_msr: u64) -> bool {
    (ia32_arch_caps_msr & 0x1) != 0
}

unsafe fn rdmsr(msr: u32) -> u64 {
    let (high, low): (u32, u32);
    unsafe {
        asm!("rdmsr",
            out("eax") low,
            out("edx") high,
            in("ecx") msr
        );
    }
    ((high as u64) << 32) | (low as u64)
}

const IA32_ARCH_CAPABILITIES_MSR: u32 = 0x10A;

/// Returns None if the CPU doesn't support capabilities MSR.
///
/// Documentation:
/// https://www.intel.com/content/www/us/en/developer/articles/technical/software-security-guidance/technical-documentation/cpuid-enumeration-and-architectural-msrs.html
pub fn read_ia32_arch_cap_msr() -> Option<u64> {
    let cpuid_007_edx = unsafe { __cpuid(0x7) }.edx;
    if cpuid_007h_edx_get_ia32_arch_cap_msr(cpuid_007_edx) {
        let msr = unsafe { rdmsr(IA32_ARCH_CAPABILITIES_MSR) };

        Some(msr)
    } else {
        None
    }
}

#[unsafe(link_section = ".boot.text")]
pub fn x86_cpuid_get_vendor() -> CpuVendor {
    // DEPARTURE: I've dramatically simplified this code. Hopefully, keeping the same behaviour.

    // The SeL4 code populates a static CPU info struct, and only uses it in one place. I'm just
    // going to combine all that behaviour together.
    match VendorInfo::new().as_slice() {
        // SeL4 only officially supports AMD and Intel CPUs.
        b"GenuineIntel" => CpuVendor::Intel,
        b"AuthenticAMD" => CpuVendor::Amd,
        slice => {
            let Ok(vendor_str) = (unsafe { core::str::from_utf8(slice) }) else {
                kpanic!("CPU vendor string does not contain valid bytes");
            };

            kwarnln!("Warning: Your x86 CPU has an unsupported vendor, '{}' \n\
                   \tYour setup may not be able to competently run seL4 as \
                   \tintended.\
                   \tCurrently supported x86 vendors are AMD and Intel.",
                   vendor_str);
            CpuVendor::Other
        },
    }
}
