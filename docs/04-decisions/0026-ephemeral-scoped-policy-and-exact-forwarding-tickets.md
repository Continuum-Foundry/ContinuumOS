# ADR-0026: Ephemeral scoped policy and exact forwarding tickets

Status: Proposed. Date: 2026-10-30. Gate: P3-10 local implementation and review;
independent SDK and later account/replacement gates remain separate.

## Decision

Run the bounded policy broker and synthetic resource provider together in one
isolated user process. Translate authorized scoped role templates into explicit
process/session/resource/operation/epoch assignments. Treat trusted IPC provenance
as attribution, then require current kernel binding liveness and broker session
leases at use time. Authorize role administration only through the exact trusted
bootstrap administrator binding. Host boot correlation never serves as a credential. The out-of-band bootstrap
administrator is a trusted fixture root bounded by its kernel process/session
binding lifetime; ordinary subject-policy changes do not revoke that root.
Credential-backed expiring administrative sessions remain a later accounts gate.

Use single-use, exact-request forwarding tickets and intersect the original
caller's current grant with the forwarding service's current grant. A service's
stronger direct authority cannot satisfy a missing caller grant. Remove all grants
and tickets on a policy epoch change; session invalidation remains monotonic.
This deliberately conservative invalidation includes all delegated children.

## Alternatives and consequences

Ambient per-user roles would incorrectly empower every app of that user. Raw
payload identity and owner fields cannot authenticate a caller. Service-only
authorization would create a confused deputy. Long-lived cached capabilities
would fail the required preview/removal and queued-expiry tests. Universal kernel
handle revocation would require a broader mapping and retained-reference design.

The combined broker/provider avoids inventing a trusted transport between two
policy components at this gate. It exposes one process failure domain, explicitly
fails closed after death and requires fresh trusted restoration. No production
availability, credential security or durable RBAC is accepted. Bounded arrays,
non-reusable process generations, checked policy epochs and at-most-60-second
session leases make resource and lifetime limits explicit. Ordinary direct kernel
handles and mappings retain their separately documented rights and teardown rules.

## Experiment and reconsideration

Require real independently addressed A/B/A2/service IPC, explicit shared read,
write denial, administration/delegation ceilings, exact forwarded requests,
expiry while queued, stale revision/context, removal between preview and commit,
typed privileged-scope separation, actual broker faults, fresh restart bindings
and exact resource settlement. Retain raw decisions, source/artifact hashes and
independent mutation checks. The [runbook](../05-delivery/phase-3/policy-runbook.md)
states the concrete limits. Reconsider co-location, conservative invalidation and
fixed capacities before durable policy, multi-session account services or M5
live replacement. No ADR acceptance follows from a passing host model alone.
