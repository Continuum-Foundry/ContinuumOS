# P3-11 — Fence service writers behind proxy routes and epochs

Issue type: Story. Epic: P3. Status: Not started. Owner: Unassigned.
Depends on: P3-10 Done. Suggested labels: `phase-3`, `service-routes`, `epochs`.
Backlog: B13 routes/invalidation/restart. Requirements: R10/R14/R18;
preparatory R05/R06/R17/R19. Test families: T06/T11; synthetic route subset of T08.

## User story

As a client of a logical service, I need authorized routing and a mandatory writer
fence so a retired endpoint cannot continue mutating state after a route changes.

## Implementation scope

1. Implement broker-owned logical service IDs distinct from process/channel IDs,
   a proxy/reconnectable stub, activation epochs and authority-checked route changes.
   Bind requests/replies to protocol/version, request ID, caller/session and epoch.
   Bound proxy queues and unavailable-service behavior; no raw endpoint persistence
   or implicit privilege from knowing a service name.
2. Use a broker-owned in-memory synthetic resource as the authoritative mutation
   point. Enforce current writer lease/epoch on every mutation, including attempted
   direct use of an old endpoint. Fence the old writer before granting the new one;
   distinguish route publication from resource authorization and writer ownership.
3. Define request outcomes at admission, forwarding, execution and reply. Specify
   bounded dedupe/result retention for the synthetic operation, expiry and duplicate
   behavior; cancellation/retry does not prove nonexecution or generic exactly-once
   external effects. Reject stale/reordered replies and mismatched caller correlation.
4. Switch between two separately launched fixture service instances with fresh
   process bindings and reauthorized grants. Check current policy/session at use
   time, not only route lookup; old/revoked sessions cannot borrow a new route.
5. Inject service/proxy/policy-broker death around fence/publication/grant/reply.
   Fail closed when current authority is unknown, settle or explicitly reject queued
   work, and recover through trusted bootstrap without resurrecting stale writers.
   Enumerate direct handles/shared mappings outside the mediated mutation path.

## Acceptance criteria

- [ ] Only authorized supervision/broker context can change a route; compatible
  clients reach the active endpoint and receive precise incompatibility/unavailable
  errors with bounded queues and no endpoint-creator identity confusion.
- [ ] During an actual guest route switch, at most one authorized synthetic writer
  can mutate; retained old endpoints and delayed writes fail the resource fence.
- [ ] Stale replies, duplicate/reordered requests, wrong epochs and correlation
  mismatches have defined outcomes consistent with the bounded result record.
- [ ] Replacement fixtures obtain fresh grants; expired/revoked caller or serialized
  old authority cannot become valid through a new route/process generation.
- [ ] Death at each defined transition preserves the single-writer invariant or a
  declared no-writer state; broker restart fails closed with explicit request results.

## Tests and evidence

Keep old and new fixture processes alive during switches and attempt concurrent
old/new mutation through proxy and retained endpoints. Inject failure before/after
fence, grant, route publication, operation commit and reply; compare authoritative
resource revisions with accepted request IDs. Test proxy saturation, unavailable
service, incompatible major/mandatory features, session expiry and policy change.
Retain event ordering, lease/epoch checks and surviving direct-authority inventories.

## Exclusions and handoff

This is a synthetic routing/fencing experiment, not state-preserving app reload,
quiescence/checkpoint migration, safe old-code retirement, durable activation or
rollback of external effects. M5/M6 retain those gates. P3-12 receives tested route,
policy and error semantics for independently built SDK consumers. Sources:
[capability design](../../02-architecture/capabilities-and-ipc.md),
[objects contract](../../06-contracts/ipc-and-objects.md),
[update transactions](../../03-live-update/protocol-and-transactions.md),
[identity model](../../02-architecture/identity-and-authorization.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
