//! This file contains helper code for x86 SMP support.
//!
//! SeL4 has a reasonably simple SMP model - all core kernel code only runs on a single core at a
//! time. Any kernel structures are wrapped in a mutex, and pull the associated data to the
//! executing core.

