ENTRY(fixture_entry)
PHDRS { text PT_LOAD FLAGS(5); rodata PT_LOAD FLAGS(4); data PT_LOAD FLAGS(6); }
SECTIONS {
 . = 0x400000;
 .text : ALIGN(4096) { KEEP(*(.text.fixture_entry)) *(.text .text.*) } :text
 .rodata : ALIGN(4096) { KEEP(*(.rodata .rodata.*)) } :rodata
 .data : ALIGN(4096) { KEEP(*(.data .data.*)) *(.got .got.*) } :data
 .bss (NOLOAD) : ALIGN(4096) { KEEP(*(.bss .bss.*)) *(COMMON) } :data
 /DISCARD/ : { *(.eh_frame*) *(.comment*) }
}
