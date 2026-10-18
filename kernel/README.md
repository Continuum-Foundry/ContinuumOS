# Minimal original kernel (P1-02)

Build through `cargo xtask build --profile dev`, after dot-sourcing
`tools/Enter-Dev.ps1`. This is a freestanding original Rust binary, separate from
the unchanged Phase 0 link probes. It has no allocator, runtime, exceptions,
interrupt handlers, clock, processes, framebuffer validation or desktop.

## Handoff and assembly boundary

The pinned Limine protocol commit is
`3a0526b700e356f0eac1b71a77697b3fd1c707a3`; loader 12.9.3 and Rust bindings
`=0.5.0`, defaults off, negotiate base revision 3. The link script retains one
8-byte-aligned start/base/end region in a writable load segment, with separate
RX code and read-only data. No native Rust layout is a replaceable-service ABI.

Limine enters x86-64 ring 0 with paging active, IF/DF clear, SysV calling
conventions without FP/SIMD, and an ABI-aligned stack of at least 64 KiB with a
zero return address. `kernel_entry` never returns. We keep that stack, the loader
GDT/page tables and all bootloader memory live; no frame is reclaimed. No UEFI
boot-service calls occur. Boot metadata normalization and ownership follow later.

First-party assembly is limited to `cli`, `hlt`, byte `in`/`out` for COM1 at
0x3f8–0x3ff, and a test-only 32-bit `out` to 0xf4. Each call documents its ring-0
and device assumptions. UART interrupts stay disabled. A 100,000-iteration poll
budget per byte bounds a stuck UART; this is not a measured guest timeout.
Records are fixed literals plus a compile-time hexadecimal identity, without heap
allocation or general formatting. A leading newline separates the first record
from firmware console escape sequences. Panic reporting deliberately omits
arbitrary panic arguments to avoid recursive formatting failures.

The normal binary emits `m1.kernel_entry version=1 build=<sha256>` once and halts.
`emulator-test` instead writes success 0x10 (QEMU exit 33) or failure 0x11 (exit 35)
to an explicitly configured `isa-debug-exit` device. `fault-before-entry` is a
separate test identity that halts before the marker. These features do not provide
physical-machine shutdown, secure identity, uptime or full M1 readiness.

The built-in `x86_64-unknown-none` target supplies the no-red-zone, no-FP/SIMD
baseline. xtask checks the static ELF and retained requests and saves independent
LLVM headers, symbols and disassembly; unexpected FP/SIMD code fails inspection.
