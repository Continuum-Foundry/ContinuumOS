# P1-04 — Fatal exception diagnostics

Implementation: original Rust kernel with a narrow stable `global_asm!` boundary.
Acceptance is tracked in the [story](04-exception-handling.md). This does not
complete M1 or prove arbitrary-fault recovery, process isolation or stack guards.
This document describes P1-04's retained version-3 baseline. The current P1-05
[memory integration](memory-ownership-runbook.md) adds version 4, guards and real
double-fault/IST tests; check its acceptance status before continuing.
The [2026-10-19 local acceptance review](../../../evidence/phase-1/P1-04/2026-10-19/README.md)
passes 19 host tests and 21 VM launches, including all four new fatal cases.
The [final source formatting check](../../../evidence/phase-1/P1-04/2026-10-19-formatting/README.md)
records a later EOF-only cleanup and one additional passing fault launch, separately
from the complete pre-formatting matrix.

## Run locally

Dot-source `tools/Enter-Dev.ps1` from the repository root. The pinned xorriso
runtime requires the documented execution context outside the sandbox.

```powershell
cargo xtask test --suite m1 --case exception-invalid-opcode
cargo xtask test --suite m1 --case exception-page-fault
cargo xtask test --suite m1 --case panic
cargo xtask test --suite m1 --case exception-emergency
pwsh -NoProfile -File tools/Test-P1-04.ps1
```

The acceptance script also checks formatting, Phase 0, dependency probes, kernel
library tests, M1 prerequisites and all five existing run/ISO/metadata cases.
Every VM is bounded, offline, no-reboot and reaped by the existing observer.
The full `test --suite m1` is implemented in P1-07; see its
[runbook and acceptance status](acceptance-runbook.md).

## CPU contract and entry review

