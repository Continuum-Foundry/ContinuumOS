# P4-16 — Prove continuity through update and transport faults

Issue type: Story. Epic: P4. Status: Not started. Owner: Updates/Quality.
Depends on: P4-15. Backlog: B17/B32 failure matrix.
Requirements: R04/R10/R12 groundwork/R13/R17. Test families: T05/T06/T07/T08/T11.
Labels: `phase-4`, `faults`, `continuity`.

## User story

As a developer, I need disconnects, retries and failed candidate preparation to
leave a known guest generation and an honest account of preserved work.

## Implementation scope

1. Lock deterministic fault hooks at receive/seal/validate/prepare, final authority
   check, scene-root publication, first presentation, ACK/status and reader release.
   Define each pre/post publication outcome before runs; ephemeral state is explicit.
2. Disconnect/kill the host adapter and resource worker at each applicable boundary.
   Pre-publication failure keeps old authority/root intact; post-publication reconnect
   queries authoritative status rather than blindly executing the update again.
3. Enforce bounded request/result retention, idempotency, duplicate/reorder/late
   replies, expired sessions, wrong base and cancel/commit races. Recovery never
   grants a second writer, revives stale resource authority or discards new edits.
4. Keep recovery status/serial outside the updated data boundary. Define safe
   behavior on update broker/validator death, stalled readers and exhausted staging.
   Separate app/compositor/kernel crashes from lossless resource-update claims.

## Acceptance criteria

- [ ] Every applicable transition fault produces its declared old/new generation,
  accurate status and bounded cleanup with no mixed UI or double activation.
- [ ] Disconnect/retry before and after publication preserves unsaved text/history,
  window/focus and accepted new edits in supported paths with unchanged boot/uptime.
- [ ] Reordered/duplicate/cancelled/stale/revoked requests cannot bypass current
  authority or overwrite a newer generation; unknown/evicted status is explicit.
- [ ] Reader/worker/broker failures cannot free live resources or exhaust recovery
  reserves; unsupported work-loss cases are reported without a success marker.
- [ ] Independent fault/state/resource assertions and precise expected-error records
  pass; timeouts and missing evidence fail instead of becoming accepted cancellation.

## Tests and evidence

Keep the complete local transition table, selected raw fault records, current
document/window/route/resource identities and before/after inventories. Include
auth-policy changes during prepare, lost ACK after visible publication, status
cache exhaustion and delayed old messages after a fresh session.

## Exclusions and handoff

No power-loss durability or general service/code replacement. M5/M6 inherit the
documented boundary, not an assertion that these faults prove their recovery.
Read [failure recovery](../../03-live-update/failure-recovery-and-security.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
