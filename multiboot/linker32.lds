ENTRY(_start)

USER_TOP = 0x7FFFFFFFFFFF;
PADDR_BASE = 0x00000000;
PPTR_BASE = 0xffffff8000000000;
PPTR_TOP = 0xffffffff80000000;
KERNEL_ELF_PADDR_BASE = 0x00100000;
KERNEL_ELF_PADDR_BASE_RAW = KERNEL_ELF_PADDR_BASE;

KERNEL_ELF_BASE =  (PPTR_TOP + KERNEL_ELF_PADDR_BASE);
KERNEL_ELF_BASE_RAW =  (PPTR_TOP + KERNEL_ELF_PADDR_BASE_RAW);


KLOAD_PADDR = KERNEL_ELF_PADDR_BASE_RAW;
KLOAD_VADDR = KERNEL_ELF_BASE_RAW;

KERNEL_OFFSET = KLOAD_VADDR - KLOAD_PADDR;

PHDRS {
    phys PT_LOAD FILEHDR PHDRS ;
    boot PT_LOAD ;
    virt PT_LOAD ;
}


SECTIONS
{
    /* load kernel to 1M to avoid the famous IA-32 memory holes below */
    . = KLOAD_PADDR + SIZEOF_HEADERS;


    /* code/data only needed during bootstrapping, linked to physical addresses */

    .phys . :
    {
        KEEP(*(.mbh))
        *(.phys.text)
        *(.phys.data)
        . = . + 1;
    } :phys

    .phys.bss ADDR(.phys) + SIZEOF(.phys) + 4K (NOLOAD) :
    {
        . = ALIGN(16);
        boot_stack_bottom = .;
        . = . + 2K;
        boot_stack_top = .;
        *(.phys.bss)
    } :phys

    . = ALIGN(4K) + KERNEL_OFFSET;
    /* Make the masking math explicitly absolute so lld is happy. */
    . = . + (ABSOLUTE(ADDR(.phys) + SIZEOF(.phys)) & (4K - 1));
    .boot . : AT(ADDR(.boot) - KERNEL_OFFSET)
    {
        *(.boot.text)
        *(.boot.data)
    } :boot

    .boot.bss . (NOLOAD) : AT(ADDR(.boot.bss) - KERNEL_OFFSET)
    {
        *(.boot.bss)
    } :boot


#ifdef CONFIG_KERNEL_SKIM_WINDOW
    /* Align up so that the SKIM portion of the kernel is by itself
       on large pages */
    . = ALIGN(2M);
#else
    . = ALIGN(8K);
#endif


    . = . + (ABSOLUTE(ADDR(.boot) + SIZEOF(.boot)) & (8K - 1));



    .text . : AT(ADDR(.text) - KERNEL_OFFSET)
    {
        *(.text)
    } :virt

    .rodata . : AT(ADDR(.rodata) - KERNEL_OFFSET)
    {
        *(.rodata)
        *(.rodata.*)
    } :virt


#ifdef CONFIG_KERNEL_SKIM_WINDOW
    /* Align up so that the SKIM portion of the kernel is by itself
       on large pages */
    . = ALIGN(2M);
#else
    . = ALIGN(8K);
#endif

    ki_skim_end = .;



    .data . : AT(ADDR(.data) - KERNEL_OFFSET)
    {
        *(.data)
    } :virt

    .bss . (NOLOAD) : AT(ADDR(.bss) - KERNEL_OFFSET)
    {
        *(.bss)
        *(COMMON) /* fallback in case '-fno-common' is not used */
    } :virt

    . = ALIGN(4K);

    . = ALIGN(4K);
    ki_end = .;


    /DISCARD/ :
    {
        *(.eh_frame)
        *(.note.gnu.build-id)
        *(.comment)
    }
}
