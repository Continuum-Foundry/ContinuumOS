# ADR-0020: Single-process SYSCALL/IRETQ and contained user failure

Status: Proposed. Date: 2026-10-25. P3-04 engineering experiment.

Use one trusted current process, an owned guarded kernel stack and TSS RSP0,
SYSCALL with masked entry flags, and validated IRETQ returns. Keep a surviving
kernel continuation for process exit/fault; switch back before reclaiming storage.
Keep fatal kernel/IST behavior. Copy short control buffers under mapping pins with
IRQ/preemption exclusion and precise exception recovery. Generate experimental
operation/error IDs and the exit trampoline from the explicit runtime JSON.

This deliberately avoids SYSRET's return-state hazards, a scheduler before the
context gate, a kernel heap, public numeric-address authority and Rust-layout
serialization. Fixed run metadata and eager pages trade internal fragmentation
for a bounded reclaiming runtime. IRQ exclusion is sufficient only for this one
CPU/no-preemption experiment. P3-05/P3-06 must retain stabilization with actual
preemption, lifetime rights and scheduling; this does not prove those races.

The gate requires real CPL3 calls, denied pointers, exact register/flag state,
process-local hardware faults/panic/OOM and subsequent healthy execution in the
same guest, settlement after every process, independently parsed traces/codegen,
and the complete M1 regression. The [runbook](../05-delivery/phase-3/syscall-boundary-runbook.md)
records assembly/unsafe-copy review obligations and experimental limits. Keep the
P3-03 staging-only test and its UD2 bytes unchanged; runnable launches explicitly
install the generated exit trampoline. No process enters with the placeholder.

Reconsider fixed slots and IRQ masking at P3-05/P3-06, public launch/supervision at
P3-09/P3-10, and freeze only the independently exercised ABI subset at P3-12. No
guest UI, accounts, general blocking IPC, live replacement or performance result
is accepted here. Remain Proposed pending the documented gates and project review.
