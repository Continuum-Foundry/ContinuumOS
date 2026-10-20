# P1-05 — Memory ownership and enforced mappings

Original allocation-free Rust implementation; acceptance status is in the
[story](05-memory-ownership.md). This is a single-core kernel memory baseline,
not user isolation, a general heap, a desktop or completed M1.
The [2026-10-19 local acceptance review](../../../evidence/phase-1/P1-05/2026-10-19/README.md)
passes all six story criteria with 23 host tests and 27 selected launches.
Five earlier launches affected by a README input change are retained separately;
the final selected builds use one frozen source inventory. This is local
self-review, not independent approval.

## Run locally

Dot-source `tools/Enter-Dev.ps1`. The pinned native xorriso runtime requires the
documented execution context outside the sandbox's MSYS2 restriction.

```powershell
cargo test --locked -p continuum-kernel --lib
cargo xtask test --suite m1 --case memory-ownership
cargo xtask test --suite m1 --case memory-exhaustion
cargo xtask test --suite m1 --case guard-fault
cargo xtask test --suite m1 --case write-protect-fault
cargo xtask test --suite m1 --case execute-disable-fault
cargo xtask test --suite m1 --case emergency-stack
pwsh -NoProfile -File tools/Test-P1-05.ps1
```

The full acceptance script includes formatting, Phase 0, dependency probes,
M1 prerequisites, all prior ISO/metadata/exception cases and the six new cases.
Every guest launch uses a newly configured, hashed optical ISO, bounded offline
Q35/TCG/OVMF process, one vCPU and 2 GiB capacity. No reboot, network, host share,
snapshot, writable user disk or direct-kernel boot is introduced.

## Ownership and capacity

`kernel/src/memory.rs` implements a checked model with two fixed 128 KiB bitmaps:
eligible and live 4 KiB physical frames. The supported physical-address ceiling
is 4 GiB; any usable descriptor extending beyond it fails with `address_capacity`.
The model admits at most 512 descriptors, checks range arithmetic and rejects
any overlapping descriptor pair involving usable RAM. It handles unsorted maps;
the upstream BootInfo boundary still requires the loader's declared order and
alignment. Independent model tests admit partial edges and align them inward.
Reserved-only overlaps are unions, not duplicate accounting. Reservations remove
every touched page, including partial pages. Page zero is always excluded.

Only kind-0 usable RAM is eligible. Before sealing, all BootInfo reservations
are subtracted: the complete image (including allocator, page-table arena,
GDT/TSS/IDT, metadata and stacks), raw ELF backing, modules, framebuffer,
firmware/MMIO, ACPI locator and every loader-reclaimable range. No loader memory
is reclaimed, even after CR3 changes. Old stacks/tables and loader pointer backing
remain reserved. ACPI is still an undereferenced physical locator.

Allocation scans a bounded bitmap, zeros the entire frame before marking it live
and returns `exhausted` when no frame is available. Zeroing failure publishes no
allocation. Unaligned, out-of-capacity, reserved and double frees fail explicitly.
Reservations are sealed before allocation. There is no heap, demand paging,
overcommit, DMA allocator or concurrent allocator. Frame release does not promise
immediate erasure; the next supply is eagerly zeroed. The explicit exhaustion
fixture caps simultaneous live allocations at eight; ordinary boots have no such
quota. The fixed bitmap/address bound remains in both paths.

## Mapping and CPU enforcement

`kernel/src/paging.rs` builds new four-level tables in 2304 image-owned pages
(9 MiB fixed arena, with an additional metadata/padding page). Only 4 KiB leaves
are supported. Capacity exhaustion is a reported error; no mappings are silently
omitted. A failed map may leave empty intermediate tables allocated, but never
publishes its leaf. Tables cannot be reclaimed/reset while attached. No table
arena frame can be supplied by the ordinary allocator.

The linker exposes aligned text/rodata/image boundaries. Mappings are:

| Memory | Mapping |
| --- | --- |
| Kernel text | supervisor, read-only, executable |
| Kernel rodata and linker padding before writable requests | supervisor, read-only, NX |
| Requests/data/GOT/BSS, metadata and table arena | supervisor, writable, NX |
| Runtime, double-fault IST1 and NMI IST2 stack bodies | supervisor, writable, NX; 64 KiB each |
| Both end pages of each stack | absent, including all direct-map aliases |
| Eligible ordinary RAM only | supervisor, writable, NX through the checked HHDM offset |
| Loader, module, framebuffer, ACPI, firmware/MMIO and kernel physical aliases | absent from the direct map |

The direct mapping intentionally covers only eligible ordinary RAM with ordinary
WB RAM attributes; device and framebuffer cache policy is not guessed. Copied
descriptors survive in the kernel image, but their backing need not remain mapped.
No boot API or stale loader pointer is used after the switch. Serial is port IO.
P1-06 must explicitly map validated ACPI/device ranges before accessing them.

