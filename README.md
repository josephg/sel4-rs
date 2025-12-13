# Sel4 in Rust

This is a little attempt at porting the SeL4 kernel to rust. Its mostly a learning exercise, but I hope to make this into a capable little kernel implementation.

This is experimental code. It is not feature complete. If you want SeL4 you should use SeL4. And given SeL4's proofs, rust doesn't really bring anything to the table here. So just go use it. Go! What are you still doing here??

## Status

Working:

- x86_64 initial boot via multiboot
- Rust code running in 64 bit mode via qemu

Todo:

- Setup running page table
- Capabilities
- Scheduler
- Syscall API
- Sel4 tests
- Proper kernel debugging support

Goals:

- Feature complete, able to run sel4 binaries unmodified
- Performance parity with sel4 on the same hardware
- UEFI, BIOS, USB and netboot targets
- Deeper support for monitoring than sel4 provides
- Root task with supervisor trees, and some ported drivers from freebsd / genode. Specifically:
  - Network stack
  - Graphics drivers
  - Power management
  - USB
  - A filesystem of some sort (tbd)
- Simple GUI applications running on top
- Tiny binary. (I'm probably going to write some of my own build tooling)

Non goals:

- Support for hardware platforms other than x86_64. Arm support would be nice at some point but its not a priority. 
- Support for all of SeL4's configuration options. This project will be more opinionated than sel4. We require a "modern" CPU (for some definition of modern - like cpus sold in the last 10+ years). Specifically, right now I'm requiring silicon support for:
  - `syscall`
  - Large pages
  - Long mode (64 bit support)
  - `invpcid`
  - Probably others.
- Linux syscall emulation and support


# License

As this code is heavily based on SeL4, the kernel itself is distributed under the same GPLv2 license. [See SeL4 license page for details](https://sel4.systems/Legal/license.html).

All associated userland is distributed under the ISC license.
