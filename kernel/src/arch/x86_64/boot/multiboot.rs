//! This file implements a simple multiboot header to make the kernel easy to boot from qemu and
//! grub.
//!
//! Currently only text mode is supported. At some point it'd be good to add framebuffer support.
//!
//! I also don't have a multiboot2 header here. Could add it when I work on bare metal support. But
//! for now, Qemu doesn't support multiboot2. So it won't add much.

use ufmt::derive::uDebug;
use crate::arch::x86_64::{CStr32, U32Ptr};
use crate::config::{ConfigGraphicsMode, CONFIG_MULTIBOOT_GRAPHICS_MODE};
use crate::const_assert;

#[repr(C)]
#[repr(align(8))]
struct MultibootHeader {
    /// Must be MULTIBOOT_HEADER_MAGIC
    magic: u32,
    /// Feature flags (see [MultibootFlags])
    flags: u32,
    checksum: u32,

    // The following fields only get read for non-ELF file kernels. (If AoutKludge flag is passed)
    binary_header_addr: u32,
    binary_load_addr: u32,
    binary_load_end_addr: u32,
    binary_bss_end_addr: u32,
    binary_entry_addr: u32,

    // Video info. These fields are only read if the VideoMode flag is passed.

    /// Contains 0 for linear graphics mode or 1 for EGA-standard text mode.
    /// Note that the boot loader may set a text mode even if this field contains ‘0’, or set a
    /// video mode even if this field contains ‘1’.
    graphics_mode_type: u32,
    /// Requested number of columns. 0 for no preference.
    graphics_width: u32,
    /// Requested number of lines. 0 for no preference.
    graphics_height: u32,
    /// Requested number of bits per pixel in graphics mode. 0 for no preference.
    graphics_depth: u32,
}

// This would be nicer using bitflags crate.
#[derive(uDebug, Copy, Clone)]
#[repr(u32)]
#[allow(unused)]
enum MultibootFlags {
    /// Align all boot modules on i386 page (4KB) boundaries.
    AlignModules = 0x1,
    /// Must pass memory information to OS.
    MemoryInfo = 0x2,
    /// Must pass video information to OS.
    VideoMode = 0x4,
    /// This flag indicates the use of the address fields in the header. (Name taken from docs)
    AoutKludge = 0x0001_0000,
}

const MULTIBOOT_HEADER_MAGIC: u32 = 0x1BADB002;

/// Returns (flag, mode).
const fn gfx_flags() -> (u32, u32) {
    match CONFIG_MULTIBOOT_GRAPHICS_MODE {
        ConfigGraphicsMode::None => (0, 0),
        ConfigGraphicsMode::Text => (MultibootFlags::VideoMode as u32, 0),
        ConfigGraphicsMode::Linear => (MultibootFlags::VideoMode as u32, 1),
    }
}

const FLAGS: u32 = (MultibootFlags::AlignModules as u32)
    | (MultibootFlags::MemoryInfo as u32)
    | gfx_flags().0;

/// This is linked as static data within the binary so multiboot knows how to boot our kernel.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".mbh")]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: MULTIBOOT_HEADER_MAGIC,
    flags: FLAGS,
    checksum: u32::MAX - FLAGS - MULTIBOOT_HEADER_MAGIC + 1,

    binary_header_addr: 0,
    binary_load_addr: 0,
    binary_load_end_addr: 0,
    binary_bss_end_addr: 0,
    binary_entry_addr: 0,

    graphics_mode_type: gfx_flags().1,
    graphics_width: 0,
    graphics_height: 0,
    graphics_depth: 0,
};




#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct MultibootSlice<T> {
    pub len: u32,
    pub addr: U32Ptr<T>,

    // Could add PhantomData of type T to make coercing more safe?
}

impl<T> MultibootSlice<T> {
    /// SAFETY: This is only safe if all 3 parameters (len, addr and type) are valid.
    ///
    /// This function takes a container object as a parameter. The lifetime of the container object
    /// is used as the lifetime of the returned cstr.
    pub unsafe fn to_slice<P>(self, _container: &P) -> &[T] {
        let ptr = self.addr.as_ptr();
        unsafe {
            core::slice::from_raw_parts(ptr, self.len as usize)
        }
    }
}

const_assert!(MULTIBOOT_HEADER.checksum.wrapping_add(FLAGS + MULTIBOOT_HEADER_MAGIC) == 0);

/// *** The info we get *back* from multiboot at boot time. ***
///
/// Based on C structs: https://www.gnu.org/software/grub/manual/multiboot/multiboot.html

pub const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;

// Commented out = unused by sel4.
#[derive(uDebug, Copy, Clone)]
#[repr(u32)]
#[allow(unused)]
pub(crate) enum MultibootInfoFlags {
    /// is there basic lower/upper memory information?
    Memory = 0x1,
    // /// is there a boot device set?
    // BootDev = 0x2,
    /// is the command-line defined?
    CmdLine = 0x4,
    /// are there modules to do something with?
    Mods = 0x8,

    // /// is there a symbol table loaded? (Mutually exclusive with [MultibootInfoFlags::ElfShdr])
    // AoutSyms = 0x10,
    // /// is there an ELF section header table? (Mutually exclusive with [MultibootInfoFlags::AoutSyms])
    // ElfShdr = 0x20,
    /// is there a full memory map?
    MemMap = 0x40,
    // /// Is there drive info?
    // DriveInfo = 0x80,
    // /// Is there a config table?
    // ConfigTable = 0x100,
    // /// Is there a boot loader name?
    // BootLoaderName = 0x200,
    // /// Is there a APM table?
    // APMTable = 0x400,
    /// Is there video information?
    VBEInfo = 0x800,
    FramebufferInfo = 0x1000,
}

