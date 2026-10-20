# P1-06 — Interrupt delivery and monotonic uptime

The [2026-10-20 local acceptance review](../../../evidence/phase-1/P1-06/2026-10-20/README.md) passes all six criteria with 27 host tests and 36 selected VM launches. This is local self-review, not independent approval. See the
[story](06-interrupts-and-timekeeping.md). This is a single-core timer baseline,
not scheduling, wall time, precision measurement, a desktop or completed M1.

## Run locally

Dot-source `tools/Enter-Dev.ps1`. ISO operations use the established execution
context outside the sandbox's MSYS2 named-object restriction.

```powershell
cargo test --locked -p continuum-kernel --lib
cargo xtask test --suite m1 --case timer-progress
cargo xtask test --suite m1 --case timer-masked
cargo xtask test --suite m1 --case clock-read
cargo xtask test --suite m1 --case idle-wakeup
cargo xtask test --suite m1 --case timer-exception
cargo xtask test --suite m1 --case acpi-corrupt
cargo xtask test --suite m1 --case timer-unsupported
cargo xtask test --suite m1 --case timer-calibration
pwsh -NoProfile -File tools/Test-P1-06.ps1
```

The script includes Phase 0, dependency probes, M1 doctor/prerequisite rejection
checks, every P1-03/P1-04/P1-05 case and the eight new cases. `clock-read` launches
the same ELF twice in separately configured ISOs, requiring distinct sessions
and a zero initial tick/uptime epoch each time. Every launch is bounded, offline,
Q35/TCG/OVMF, one vCPU and 2 GiB capacity, with optical ISO boot and no automatic
reboot, network, share, snapshot, writable user disk or direct-kernel boot.

## Discovery and mappings

`platform.rs` parses little-endian bytes without native-layout casts. The guest
requires revision-2, 36-byte RSDP with both checksums, XSDT (at most 128 unique,
nonzero pointers), MADT and one HPET table. Each SDT is 36–65536 bytes with a
checked length and checksum; even unconsumed XSDT tables are validated. Unknown
MADT entries are skipped only after checking their bounded length. The BSP ID
from CPUID must have an enabled local-APIC record. One IOAPIC with GSI base zero
is supported. Duplicate controllers, LAPIC overrides, x2APIC topology, absent
required tables, unsupported GAS or conflicting device ranges reject explicitly.
This is not general ACPI/AML or platform support.

Before dereferencing, every touched ACPI page must lie wholly within a retained
ACPI-reclaimable/NVS descriptor. A single temporary supervisor RO/NX WB page at
`0xffffc00000000000` copies bytes into fixed storage; it is unmapped and INVLPG'd
after every page. No loader pointer is reused and no table reference survives
mapping removal. A 64 KiB image-owned buffer is borrowed by one discovery scope.
Header and full-table reads are separately range-checked. Firmware/loader memory
remains reserved; no reclamation is added.

Device pages must be nonzero, aligned, below 4 GiB and disjoint from each other
and every descriptor except reserved memory. An absent map descriptor is allowed
for a device identified by validated ACPI. Three fixed supervisor RW/NX mappings
follow the scratch page: LAPIC, HPET, IOAPIC. There are no ordinary-RAM/direct-map
aliases. PWT/PCD select PAT index 3; IA32_PAT must report entry 0 WB and entry 3 UC
before any new device mapping. Unsupported cache policy rejects rather than
rewriting host/firmware policy. Existing P1-05 guards and W^X remain intact.

QEMU encodes HPET GAS width as zero (unspecified). The parser admits zero or 64,
memory space, zero bit offset and zero reserved/access byte. The MMIO capability
must independently advertise a 64-bit main counter, nonzero revision and a period
between 1 and 100,000,000 femtoseconds. Its low ID must equal the ACPI block ID.
No frequency is inferred from the machine name or TSC. The first overly strict
GAS-width experiment rejected before enabling interrupts; retain that failure.

## Controller startup and calibration

IF stays clear through discovery, mapping and setup. PIC masks from P1-04 stay
at `0xff`. Every discovered IOAPIC redirection is explicitly masked and read back;
the supported IOAPIC version is `0x20`, with at most 120 entries. All six LAPIC
LVT sources start masked, including both LINT pins. Firmware NMI/ExtINT routing
on those pins is deliberately disabled. Owned IDT vector 64 is the timer; vector
255 is spurious. Other external vectors retain fatal unexpected-entry handling.

CPUID must advertise APIC and PAT; the full IA32_APIC_BASE address must indicate
enabled BSP xAPIC, matching MADT. LAPIC ID must match CPUID, and the supported
version must expose exactly the six used LVT registers; extra LVT sources require
a new masking policy. TPR is zero; SVR enables the controller and vector 255. HPET comparators
and FSB delivery are disabled; legacy replacement stays off. Only its free-running
counter is started. No HPET interrupt routing is installed.

