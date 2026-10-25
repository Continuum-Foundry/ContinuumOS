# Kernel and execution design

Status: proposed. R01, R03, R10, R14. References: [CPU manual](../01-research/sources.md#s30),
[Rust target](../01-research/sources.md#s01), [ELF](../01-research/sources.md#s31).

P3-01's [runtime draft](../06-contracts/runtime-v0.md) records the implementation
defaults and owning test gates; [locked profiles](../../config/m3-profiles.json)
fix capacities, CPU policy and measurement rules before runtime experiments.
This does not implement isolation, scheduling or a guest heap.

## Boot-to-process path

Validate the boot response, establish serial output, reserve physical ranges,
install owned page tables and exception stacks, create allocators, configure
interrupt routing/timers, and start an initial user-mode supervisor. Every step
emits a structured boot checkpoint. Before a heap exists, logging uses fixed buffers.

## Memory

Start with 4 KiB pages and a physical frame allocator that never returns firmware,
MMIO, kernel, boot-module or still-live boot data frames. Reserve before reclaiming.
Use typed virtual/physical addresses with overflow-checked range operations.
Kernel stacks have guard pages. User stacks and mappings have explicit limits.

Kernel and user virtual mappings enforce user/supervisor permissions, NX and W^X
where supported. Validate user buffers before copying; pin or otherwise stabilize
their mappings for the operation to avoid check/use races. Copying small control
messages is an acceptable baseline. No syscall trusts a userspace pointer because
the caller wrote it in Rust.

Use eager zero-filled pages initially. Defer swap and advanced demand paging.
Define a kernel OOM path and per-process allocation failures; do not loop forever
or panic the entire OS for an ordinary app quota failure. DMA memory has separate
lifetime/cache/ordering rules; a normal allocator is not a DMA API.

## Threads and scheduling

Bring up one CPU, then kernel threads and real user tasks. Use a simple preemptive
priority/fairness policy with bounded queues and no claim of hard real-time behavior.
Track runnable time, blocking and dispatch latency. Give input/display work enough
priority for responsiveness without letting an app permanently inherit it.

Prohibit blocking allocations and arbitrary IPC inside hard IRQ context. Acknowledge
the device/controller, record bounded work and wake a service. Context switching
must preserve the documented register set. Save/restore FP/SIMD state before any
user or renderer code can rely on it; define initial state and prevent leakage.

Maintain an explicit lock hierarchy. No global lock held while waiting for a user
process. Count preemption/IRQ nesting and assert balanced transitions. SMP adds
per-CPU state, TLB shootdowns, cross-core wakeups and memory-order analysis; it is
not enabled merely by passing `-smp 2` to QEMU.

## Process loading and syscall ABI

Initially load static userland ELF images from a trusted package. Check architecture,
segment bounds, overlaps, alignment, permissions, entry point and resource limits.
Do not implement a general dynamic linker for the first reload: start a new process
image instead. Reject unsupported relocations rather than approximating them.

Specify syscall numbers, argument registers, error encoding, clobbers, pointer
validation and version discovery in one generated SDK. Select a userland code
model/entry ABI separately from kernel defaults. Document syscall entry/return
privilege transitions, canonical address checks and interrupt-state handling.

## Failure behavior

P1-04 implements the bootstrap fatal-only baseline described in the
[exception runbook](../05-delivery/phase-1/exception-diagnostics-runbook.md):
owned GDT/TSS/IDT, static runtime and IST stacks, and bounded version-3 serial
records. P1-05 implements owned frame allocation/page tables, WP/NX enforcement,
guarded stacks and controlled actual double-fault integration; check the
[memory runbook and acceptance status](../05-delivery/phase-1/memory-ownership-runbook.md).
The user/process behavior below is still proposed.

User page faults kill or suspend only the offending process and notify its supervisor.
Device service faults trigger class-specific recovery. Kernel faults enter a bounded
panic path with serial output, build/boot IDs and crash context; automatic reboot is
optional and must not masquerade as successful live updating.

The kernel is nonreloadable initially. Keep booting and debugging it straightforward.
Any future livepatch mechanism needs a separate ADR, CPU-wide consistency model,
state proof/validation and failure strategy.

## P1-06 timer baseline

The [interrupt/timekeeping implementation](../05-delivery/phase-1/interrupts-timekeeping-runbook.md)
adds ACPI-discovered xAPIC/HPET, explicit UC device mappings, masked PIC/IOAPIC/LVT
startup, all-GPR interrupt return and a single-writer atomic event counter. Only
the periodic LAPIC timer is enabled. Spurious interrupts return without EOI;
timer delivery performs one acknowledgement and no logging or allocation.
Checked uptime counts delivered ticks at the calibrated resolution, and can lag
elapsed time while interrupts are masked/coalesced. It is not wall time or a
scheduler deadline clock. Read the story review before treating its gate as passed.
Scheduling, FP/SIMD context, SMP and user isolation remain later work.
