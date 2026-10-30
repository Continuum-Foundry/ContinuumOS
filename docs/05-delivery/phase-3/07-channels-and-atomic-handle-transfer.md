# P3-07 — Implement bounded channels and atomic authority transfer

Issue type: Story. Epic: P3. Status: Done (local self-review). Owner: Codex (local implementation).
Depends on: P3-06 Done. Suggested labels: `phase-3`, `ipc`, `handle-transfer`.
Backlog: B12 messages/transfer/provenance. Requirements: R10/R14/R18;
preparatory R19. Test families: T02/T03/T06/T11.

## User story

As a service, I need bounded messages with trusted sender context and all-or-nothing
handle transfer so malformed data or a failed send cannot leak or forge authority.

## Implementation scope

1. Implement channel pairs with bounded inline bytes, attached handles, message
   count and total-byte/handle quotas. Supply nonblocking send/receive first;
   P3-08 adds waits/deadlines. Reject arithmetic overflow and impossible capacity
   before allocation or changing sender authority.
2. Specify transfer-versus-duplication semantics, attenuation, acceptance commit
   point, ownership of in-flight references and destination admission. Bytes and
   authority form one accepted message. Full queues, failed copies and rejected
   attachment lists must not partially consume sender handles or install a subset.
3. Define receive sizing/BufferTooSmall, invalid output pointer and full receiver
   table outcomes, including whether a message remains queued. Resolve peer/sender/
   receiver death before/after acceptance; reclaim only the references actually owned.
4. Attach trusted sender provenance at send time, including process generation and
   bound principal/session reference. Sending through a delegated/transferred endpoint
   must identify the current caller, not automatically its original creator. Define
   queued context validity after sender/session death for later broker checks.
5. Implement explicit little-endian envelope codecs and bounded decoders for version,
   operation, request ID, feature/length fields and attached authority. Keep kernel
   transport metadata separate from spoofable payload owner/role/session assertions.
   Update draft schemas/examples/spec together; retain unfrozen ABI status.

## Acceptance criteria

- [x] Bounded guest client/service messages round-trip using explicit codecs and
  typed errors; no native struct casting or unbounded decoder allocation is used.
- [x] Transfer success, failure and peer-death paths have a single defined acceptance
  outcome with no duplicated/lost authority or partial handle installation.
- [x] Payload owner/role/principal/session values never replace trusted metadata;
  delegated-channel tests attribute each real sender correctly.
- [x] Malformed/oversized messages, unsupported mandatory fields, invalid attachments,
  byte/count/table exhaustion and bad receive buffers preserve the specified state.
- [x] Queued and in-flight teardown restores references/quota; a blocked or malicious
  peer cannot cause unbounded kernel memory growth or a kernel panic.

## Tests and evidence

Use independent host codec fixtures alongside real ring-3 exchange. Inject failure
at validation, reservation, copy, enqueue, receive installation and acknowledgement
boundaries; test full queue, empty queue, receiver table full and each peer dying.
Transfer an endpoint between different principals and same-user apps, then compare
trusted sender records with fabricated payload claims. Retain per-case authority
inventories and exact errors; hosted round trips alone cannot satisfy this story.

## Exclusions and handoff

No network/dev transport, zero-copy ring, generic exactly-once execution or live
service replacement. P3-08 receives atomic nonblocking operations and readiness
conditions; P3-10 enforces current scope/session policy at service use time. Sources:
[objects contract](../../06-contracts/ipc-and-objects.md),
[identity model](../../02-architecture/identity-and-authorization.md),
[capability design](../../02-architecture/capabilities-and-ipc.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation and acceptance procedure: [channel runbook](channels-runbook.md).
The [2026-10-29 local acceptance review](../../../evidence/phase-3/P3-07/2026-10-29/README.md) passes the declared subset, including the complete regression gate. This is not external approval or the M3 exit.
