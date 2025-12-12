# Multiboot trampoline

This sub crate acts as a little trampoline for launching the kernel via multiboot.

x86 chips can boot into a kernel using a bunch of different methods:

- Multiboot (used by grub and qemu with `-kernel` option)
- BIOS / MBR: The OG, old school booting process, which initializes the CPU in 16 bit mode. The kernel starts with nothing
- UEFI, which initializes some hardware and runs the kernel in 64 bit mode. UEFI binaries are passed some supporting structures, initialized by the UEFI bios.

Ideally, I'd really like to do something like cosmopolitan and have a single binary that magically works in all those settings. But failing that, I'd like to have different build modes which support all these different scenarios.

So for now, I'm just going to support multiboot. I'm splitting the kernel itself into 2 parts: The kernel (the main crate here) is simply a 64 bit binary, with nopic and a fixed entrypoint. Then I've got a multiboot loader stub & build toolchain which can embed (link) the kernel itself into a working multiboot kernel binary.
