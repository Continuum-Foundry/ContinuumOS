# Capabilities, IPC and service routes

Status: proposed. R06, R10, R13, R14. Contract: [IPC and objects](../06-contracts/ipc-and-objects.md).

Existing capability systems demonstrate rights-bearing, process-local handles and
explicit authority routing. Our design uses that principle; it does not inherit
the verification or detailed ABI of those systems.
[S19](../01-research/sources.md#s19), [S21](../01-research/sources.md#s21),
[S23](../01-research/sources.md#s23)

## Objects and handles

Initial kernel objects: process, thread, address space/memory object, channel,
event/wait set, timer, IRQ resource and mediated device resource. A process-local
handle table maps opaque IDs to object references and rights. Use slot generations
or another validated anti-stale strategy. Forged numbers must never create access.

Rights include read, write, wait, signal, map, execute, duplicate and transfer as
appropriate. Duplication can attenuate rights; it cannot amplify them. Transfer
changes ownership only when a complete message is accepted. Object references
and in-flight transfers have defined teardown behavior on process/channel death.

Revocation is explicit: closing one handle does not necessarily revoke duplicates.
For authority that must be revoked or replaced, use a brokered lease/epoch with
checks at each operation. Do not advertise universal recursive capability revocation
until a derivation/revocation mechanism has been implemented.

## Messaging

Start with copied bounded control messages and bounded attached handles. Sending
to a full queue returns `WouldBlock` or waits under a deadline. Handles and bytes
are accepted atomically as one message; no partial authority leaks on failure.
Schema decoders reject length overflows, invalid enum values, unknown mandatory
fields and resource counts beyond negotiated limits.

Large surfaces/files use explicitly granted shared memory. Map immutable snapshots
read-only where possible. For mutable rings, define producer/consumer ownership,
memory ordering, bounds and cancellation. “Zero-copy” refers only to the measured
segment of the path and is never assumed end to end.

## Stable service identity

A logical service route is distinct from a particular process endpoint. The route
broker maps a service ID and activation epoch to an authorized endpoint. Only the
supervisor/coordinator can change that mapping. Clients include request IDs and
epochs; stale replies and stale mutation attempts are rejected.

A stable route by itself does not preserve a direct old endpoint held by a client.
MVP control calls go through a proxy or a reconnectable client stub. Fast direct
channels require lease expiry and mandatory epoch validation. This overhead is
measured before optimizing the common path.

## Delivery and cancellation

No generic exactly-once claim. Request delivery can fail before or after service
execution. Mutating operations declare idempotency keys and keep an authoritative
deduplication/result record for a specified retention window. Cross-machine effects
need cooperation from their external endpoint or remain uncertain after a crash.
Cancelling a request does not prove an effect was not committed.

When an update quiesces a service, choose one rule per operation: finish before
checkpoint, cancel with an explicit result, or retain broker-owned work for replay.
Do not serialize active Rust futures or stacks. Bounded client queues and backpressure
prevent an indefinitely paused service from consuming all memory.

## Initial acceptance

An independently built client must survive compatible service replacement, receive
a clear incompatibility error, and never talk to a stale writer. Tests cover forged
handles, rights attenuation, queue exhaustion, process death during transfer,
timeouts, stale replies, duplicate requests and reordering at activation.
