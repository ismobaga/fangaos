/* kernel/linker.ld */
OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

SECTIONS
{
  /* Higher half kernel base (classic) */
  . = 0xffffffff80000000;

  /* --- Limine requests (MUST be kept) --- */
  .limine_requests : ALIGN(8) {
    KEEP(*(.limine_requests_start))
    KEEP(*(.limine_requests))
    KEEP(*(.limine_requests_end))
  }

  /* --- Code --- */
  .text : ALIGN(4K) {
    *(.text .text.*)
  }

  /* --- Read-only data --- */
  .rodata : ALIGN(4K) {
    *(.rodata .rodata.*)
  }

  /* --- Data --- */
  .data : ALIGN(4K) {
    *(.data .data.*)
  }

  /* --- BSS --- */
  .bss : ALIGN(4K) {
    *(COMMON)
    *(.bss .bss.*)
  }

  /DISCARD/ : {
    *(.eh_frame*)
    *(.comment*)
  }
}

