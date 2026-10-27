# ADR-0021: Preempt owner threads and eagerly preserve baseline FP state

Status: Proposed. Date: 2026-10-26. P3-05 engineering experiment.

Use one CPU, a fixed 32-entry equal-priority FIFO, and the locked nominal 5 ms
LAPIC quantum. The current memory backend admits 16 processes and one owning
thread per process. Public thread creation, multiple threads sharing a process,
blocking waits, priority changes and SMP remain unsupported. UI mutation stays
with its one owner. A kernel syscall completes before switching; a kernel timer
IRQ marks pending work only. User IRQs save the complete integer IRET frame and
return to a retained supervisor stack. No suspended Rust borrow or kernel stack
is transferred to another task. Pins remain live through copies, including the
test IRQ, and are released before the syscall return scheduling point.

Eager FXSAVE64/FXRSTOR64 uses private aligned 512-byte buffers. Fresh buffers
initialize x87 control to 0x037f, MXCSR to 0x1f80 and all register payloads to zero.
Require CPUID FPU/FXSR/SSE/SSE2; clear CR0.EM, set MP/NE, OSFXSR/OSXMMEXCPT, and
leave OSXSAVE/FSGSBASE off. Kernel arithmetic remains integer-only. CR0.TS guards
kernel entry, except the two audited assembly save/restore helpers and the final
user-return interval. Real x87 faults kill the owning process. All SSE exception masks are required;
unmasking kills at the next kernel boundary. Pinned TCG does not deliver the
probed native #XM, so hardware SIMD trapping is not accepted; see the runbook. AVX/XSAVE/TLS and
additional state such as PKRU are outside this admitted image contract.

The [Intel system programming manual](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf)
describes OSFXSR and OSXSAVE controls. The local experiment must independently
audit emitted instructions and exercise x87/XMM/control sentinels, fresh process
reuse, unsupported features, faults, pin-held interrupts and exact teardown.

This run-to-completion kernel policy is deliberately bounded by short syscall
admission and observations; it is not a hard real-time guarantee. Scheduler
allocation calls admit at most 64 KiB; larger requests fail without mutation.
Reconsider kernel preemptibility and shared owner threads when later blocking
operations require kernel continuations. P3-06 receives this policy for object
lifetime work; P3-08 adds blocking/wake registration. Remain Proposed pending
project review; no guest renderer, public authority, accounts or live update
follows from this experiment.
