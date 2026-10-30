# ADR-0024: elapsed deadlines and bounded wait completion

Status: Proposed. Owner: kernel/runtime. Experiment: P3-08. This records an
engineering default; local acceptance and project review remain separate.

## Context and decision

P3-05 preempts one owning thread per process, while P3-07 transfers queued bytes
and attenuated authority atomically. Delivered LAPIC ticks are boot uptime, not
elapsed time when delivery is delayed. Waits must release pins and kernel borrows
without retaining a kernel call-stack continuation or changing message ownership.

Use the selected validated 64-bit HPET main counter for elapsed nanoseconds and
absolute deadlines. Accumulate checked counter deltas with fractional precision;
reject ambiguous half-wrap/backward observations and overflow. Publish ceiling
hardware resolution separately from the periodic wake opportunity. Preserve the
existing delivered-tick boot epoch unchanged. No wall-clock service is implied.

Use 32 precharged generation-bound wait records with at most eight items each;
the registry still admits at most 16 owner processes. QUERY authorizes status
observation; channel direction additionally requires SEND or RECEIVE. Manual
events and timer objects retain ordinary object/handle lifetime and attenuation.
Watches identify exact owner generations and handle nonces without owning extra
object references. Closing the watched handle completes it as closed even when
a duplicate retains the object. Process exit readiness follows the retained
process object and its exact admitted owner generation.

Serialize snapshot/registration, completion and cancellation with IF clear on the
single CPU. Registry/runtime entry assertions reject recursive domain acquisition;
capture and idle assert both domains released, with no live pins. Nested diagnostic
IRQs may only request later preemption. A parked owner retains a validated user
frame and eager FP state, then returns to the supervisor. An empty runnable queue
uses `sti; hlt; cli` after releasing all domains. FIFO registration sequence orders
simultaneous wake admission behind already runnable work. No SMP policy follows.
Completion is latched at each completed syscall boundary before the caller can
reset a signal or drain readiness again. Supervisor polling admits already
completed records without re-deciding their outcomes. A signal/reset pulse must
not disappear merely because no scheduler turn occurred between the operations.

Complete once, in order: owner termination, watched-handle closure, cancellation,
elapsed deadline, then first ready input item. Initial invalid handles are rejected
before registration. A closed cancel token after registration completes as closed;
an initially invalid one is BadHandle. Confirmed completion never changes after a
late cancellation. Manual event/timer readiness is level and non-consuming.
Periodic timers coalesce missed expirations from their original phase; next-time
exhaustion disarms after final readiness, and unrepresentable overrun counts fail
transactionally. Cancel/rearm clears the ready level.

Blocking channel helpers compose an atomic deadline/cancel-checked attempt with
an exact-capacity readiness wait and retry under the same absolute deadline.
Readiness reserves neither messages nor queue space. Send performs its final
elapsed check after complete validation/admission, at the atomic enqueue decision;
receive checks after fallible output copies and immediately before installation
and dequeue. No diagnostic formatting intervenes before commit. Timeout/cancel
retains sender authority and queued data. Accepted enqueue does not confirm service
effects and cannot be recalled by cancelling a later wait. The helper is not an
async executor, a reservation protocol or a frozen independent SDK.

## Alternatives and consequences

Delivered-tick deadlines were rejected because masked delivery extends elapsed
timeouts. TSC deadlines would need a new frequency/invariance gate; HPET is the
already selected reference. Edge-only readiness requires extra sequencing and
consumption rules; level readiness permits bounded retry without hidden transfer.
Parking borrowed kernel continuations was rejected because it would retain pins
and complicate unwinding, termination and authority ownership.

Applications may form request cycles. Explicit timeout, cancellation, peer death
or trusted termination bounds them; no automatic universal deadlock detector is
claimed. The periodic LAPIC wake opportunity and observer/serial costs remain
visible. Dispatch-preparation latency is not first user-instruction, display/input,
hard-real-time or desktop latency. General supervision and independent SDK closure
remain P3-09/P3-12 work.

## Verification and reconsideration

Require the [P3-08 gate](../05-delivery/phase-3/08-waits-timers-and-cancellation.md)
and [runbook](../05-delivery/phase-3/waits-runbook.md): registration-boundary
signal/enqueue/close, simultaneous cancel/delivery, delayed IRQ with unchanged
ticks and expired elapsed deadlines, bounded cycles, pressured authority transfer,
parked-owner termination, exact inventories, idle and observer-cost evidence.
Run separate existing build/guest flavors and complete M1 before accepting the
story. Revisit for SMP, shared-process threads, hardware without validated HPET,
edge/event consumption, priority inheritance or a measured pacing requirement.
