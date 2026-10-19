# Original Rust kernel: ISO entry, checked metadata and fatal diagnostics

Build with `cargo xtask build --profile dev` after dot-sourcing
`tools/Enter-Dev.ps1`. This freestanding kernel has static boot storage and bounded
serial diagnostics and owned exception tables. It has no allocator, owned paging, external IRQ handlers,
clock, processes, pixel rendering or desktop. See the
[metadata runbook](../docs/05-delivery/phase-1/boot-metadata-runbook.md) for the
test contract and evidence.

The host harness starts at [m1.rs](../tools/xtask/src/m1.rs); its
[build, package, launch and observe modules](../docs/05-delivery/phase-1/minimal-iso-runbook.md#host-harness-source-layout)
handle compilation/audits, verified ISO creation, QEMU preparation and bounded
process supervision. The [post-refactor regression review](../evidence/phase-1/m1-module-refactor/2026-10-18/README.md)
records passing host tests and ISO cases. P1-04's
[exception runbook](../docs/05-delivery/phase-1/exception-diagnostics-runbook.md)
documents the fatal-only entry boundary and deferred stack-guard integration.

## Loader and assembly boundary

Loader 12.9.3, Rust bindings `=0.5.0` (defaults off), and protocol commit
`3a0526b700e356f0eac1b71a77697b3fd1c707a3` remain pinned. Base revision 3 and
four-level x86-64 paging are explicit. The linker retains start/base/request/end
markers in writable file-backed memory, with separate RX and R load segments.
Image symbols cover the loaded kernel through BSS. xtask independently inspects
ELF closure, every required request, request revisions and disassembly.

Limine enters ring 0 with paging active, IF/DF clear, SysV without FP/SIMD, and an
aligned stack of at least 64 KiB. `kernel_entry` never returns. The loader stack,
and page tables remain reserved. P1-04 switches to an owned static runtime stack,
GDT/TSS/IDT and separate double-fault/NMI stacks. No frame is reclaimed, no firmware boot service is
called, and no native Rust memory layout becomes an external ABI.

First-party assembly includes table/segment loads, a one-way stack switch and
fatal exception wrappers in [exception-entry.S](src/exception-entry.S), alongside
`cli`, `hlt`, COM1 byte `in`/`out`, and test-only 32-bit `out` to port 0xf4.
The runbook reviews stack alignment, frame layout, clobbers and unsafe assumptions.
UART interrupts stay disabled. Each byte has a bounded
100,000-iteration poll, not a measured guest timeout. Record construction uses a
512-byte stack buffer and integer/core formatting, never a heap or panic arguments.

## Metadata ownership and trust

The allocation-free library is host-testable with
`cargo test --locked -p continuum-kernel --lib`. Its internal `BootInfoV1` contains
owned fixed arrays and strings, physical/virtual address values, resource ranges
and reservations. It is initialized in static storage in place and published
immutably after validation. The private executable adapter uses pinned C ABI
prefixes with compile-time layout checks; no unbounded binding accessor is used.

Trusted firmware/loader supply readable bootstrap response/map objects. Numeric
pointer checks do not prove arbitrary mappings. Once the map validates, every
metadata read checks loader-memory backing. Preserve all loader-reclaimable,
ACPI, unknown and other non-usable ranges, plus explicit kernel/raw-ELF/module/
framebuffer reservations. P1-05 must establish allocation ownership later.

Framebuffers are validated as packed RGB 24/32-bit descriptors; no rendering or
EDID/mode traversal occurs. The base-3 physical ACPI locator is never dereferenced.
It has no assumed alignment or checksum validity. Full limitations and capacities
are documented in the runbook.

## Diagnostics and test-only behavior

Version-3 records include build ID, host-provided session ID, sequence, stage,
outcome and emergency-logger state. Early failures say `session=unavailable`. Normal boot emits one entry,
`m1.boot_info_validated`, resource/reservation summaries, `m1.exceptions_ready` and `m1.boot_complete`,
then halts. There is no guest uptime or secure random identity claim.

`emulator-test` writes 0x10 (QEMU exit 33) after complete success or 0x11 (exit 35)
after rejection; fatal diagnostics use 0x12 (exit 37). `fault-before-entry` halts before diagnostics. `boot-inject`
permits named owned-input fixtures after a safe handoff, including optional
framebuffer absence and malformed metadata. Ordinary builds reject fixture
requests. These features are separately identified test configurations, never
physical-machine shutdown or malicious-loader fault containment.

`exception-test` adds actual invalid-opcode/page-fault/panic and held-logger fault
fixtures. Fatal handlers report then halt or use the explicit test device. No
unresolved exception returns to its caller. Guards and a real double-fault entry
test remain P1-05 work; no arbitrary-fault recovery is established.