The LAPIC downcounter runs masked, divide-by-16, from `u32::MAX` over a requested
20 ms HPET interval. Both observed deltas and the HPET period are recorded. A
20-million-iteration polling bound and host 30-second VM deadline prevent a stuck
reference from passing. Calibration must be within 10–100 ms, with a nonzero,
nonexpired LAPIC delta and representable nonzero initial count. Integer arithmetic
selects an approximately 10 ms periodic LAPIC interval and records the effective
nanoseconds per delivered tick. There is one calibration window, no drift/accuracy
guarantee and no equivalence to host elapsed time. Only the LAPIC timer is unmasked.

## Returning entry, counter and idle protocol

`timer-entry.S` saves all 15 non-RSP GPRs on the owned runtime stack, clears DF,
keeps the saved-frame pointer in SysV-preserved RBX, aligns RSP to 16 before CALL,
restores every GPR and returns with IRETQ. The hardware frame restores RSP/RFLAGS;
there is no ring transition or SWAPGS. Interrupt gates keep IF clear in handlers;
no nesting or SMP is supported. The target prohibits red-zone and FP/SIMD use.
Every build checks LLVM disassembly for exact saves/restores, alignment, vector,
CLD, CALL and IRETQ and rejects FP/SIMD anywhere in the kernel.

The timer handler is the sole writer of an AtomicU64 delivered-event counter. It
uses one load, checked increment and store, then one 32-bit LAPIC EOI. Overflow
latches an error without wrapping or logging. The spurious handler saturates a
separate counter and returns without EOI. Neither path allocates, formats, blocks,
calls arbitrary code or touches boot/session/log state. UART interrupts stay off.
Fatal reporting clears IF and remains bounded. This does not save FP/SIMD state.

`uptime()` takes one atomic tick snapshot and checked multiplication by the
immutable calibrated resolution; before initialization or after overflow it
returns an error. It never resets the epoch or interpolates from TSC/HPET.
This is delivered-interrupt uptime: masked, coalesced or delayed interrupts can
make it lag elapsed time. It is monotonic at tick resolution, not wall/UTC time,
suspend time, scheduler accounting or a deadline clock. Nanosecond units do not
mean nanosecond resolution. Counter/conversion overflow rejects; it never wraps.
Synthetic tests cover HPET 32/64-bit modular deltas and ambiguous half-range wraps;
the hardware route requires 64-bit HPET and bounded intervals, not a wrap extender.

Each successful establishment performs 100 STI/HLT/CLI sentinel cycles. All GPRs
hold distinct constants and DF is deliberately set across real interrupts, then
verified and cleared before returning to Rust. Each wake must advance the event
counter and its uptime read. Mainline emits 100 samples with unchanged identity;
there is no interrupt-context logging. Normal completion idles with STI/HLT and
timer wakeups. The masking case drains a pending event, enables IF while polling
HPET for at least 60 ms with the timer LVT masked, requires unchanged delivery
count, then restores it and requires a real resumed event.

## Version-5 diagnostics and evidence boundaries

All records retain exact build/session correlation, sequences and the 512-byte
bound. New records are `m1.acpi_ready`, `m1.timer_calibrated`, `m1.timer_ready`,
`m1.clock_sample`, optional `m1.timer_masked`, `m1.timer_progress` and
`m1.platform_rejected`. Boot completion requires timer progress. Earlier controlled
exception/protection cases still fail before timer setup; `timer-exception` takes
a real #UD after delivery and checks its exact ELF RIP, IF-set saved flags, owned
frame and exit 37. Exceptions are fatal, not recovery demonstrations.

Only `timer-test` admits the new selectors; ordinary binaries reject them. The
negative cases corrupt the copied RSDP checksum, remove discovered HPET support,
or force zero calibration delta. These enter real guest rejection paths but do
not claim physical device removal/failure. They require exact field/reason,
exit 35 and no timer readiness/completion. Host fixtures additionally cover
malformed lengths, ranges, topology, checksums, overflow and synthetic wraps.
Success requires exit 33; timeout, restart, malformed log, wrong exit or context
cannot pass. The emulator test-exit device is not physical shutdown.

## Sources and reconsideration

Reviewed 2026-10-19: [ACPI 6.6 section 5.2](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html)
(RSDP/XSDT/MADT), [Intel HPET 1.0a](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/software-developers-hpet-spec-1-0a.pdf)
(register access/capability/configuration/counter and section 3.2.4 table), and
[Intel SDM 3A revision 090](https://cdrdv2-public.intel.com/874249/253668-090-sdm-vol-3a.pdf)
(chapters 7/12/13: interrupt frames, PAT, xAPIC LVT/timer/EOI/spurious).
[QEMU's current HPET table builder](https://github.com/qemu/qemu/blob/master/hw/i386/acpi-build.c)
corroborates the unspecified GAS width. This moving reference is not a source pin
for the locked binary; its reported abbreviated commit was unavailable from the
public repository during review. The local before/after guest rejection experiment
and runtime capabilities supply the profile evidence.
These are public contract references, not new build dependencies or copied code.
See [ADR-0017](../../04-decisions/0017-bootstrap-timekeeping.md). Reconsider this
bounded route before SMP, different firmware/CPU, suspend, scheduling deadlines
or physical hardware; retain the 256 MiB desktop target as unmeasured.
