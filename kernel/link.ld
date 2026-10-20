ENTRY(kernel_entry)
PHDRS { text PT_LOAD FLAGS(5); rodata PT_LOAD FLAGS(4); data PT_LOAD FLAGS(6); }
SECTIONS {
  . = 0xffffffff80000000;
  __image_start = .;
  .text : ALIGN(4096) { KEEP(*(.text.kernel_entry)) *(.text .text.*) } :text
  . = ALIGN(4096);
  __text_end = .;
  .rodata : ALIGN(4096) { *(.rodata .rodata.*) } :rodata
  . = ALIGN(4096);
  __rodata_end = .;
  .limine_requests : ALIGN(4096) {
    KEEP(*(.limine_requests_start))
    KEEP(*(.limine_requests))
    KEEP(*(.limine_requests_end))
  } :data
  .data : ALIGN(4096) { *(.data .data.*) *(.got .got.*) } :data
  .bss : ALIGN(4096) { *(.bss .bss.*) *(COMMON) . = ALIGN(4096); } :data
  __image_end = .;
  /DISCARD/ : { *(.eh_frame*) *(.comment*) }
}
