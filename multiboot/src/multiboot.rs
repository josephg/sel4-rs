#[repr(C)]
#[repr(align(8))]
struct MultibootHeader {
    magic: i32,
    flags: i32,
    checksum: i32,
    padding: u32,
}

enum MultibootFlags {
    AlignModules = 1,
    MemoryInfo = 1 << 1,
}

const MULTIBOOT_MAGIC: i32 = 0x1BADB002;
const FLAGS: i32 = (MultibootFlags::AlignModules as i32) | (MultibootFlags::MemoryInfo as i32);
const CHECKSUM: i32 = -(MULTIBOOT_MAGIC + FLAGS);

#[unsafe(no_mangle)]
#[unsafe(link_section = ".mbh")]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: MULTIBOOT_MAGIC,
    flags: FLAGS,
    checksum: CHECKSUM,
    padding: 0,
};