The implementation is based on the Intel SDM Volume 3A, revision 090, sections
7.13–7.14, 10.2 and 10.7, and the stable Rust assembly reference:
[Intel manual](https://cdrdv2-public.intel.com/874249/253668-090-sdm-vol-3a.pdf),
[Rust reference](https://doc.rust-lang.org/stable/reference/inline-assembly.html).
These were reviewed on 2026-10-19. Code is original; no upstream handler code was copied.

- After checked metadata/reservation summaries, `exception_start` switches RSP to
  a static 64 KiB stack and calls Rust with SysV alignment. It never returns.
  RDI carries only the numeric test selector. The old loader stack stays reserved.
- Rust constructs a writable GDT, packed 104-byte TSS, and 256 16-byte IDT gates
  in static kernel storage. A far return reloads ring-0 CS=8; data selectors use
  16. `ltr` selects the two-slot available 64-bit TSS descriptor at 24; `lidt`
  publishes the completed interrupt table. IO-map offset is beyond the TSS limit.
- Vectors 0–31 have distinct wrappers. CPU error-code vectors 8, 10–14, 17, 21,
  29 and 30 retain their hardware word; other wrappers push zero. The last two
  are reserved for vendor extensions absent on the declared CPU. No software
  `int` is used to fake a CPU error-code exception. Remaining gates use the fatal
  vector-255 unexpected-interrupt sentinel, not a claim to identify external IRQs.
- The normalized frame is seven 64-bit words: vector, error, RIP, CS, RFLAGS,
  saved RSP, SS. The common wrapper captures its pointer and CR2 before aligning
  RSP for a SysV call. DF is cleared before Rust. No general register needs
  restoration because every handler is fatal and no interrupted code resumes.
- Double fault selects IST1; NMI selects IST2. Each has its own aligned static
  64 KiB stack. Ordinary faults use the owned runtime stack. All three stacks,
  descriptors and handler text are covered by the validated kernel PT_LOAD range
  and retained kernel reservation. Existing loader page tables supply mappings.
- IF remains clear, both legacy PIC masks are set to 0xff, UART interrupts stay
  disabled. APIC/IOAPIC routing is not configured. There is no `sti`, `iret`,
  user transition, FRED, CET setup, FP/SIMD save, scheduler or handler return.

Unsafe sites are limited to static initialization, CPU frame copying, table/stack
entry and port operations. Single CPU and no task reentry are required. Packed
TSS fields use unaligned raw writes rather than unaligned Rust references. Table
and frame sizes are compile-time checked. Every build retains LLVM disassembly
and independently rejects FP/SIMD, undefined symbols and incomplete image bounds.
The setup transition itself still depends on readable, valid bootstrap mappings;
it is not a guarantee of diagnostics before the owned IDT is loaded.

## Fatal reporting and protocol version 3

Normal entry, checked metadata, summaries and completion retain their meanings;
`m1.exceptions_ready` now precedes completion. Version 3 adds an `emergency`
boolean to every record and the setup/fatal events. Historical version-2 evidence
is unchanged; current assertions reject older records. The 512-byte record limit,
exact build/session identity and gap-free sequence remain enforced.

Reporting uses stack buffers, integer formatting and bounded UART polls. Panic
arguments and arbitrary filenames are never formatted. A nonspinning atomic
logging gate lets fatal reporting bypass an occupied logger and begin a fresh
line. It uses the same bounded builder without reacquiring the gate. A second
fault during fatal reporting emits only `m1.emergency_failed` and exits as failure
or halts. UART failure/overflow also fails; neither can count as an expected fault.
An interrupted partially emitted record may make the transcript invalid, which
is deliberately rejected. The emergency test covers a held gate before emission,
not every possible byte boundary, NMI timing or corrupted-memory failure.

`m1.exception` records vector, RIP, error code, CR2 for page faults (zero otherwise),
saved RSP/CS/RFLAGS and frame address. `m1.panic` records a symbolized reporting
context RIP/RSP and source line; it is not a backtrace or a claimed caller RIP.
Build/session/stage are present in both. Fatal records are never followed by a boot-success
record on tested fault paths. No guest clock or uptime is claimed.

Only `exception-test` binaries accept the new test selectors; this feature implies
`emulator-test`. Ordinary builds reject them. Fault fixtures execute `ud2`, an
assembly load at canonical address `0x400000000000`, or a real Rust panic after
setup. The selected read address is absent on the tested loader profile; a changed
mapping yielding a different outcome fails the assertion. The emergency fixture
marks the logger busy and executes the same real `ud2`.

The host extracts diagnostic symbols from the exact ELF via LLVM and carries
them through package manifests. RIP must match the expected symbol, not just fall
somewhere in kernel text. Fault frames and RSP must lie inside the reported owned
runtime stack. Vector 6 and vector 14/error zero/CR2 checks distinguish the two
hardware exceptions. Panic has its own terminal event. QEMU test exits are:

| Outcome | Port value | Process exit |
| --- | --- | --- |
| Ordinary complete boot | 0x10 | 33 |
| Metadata rejection/reporting failure | 0x11 | 35 |
| Expected fatal diagnostic | 0x12 | 37 |

Normal binaries halt after fatal reporting. The test exit is an explicit emulator
device, not physical-machine shutdown. Missing records, wrong context, reset,
triple fault, wrong exit and timeouts fail. A host session ID alone proves no reset
property; exactly one entry and ordered terminal records are also required.

## P1-05 handoff

Zero loader bytes are reclaimed. ACPI remains an undereferenced physical locator.
P1-05 must preserve the runtime/IST stacks, TSS, GDT, IDT, handler text, boot metadata
and serial mappings when replacing page tables; kernel-image reservations include
them all. Static stack mappings are presently contiguous and unguarded. P1-05 must
establish and fault-test guards and actual double-fault/emergency-stack entry.
Current setup and logger-gate tests do not establish that integration gate.
P1-07 cannot complete until that deferred test passes. No allocator, external IRQ,
timer, user-mode containment, rendering, live update or efficiency result follows.
