# ADR-0023: Atomic queued authority and bounded cycle collection

Status: Proposed. Date: 2026-10-29. Gates: P3-07, P3-08, P3-12 and P3-13.

## Decision and reason

Use copied bounded messages and explicit attenuating move/duplicate descriptors.
Enqueue commits the entire message and owns its object references. Receive admits
all fresh destination handles after successful output copies and then dequeues.
No process-specific handle reservation is held by a queued message. Table or copy
failure leaves it queued, with no partial authority. An accepted move cannot be
recovered by its sender. No separate acknowledgement syscall is introduced.

Bind sender metadata to trusted launch admission at each actual send, including
process generation and principal/session reference. Endpoint delegation conveys
endpoint rights, not identity. Historical attribution remains meaningful after
sender death, while current broker/session authorization requires a later check.

Precharge fixed queue capacity. Trace from live table/mapping/supervisor roots
through incoming queues to collect unreachable endpoint cycles. Reference counts
still govern individual backing release. A queue reference alone cannot establish
an external root. Run only with IF clear on the declared single CPU; no pointer,
pin, registry borrow or suspended continuation crosses scheduling.

## Alternatives and consequences

Reserving a named destination table on send conflicts with endpoint delegation
and lets inactive receivers pin table capacity. Immediate source invalidation
before full copy/enqueue loses authority on failure. Receiving into user memory
before reserving all new nonces risks publishing incomplete grants. The chosen
order exposes partially copied bytes on failure, but those bytes convey no valid
authority and are explicitly unspecified until successful return.

Simple reference counting leaks cycles when endpoints travel through channels.
Forbidding all endpoint transfer would violate delegation requirements; general
unbounded graph collection is unnecessary for this fixed registry. Bounded tracing
costs at most the object ceiling in passes and uses no dynamic memory. Future SMP,
larger profiles or zero-copy transport must revisit synchronization and latency.

## Verification and reconsideration

Use the [runbook](../05-delivery/phase-3/channels-runbook.md), host rollback/codec
fixtures and real ring-3 exchange. The [local acceptance review](../../evidence/phase-3/P3-07/2026-10-29/README.md) passes this subset. Require quota/table/copy
denials, attenuation, current-caller attribution, queued sender/receiver teardown,
cycle collection and exact reference/frame settlements. Preserve P3-06 and all
narrower feature regressions. P3-08 owns readiness/waits, P3-10 current brokered
policy/session checks, P3-12 independent SDK freeze and P3-13 sustained stress.
No exactly-once, universal revocation or live replacement claim follows.
