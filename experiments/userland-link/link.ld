ENTRY(user_probe_entry)
PHDRS { text PT_LOAD FLAGS(5); rodata PT_LOAD FLAGS(4); data PT_LOAD FLAGS(6); }
SECTIONS {
  . = 0x400000;
  .text : ALIGN(4096) { KEEP(*(.text.user_probe_entry)) *(.text .text.*) } :text
  .rodata : ALIGN(4096) { *(.rodata .rodata.*) } :rodata
  .data : ALIGN(4096) { *(.data .data.*) } :data
  .bss : ALIGN(4096) { *(.bss .bss.*) *(COMMON) } :data
  /DISCARD/ : { *(.eh_frame*) *(.comment*) }
}
