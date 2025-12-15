//! This file implements a simple multiboot header to make the kernel easy to boot from qemu and
//! grub.
//!
//! Currently only text mode is supported. At some point it'd be good to add framebuffer support.
//!
//! I also don't have a multiboot2 header here. Could add it when I work on bare metal support. But
//! for now, Qemu doesn't support multiboot2. So it won't add much.

use ufmt::derive::uDebug;

#[repr(C)]
#[repr(align(8))]
struct MultibootHeader {
    /// Must be MULTIBOOT_HEADER_MAGIC
    magic: u32,
    /// Feature flags (see [MultibootFlags])
    flags: u32,
    checksum: u32,
    padding: u32,

    // This struct has more headers if AOutKludge or VideoMode is passed.
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
const FLAGS: u32 = (MultibootFlags::AlignModules as u32)
    | (MultibootFlags::MemoryInfo as u32);


/// This is linked as static data within the binary so multiboot knows how to boot our kernel.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".mbh")]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: MULTIBOOT_HEADER_MAGIC,
    flags: FLAGS,
    checksum: u32::MAX - FLAGS - MULTIBOOT_HEADER_MAGIC + 1,
    padding: 0,
};



// TODO: Hoist this into a utils library or something.
macro_rules! const_assert {
    ($condition:expr) => {
        #[allow(unknown_lints, clippy::eq_op)]
        const _: () = assert!($condition);
    };
}

const_assert!(MULTIBOOT_HEADER.checksum.wrapping_add(FLAGS + MULTIBOOT_HEADER_MAGIC) == 0);

/// *** The info we get *back* from multiboot at boot time. ***
///
/// Based on C structs: https://www.gnu.org/software/grub/manual/multiboot/multiboot.html

pub const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;

#[derive(uDebug, Copy, Clone)]
#[repr(u32)]
enum MultibootInfoFlags {
    /// is there basic lower/upper memory information?
    Memory = 0x1,
    /// is there a boot device set?
    BootDev = 0x2,
    /// is the command-line defined?
    CmdLine = 0x4,
    /// are there modules to do something with?
    Mods = 0x8,

    /// is there a symbol table loaded? (Mutually exclusive with [MultibootInfoFlags::ElfShdr])
    AoutSyms = 0x10,
    /// is there an ELF section header table? (Mutually exclusive with [MultibootInfoFlags::AoutSyms])
    ElfShdr = 0x20,
    /// is there a full memory map?
    MemMap = 0x40,
    /// Is there drive info?
    DriveInfo = 0x80,
    /// Is there a config table?
    ConfigTable = 0x100,
    /// Is there a boot loader name?
    BootLoaderName = 0x200,
    /// Is there a APM table?
    APMTable = 0x400,
    /// Is there video information?
    VBEInfo = 0x800,
    FramebufferInfo = 0x1000,
}

// TODO: Probably better to make a newtype for this
type MultibootPtr = u32;

#[derive(uDebug, Copy, Clone)]
#[repr(C)]
struct AOutSymbolTable {
    tab_size: u32,
    str_size: u32,
    addr: MultibootPtr,
    reserved: u32,
}


#[derive(uDebug, Copy, Clone)]
#[repr(C)]
struct ElfSectionHeaderTable {
    num: u32,
    size: u32,
    addr: MultibootPtr,
    shndx: u32,
}

#[repr(C)]
union BinaryTable {
    aout: AOutSymbolTable,
    elf: ElfSectionHeaderTable,
}

#[repr(C)]
struct MultibootSlice {
    len: u32,
    addr: MultibootPtr,
}

/// This struct is passed to the kernel at boot time when we boot with multiboot2.
#[repr(C)]
pub(crate) struct MultibootBootInfo {
    /// [MultibootInfoFlags]
    flags: u32,

    /// Available memory from bios
    mem: MultibootSlice,

    /// "Root" partition
    boot_device: u32,
    /// Kernel command line
    cmdline: u32,

    /// Boot module list
    mods: MultibootSlice,

    /// Information about the loaded kernel. The multiboot loader partially parses the ELF headers
    /// in order to load the kernel into memory and boot into it. This table contains this data.
    bin_table: BinaryTable,

