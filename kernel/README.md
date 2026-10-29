# Original Rust kernel: ISO entry, checked metadata and owned memory

Build with `cargo xtask build --profile dev` after dot-sourcing
`tools/Enter-Dev.ps1`. This freestanding kernel has static boot storage and bounded
serial diagnostics and owned exception tables. P1-05 adds a bounded physical-frame
allocator, owned paging and guarded runtime/IST stacks; check its
[memory runbook and acceptance status](../docs/05-delivery/phase-1/memory-ownership-runbook.md).
P1-06 adds validated ACPI, masked controller startup, a returning local APIC timer,
HPET calibration and monotonic delivered-tick uptime. See the
[timekeeping runbook](../docs/05-delivery/phase-1/interrupts-timekeeping-runbook.md)
for acceptance status and limits. It has no general heap, processes, pixel rendering or desktop. See the
[metadata runbook](../docs/05-delivery/phase-1/boot-metadata-runbook.md) for the
test contract and evidence.

The host harness starts at [m1.rs](../tools/xtask/src/m1.rs); its
[build, package, launch and observe modules](../docs/05-delivery/phase-1/minimal-iso-runbook.md#host-harness-source-layout)
handle compilation/audits, verified ISO creation, QEMU preparation and bounded
process supervision. The [post-refactor regression review](../evidence/phase-1/m1-module-refactor/2026-10-18/README.md)
records passing host tests and ISO cases. P1-04's
[exception runbook](../docs/05-delivery/phase-1/exception-diagnostics-runbook.md)
documents the fatal-only entry boundary. P1-05 preserves that entry while switching mappings.

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
GDT/TSS/IDT and separate double-fault/NMI stacks. P1-05 switches CR3, enables WP/NX,
removes kernel direct-map aliases and guards both ends of each stack. No loader frame is reclaimed, no firmware boot service is
called, and no native Rust memory layout becomes an external ABI.

First-party assembly includes table/segment loads, a one-way stack switch and
fatal exception wrappers in [exception-entry.S](src/exception-entry.S), alongside
`cli`, `hlt`, COM1 byte `in`/`out`, and test-only 32-bit `out` to port 0xf4.
The runbook reviews stack alignment, frame layout, clobbers and unsafe assumptions.
P1-05 adds CR0/CR3/CR4, EFER, CPUID and INVLPG operations plus explicit fault wrappers
in [memory-entry.S](src/memory-entry.S). The fixed allocator and checked mapping
builder are host-testable in [memory.rs](src/memory.rs); the unsafe guest adapter
is isolated in [paging.rs](src/paging.rs).
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
framebuffer reservations. Only eligible kind-0 RAM becomes allocatable after
subtracting these reservations; no loader memory is reclaimed.

Framebuffers are validated as packed RGB 24/32-bit descriptors; no rendering or
EDID/mode traversal occurs. P1-03 records the base-3 physical ACPI locator without dereferencing it; P1-06
checks its ACPI backing and explicitly maps table pages read-only before copying.
It has no assumed alignment or checksum validity. Full limitations and capacities
are documented in the runbook.

## Diagnostics and test-only behavior

Version-5 records include build ID, host-provided session ID, sequence, stage,
outcome and emergency-logger state. Early failures say `session=unavailable`. Normal boot emits one entry,
`m1.boot_info_validated`, resource/reservation summaries, `m1.exceptions_ready`,
`m1.memory_switch`, bounded frame checks, `m1.memory_ready`, ACPI/calibration/timer
checkpoints, 100 clock samples, timer progress and `m1.boot_complete`,
then idles with STI/HLT. Uptime counts delivered interrupts at the calibrated
resolution; it is distinct from wall/host time. Session identity is not secure randomness.

`emulator-test` writes 0x10 (QEMU exit 33) after complete success or 0x11 (exit 35)
after rejection; fatal diagnostics use 0x12 (exit 37). `fault-before-entry` halts before diagnostics. `boot-inject`
permits named owned-input fixtures after a safe handoff, including optional
framebuffer absence and malformed metadata. Ordinary builds reject fixture
requests. These features are separately identified test configurations, never
physical-machine shutdown or malicious-loader fault containment.

`exception-test` adds actual invalid-opcode/page-fault/panic and held-logger fault
fixtures. Fatal handlers report then halt or use the explicit test device. No
unresolved exception returns to its caller. `memory-test` adds explicit ownership,
eight-frame exhaustion, guard/RO/NX fault and real double-fault/IST cases. Check
the story's review before continuing to P1-06; no arbitrary-fault recovery is established.

P1-06 adds all-GPR returning wrappers and a register/DF/idle sentinel in
[timer-entry.S](src/timer-entry.S), byte parsers in [platform.rs](src/platform.rs),
and the hardware adapter in [timer.rs](src/timer.rs). No FP/SIMD context support
or scheduling is introduced. The timer handler only increments a bounded atomic
counter and acknowledges the LAPIC; spurious delivery returns without EOI.

P3-04 adds the explicitly gated `user-boundary-test` experiment: owned per-process kernel/TSS stacks, integer-only IRETQ/SYSCALL entry, generated experimental operation IDs, checked pinned copies, private reclaiming allocation and contained user exit/faults. The trusted continuation reaps only after restoring the surviving kernel stack/root. See the [boundary runbook](../docs/05-delivery/phase-3/syscall-boundary-runbook.md) and story review for exact evidence. The ordinary M1 ISO remains independently buildable. P3-05 adds gated preemption and eager x87/SSE state; P3-06 adds gated public handle/memory rights. Guest UI remains a later gate. Test flavors register their policy through `user_fixture.rs`; all use the same syscall dispatch and return validation.
