/* Linker script for user-space programs */

ENTRY(_start)

SECTIONS
{
    /* User space starts at 0x400000 (standard for x86_64) */
    . = 0x400000;

    .text : ALIGN(4K)
    {
        *(.text.start)
        *(.text .text.*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    .bss : ALIGN(4K)
    {
        *(.bss .bss.*)
        *(COMMON)
    }

    /* Discard sections we don't need */
    /DISCARD/ :
    {
        *(.eh_frame)
        *(.note .note.*)
    }
}
