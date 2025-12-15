
// These are all defined in include/arch/x86/arch/types.h in sel4.
// When adding different architectures, check how these all match up.



// TODO: Consider wrapping some of these in Newtype.

use ufmt::derive::uDebug;

/// A user-virtual address
pub type VirtPtr = usize;

/// Physical address
pub type Paddr = usize;

pub type Pptr = usize;
/// Capability pointer
pub type Cptr = usize;
pub type DevId = usize;
pub type CpuId = usize;
pub type LogicalId = u32;
pub type NodeId = usize;
/// dom_t
pub type Domain = usize;

pub type Timestamp = u64;

// From basic_types.h


/**
 * A region [start..end) of kernel-virtual memory.
 *
 * Empty when start == end. If end < start, the region wraps around, that is,
 * it represents the addresses in the set \[start..-1\] union \[0..end). This is
 * possible after address translation and fine for e.g. device memory regions.
 */
#[derive(uDebug, Default, Copy, Clone)]
pub struct Region {
    pub start: Pptr,
    pub end: Pptr,
}

/** A region [start..end) of physical memory addresses. */
#[derive(uDebug, Default, Copy, Clone)]
pub struct PhysRegion {
    pub start: Paddr,
    pub end: Paddr,
}

/** A region [start..end) of user-virtual addresses. */
#[derive(uDebug, Default, Copy, Clone)]
pub struct VirtRegion {
    pub start: VirtPtr,
    pub end: VirtPtr,
}