    /// Memory mapping buffer
    mmap: MultibootSlice,

    /// Drive info buffer
    drives: MultibootSlice,

    /// ROM configuration table
    config_table: u32, // ptr?

    /// Boot loader name
    boot_loader_name: u32, // Ptr?

    /// APM table
    apm_table: u32, // ptr?

    // Video stuff follows... Not converted from below.
}


/*

struct multiboot_info
{
  /* Multiboot info version number */
  multiboot_uint32_t flags;

  /* Available memory from BIOS */
  multiboot_uint32_t mem_lower;
  multiboot_uint32_t mem_upper;

  /* "root" partition */
  multiboot_uint32_t boot_device;

  /* Kernel command line */
  multiboot_uint32_t cmdline;

  /* Boot-Module list */
  multiboot_uint32_t mods_count;
  multiboot_uint32_t mods_addr;

  union
  {
    multiboot_aout_symbol_table_t aout_sym;
    multiboot_elf_section_header_table_t elf_sec;
  } u;

  /* Memory Mapping buffer */
  multiboot_uint32_t mmap_length;
  multiboot_uint32_t mmap_addr;

  /* Drive Info buffer */
  multiboot_uint32_t drives_length;
  multiboot_uint32_t drives_addr;

  /* ROM configuration table */
  multiboot_uint32_t config_table;

  /* Boot Loader Name */
  multiboot_uint32_t boot_loader_name;

  /* APM table */
  multiboot_uint32_t apm_table;

  /* Video */
  multiboot_uint32_t vbe_control_info;
  multiboot_uint32_t vbe_mode_info;
  multiboot_uint16_t vbe_mode;
  multiboot_uint16_t vbe_interface_seg;
  multiboot_uint16_t vbe_interface_off;
  multiboot_uint16_t vbe_interface_len;

  multiboot_uint64_t framebuffer_addr;
  multiboot_uint32_t framebuffer_pitch;
  multiboot_uint32_t framebuffer_width;
  multiboot_uint32_t framebuffer_height;
  multiboot_uint8_t framebuffer_bpp;
#define MULTIBOOT_FRAMEBUFFER_TYPE_INDEXED 0
#define MULTIBOOT_FRAMEBUFFER_TYPE_RGB     1
#define MULTIBOOT_FRAMEBUFFER_TYPE_EGA_TEXT     2
  multiboot_uint8_t framebuffer_type;
  union
  {
    struct
    {
      multiboot_uint32_t framebuffer_palette_addr;
      multiboot_uint16_t framebuffer_palette_num_colors;
    };
    struct
    {
      multiboot_uint8_t framebuffer_red_field_position;
      multiboot_uint8_t framebuffer_red_mask_size;
      multiboot_uint8_t framebuffer_green_field_position;
      multiboot_uint8_t framebuffer_green_mask_size;
      multiboot_uint8_t framebuffer_blue_field_position;
      multiboot_uint8_t framebuffer_blue_mask_size;
    };
  };
};
typedef struct multiboot_info multiboot_info_t;

struct multiboot_color
{
  multiboot_uint8_t red;
  multiboot_uint8_t green;
  multiboot_uint8_t blue;
};

struct multiboot_mmap_entry
{
  multiboot_uint32_t size;
  multiboot_uint64_t addr;
  multiboot_uint64_t len;
#define MULTIBOOT_MEMORY_AVAILABLE              1
#define MULTIBOOT_MEMORY_RESERVED               2
#define MULTIBOOT_MEMORY_ACPI_RECLAIMABLE       3
#define MULTIBOOT_MEMORY_NVS                    4
#define MULTIBOOT_MEMORY_BADRAM                 5
  multiboot_uint32_t type;
} __attribute__((packed));
typedef struct multiboot_mmap_entry multiboot_memory_map_t;

struct multiboot_mod_list
{
  /* the memory used goes from bytes ’mod_start’ to ’mod_end-1’ inclusive */
  multiboot_uint32_t mod_start;
  multiboot_uint32_t mod_end;

  /* Module command line */
  multiboot_uint32_t cmdline;

  /* padding to take it to 16 bytes (must be zero) */
  multiboot_uint32_t pad;
};
 */