The CPU must advertise NX. The adapter enables EFER.NXE, clears CR4.PGE to flush
any old global translations, installs the owned CR3 and enables CR0.WP. It reads
back CR3/NXE/WP, rejecting unsupported PCID or five-level paging. IF remains clear;
no CPU or process other than this core can access these tables. Parent entries
are supervisor RW, leaves impose RO/NX. Page-table storage uses `UnsafeCell` and
aligned volatile entry reads/writes because hardware may set accessed/dirty bits.

After switching, the guest inspects every image and direct-map leaf, checks all
six guards and checks that every kernel physical alias is absent. It allocates
eight distinct frames, verifies zeroing and reservations, dirties/releases/reuses
them, rejects bad frees, and tests a live scratch alias at `0xffffc00000000000`.
The alias is removed/remapped to a different live frame with INVLPG between every
change; stale contents fail the check. It is removed before backing is released.
There are no globals/PCIDs/SMP shootdowns. Mapping permissions alone do not grant
a future caller authority to map arbitrary physical frames.

## Entry and unsafe review

The [Intel SDM Volume 3A, revision 090](https://cdrdv2-public.intel.com/874249/253668-090-sdm-vol-3a.pdf),
chapter 5 (four-level paging, permissions, invalidation), sections 7.14.5 (IST),
and interrupt 8's double-fault description were reviewed on 2026-10-19. Existing
[P1-04 entry invariants](exception-diagnostics-runbook.md) remain in force.
First-party code is Rust plus the documented architecture-specific wrappers;
no new crate, nightly ABI, FP/SIMD or compiler/runtime port is introduced.

P1-04's three static stack objects now have a page at either end. Entry and TSS
tops point at base + 4096 + 65536. Exception installation precedes the CR3 switch,
while the final stack already executes at its kernel virtual address. The new
mapping preserves its live body, all handler code/data, GDT, TSS and IDT. The old
loader stack is not reused or freed. Static raw initialization assumes one CPU,
IF clear and no task reentry; handlers are fatal and cannot resume a mutation.
No Rust reference is formed to a guard page or an external boot response after
the switch. Raw physical-to-virtual zeroing is confined to eligible mapped RAM.

Fault selectors are accepted only by a `memory-test` binary (which implies the
emulator-test feature). Normal binaries reject them. The small `memory-entry.S`
wrappers perform a real absent-page load, RO text store, jump to an NX data page,
or move RSP to the runtime body bottom and load an absent address. In the last
case delivery of #PF cannot push its frame through the guard, causing actual
#DF entry on IST1. It is not a software `int 8` simulation.

Double fault is an abort: Intel specifies undefined saved RIP. The host checks
the armed injection site's exact ELF symbol, vector 8/error 0, saved runtime RSP,
and frame inside the distinct double-fault stack; it records rather than asserts
a meaningful saved RIP. This is a controlled delivery failure, not a claim that
arbitrary corruption is recoverable. No interrupted code resumes.

## Diagnostic contract version 4

All records retain exact build/session IDs, sequential indices, the 512-byte bound
and bounded P1-04 fatal output. Version 4 adds `m1.memory_switch`, eight
`m1.frame_test` records, `m1.memory_ready` and test-only `m1.fault_armed`.
`m1.exceptions_ready guards=false` describes the pre-switch bootstrap mapping;
`m1.memory_ready guards=6` establishes the owned mapping after inspection.
Completion and all existing expected exception fixtures now require memory readiness.
Historical version-3 evidence is unchanged; current parsers reject older versions.

The host independently checks frame alignment, uniqueness, usable coverage and
reservation exclusion, switch/accounting fields, readiness properties and exact
fault context. Expected terminal outcomes remain:

| Case | Expected outcome |
| --- | --- |
| Ordinary/memory ownership/memory exhaustion | `m1.boot_complete`, test exit 33 (ordinary run is observed then host-stopped) |
| Guard read | #PF vector 14, error 0, CR2 at runtime low guard, exact fault-site RIP |
| Text store | #PF vector 14, error 3, CR2 at image text start, exact fault-site RIP |
| NX jump | #PF vector 14, error 17, RIP/CR2 at exact ELF NX data symbol |
| Emergency stack | #DF vector 8, error 0, frame inside IST1, controlled saved RSP |

All expected fatal cases require exit 37 and forbid boot completion. Metadata
rejections still require exit 35 before exception or memory setup. Timeout,
triple fault/reset, wrong exit, absent or malformed records cannot pass. The test
exit device is an emulator mechanism, not physical shutdown or safe recovery.
The desktop 256 MiB cold-idle target remains unchanged and unmeasured.
