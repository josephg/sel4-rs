/*
 * Copyright 2014, General Dynamics C4 Systems
 *
 * SPDX-License-Identifier: GPL-2.0-only
 */

//#define __ASSEMBLER__
//#include <config.h>
//#include <hardware.h>

ENTRY(_start)

#define USER_TOP UL_CONST(0x7FFFFFFFFFFF)
#define PADDR_BASE UL_CONST(0x00000000)
#define PPTR_BASE UL_CONST(0xffffff8000000000)
#define TLBBITMAP_PML4_RESERVED (TLBBITMAP_ROOT_ENTRIES * BIT(PML4_INDEX_OFFSET))
#define TLBBITMAP_PPTR (PPTR_BASE - TLBBITMAP_PML4_RESERVED)
#define PPTR_TOP UL_CONST(0xffffffff80000000)
#define KERNEL_ELF_PADDR_BASE UL_CONST(0x00100000)
#define KERNEL_ELF_PADDR_BASE_RAW KERNEL_ELF_PADDR_BASE

#define KERNEL_ELF_BASE (PPTR_TOP + KERNEL_ELF_PADDR_BASE)
#define KERNEL_ELF_BASE_RAW (PPTR_TOP + KERNEL_ELF_PADDR_BASE_RAW)

KLOAD_PADDR = KERNEL_ELF_PADDR_BASE_RAW;
KLOAD_VADDR = KERNEL_ELF_BASE_RAW;

/* WARNING: constants also defined in plat/machine/hardware.h */
//OUTPUT_FORMAT(elf64-x86-64)
//#if defined(ENABLE_SMP_SUPPORT) && defined(CONFIG_KERNEL_SKIM_WINDOW)
//nodeSkimScratchOffset = nodeSkimScratch - node_info;
//#endif

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
        *(.mbh)
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
    . = . + ((ADDR(.phys) + SIZEOF(.phys)) & (4K - 1));
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
#ifdef CONFIG_ARCH_IA32
    . = ALIGN(4M);
#else
    . = ALIGN(2M);
#endif
#else
    . = ALIGN(8K);
#endif

    ki_boot_end = .;
    ki_skim_start = .;

    . = . + ((ADDR(.boot) + SIZEOF(.boot)) & (8K - 1));



    .text . : AT(ADDR(.text) - KERNEL_OFFSET)
    {
        *(.text)
    } :virt

    .rodata . : AT(ADDR(.rodata) - KERNEL_OFFSET)
    {
        *(.rodata)
        *(.rodata.*)
    } :virt

    .skim_data . : AT(ADDR(.skim_data) - KERNEL_OFFSET)
    {
        *(.skim.data)
        *(.skim.data.*)
    } :virt


    .skim_bss . (NOLOAD) : AT(ADDR(.skim_bss) - KERNEL_OFFSET)
    {
        *(.skim.bss)
        *(.skim.bss.*)
    } :virt

#ifdef CONFIG_KERNEL_SKIM_WINDOW
    /* Align up so that the SKIM portion of the kernel is by itself
       on large pages */
#ifdef CONFIG_ARCH_IA32
    . = ALIGN(4M);
#else
    . = ALIGN(2M);
#endif
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

    ._idle_thread . (NOLOAD): AT(ADDR(._idle_thread) - KERNEL_OFFSET)
    {
	__idle_thread_start = .;
        *(._idle_thread)
	__idle_thread_end = .;
    } :virt

    . = ALIGN(4K);
    ki_end = .;

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.note.gnu.build-id)
        *(.comment)
    }
}
