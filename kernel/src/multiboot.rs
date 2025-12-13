//! This file implements a simple multiboot header to make the kernel easy to boot from qemu and
//! grub.
//!
//! Currently only text mode is supported. At some point it'd be good to add framebuffer support.
//!
//! I also don't have a multiboot2 header here. Could add it when I work on bare metal support. But
//! for now, Qemu doesn't support multiboot2. So it won't add much.

#[repr(C)]
#[repr(align(8))]
struct MultibootHeader {
    magic: i32,
    flags: i32,
    checksum: i32,
    padding: u32,
}

enum MultibootFlags {
    AlignModules = 1 << 0,
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