/// Some slices in the struct use a count of the items. Some use the byte length. Bleh.
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct MultibootByteLenSlice<T> {
    pub byte_len: u32,
    pub addr: U32Ptr<T>,

    // Could add PhantomData of type T to make coercing more safe?
}

impl<T> MultibootByteLenSlice<T> {
    /// SAFETY: This is only safe if all 3 parameters (len, addr and type) are valid.
    ///
    /// This function takes a container object as a parameter. The lifetime of the container object
    /// is used as the lifetime of the returned cstr.
    pub unsafe fn to_slice<P>(self, _container: &P) -> &[T] {
        let ptr = self.addr.as_ptr();
        unsafe {
            core::slice::from_raw_parts(ptr, self.byte_len as usize / size_of::<T>())
        }
    }
}


// The symbol table information is unused. We could parse it anyway, but then we'd need to deal with
// unions. And ditching it allows the multiboot info to impl Debug.

// #[derive(uDebug, Copy, Clone)]
// #[repr(C)]
// struct AOutSymbolTable {
//     tab_size: u32,
//     str_size: u32,
//     addr: MultibootPtr,
//     reserved: u32,
// }
//
//
// #[derive(uDebug, Copy, Clone)]
// #[repr(C)]
// struct ElfSectionHeaderTable {
//     num: u32,
//     size: u32,
//     addr: MultibootPtr,
//     shndx: u32,
// }
//
// #[repr(C)]
// pub union MultibootBinaryTable {
//     aout: AOutSymbolTable,
//     elf: ElfSectionHeaderTable,
// }

/// This struct is passed to the kernel at boot time when we boot with multiboot2.
#[repr(C)]
pub(crate) struct MultibootBootInfo {
    /// [MultibootInfoFlags]
    pub flags: u32,

    /// Available memory from bios
    pub mem_lower: u32,
    pub mem_upper: u32,

    /// "Root" partition
    pub boot_device: u32,
    /// Kernel command line
    pub cmdline: CStr32,

    /// Boot module list
    pub mods: MultibootSlice<MultibootModule>,

    // /// Information about the loaded kernel. The multiboot loader partially parses the ELF headers
    // /// in order to load the kernel into memory and boot into it. This table contains this data.
    // pub bin_table: MultibootBinaryTable,

    // Unused symbol table information.
    _syms: [u32; 4],

    /// Memory mapping buffer. According to the multiboot spec, the mmap table has entries of
    /// arbitrary size. So this requires some care in parsing.
    // pub mmap: MultibootByteLenSlice<MMapEntry>,
    pub mmap_bytelength: u32,
    pub mmap_addr: U32Ptr<MMapEntry>,

    /// Drive info buffer
    pub drives: MultibootSlice<()>,

    /// ROM configuration table
    pub config_table: U32Ptr<()>,

    /// Boot loader name
    pub boot_loader_name: U32Ptr<()>,

    /// APM table
    pub apm_table: U32Ptr<()>,

    // The following fields could probably be more cleanly broken into their own structs, but
    // doing it like this matches the definition in the multiboot spec.

    // Video
    pub vbe_control_info: U32Ptr<()>,
    pub vbe_mode_info: U32Ptr<()>,
    pub vbe_mode: u16,
    pub vbe_interface_seg: u16,
    pub vbe_interface_off: u16,
    pub vbe_interface_len: u16,

    // Framebuffer. Unused by sel4.
    // pub framebuffer_addr: u64,
    // pub framebuffer_pitch: u32,
    // pub framebuffer_width: u32,
    // pub framebuffer_height: u32,
    // pub framebuffer_bpp: u8,
    // pub framebuffer_type: u8,
    // pub framebuffer_color_info: FramebufferColorInfo,
}

// #[derive(uDebug, Copy, Clone)]
// #[repr(C)]
// struct MultibootColor {
//     red: u8,
//     green: u8,
//     blue: u8,
// }

// #[derive(uDebug, Copy, Clone)]
// #[repr(C)]
// union FramebufferColorInfo {
//     palette: PaletteInfo,
//     rgb: RgbInfo,
// }
//
// #[derive(uDebug, Copy, Clone)]
// #[repr(C)]
// struct PaletteInfo {
//     framebuffer_palette_addr: MultibootPtr,
//     framebuffer_palette_num_colors: u16,
// }

// #[derive(uDebug, Copy, Clone)]
// #[repr(C)]
// struct RgbInfo {
//     red_field_position: u8,
//     red_mask_size: u8,
//     green_field_position: u8,
//     green_mask_size: u8,
//     blue_field_position: u8,
//     blue_mask_size: u8,
// }

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub(crate) struct MMapEntry {
    pub size: u32,
    pub base_addr: u64,
    pub len: u64,
    pub mtype: u32,
}

#[derive(uDebug, Copy, Clone)]
#[repr(u32)]
pub enum MMapType {
    Usable = 1,
    Reserved = 2,
    Acpi = 3,
    /// Reserved memory which needs to be preserved on hibernation
    AcpiNvs = 4,
    /// Memory occupied by defective RAM modules
    Bad = 5,
}

#[derive(uDebug, Copy, Clone)]
#[repr(C)]
pub(crate) struct MultibootModule {
    /// the memory used goes from bytes ’mod_start’ to ’mod_end-1’ inclusive
    pub mod_start: u32,
    pub mod_end: u32,
    /// Module command line
    pub name: CStr32,
    /// padding to take it to 16 bytes (must be zero)
    _pad: u32,
}
