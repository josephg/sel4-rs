cargo build -p kernel && objcopy -O elf32-i386 target/x86_64-unknown-none/debug/kernel kernel.elf

echo 'Ctrl+A, X to terminate QEMU'
qemu-system-x86_64 -enable-kvm -cpu host -serial mon:stdio -m size=512M -kernel kernel.elf  -no-reboot -d cpu_reset -d int
