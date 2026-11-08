# ADR-0034: Native authorization broker, account presets and scoped roles

Status: Proposed. Date: 2026-11-08. Owner requested architecture and roadmap planning.

## Context and decision

Consumer, enterprise and eventual data-center use need one authorization model
across human users, apps, services, automation and AI. Existing P3-10 evidence is
ephemeral and synthetic; M7 owns real local accounts and durable scoped RBAC.

Use a local user-mode authorization broker backed by kernel capability rights
and provider-side use-time checks. Provide Administrator, Standard user and Guest
presets over explicit scoped assignments; service/workload identities are separate.
Use bounded role/group inheritance, delegated assignment ceilings and contextual
conditions. Elevation is action/target/session bound. Administrative account status
does not flow into every app or bypass resource policy.

Evaluate Cedar first, with exact-target/runtime/license/resource gates, and compare
Casbin and OpenFGA. Do not select a package or assume a freestanding Rust port.
The OS wrapper denies affected evaluations with errors/incomplete trusted context;
Cedar's native skip-on-error behavior alone is insufficient for this proposal.
Keep one bounded versioned contract across UI, CLI, remote management and AI;
policy evaluation stays outside the kernel and rendering/input hot paths.

## Alternatives and consequences

A single admin/user/guest privilege ladder cannot express resource-scoped duties.
Ambient superuser apps enlarge authority; remote-only authorization breaks offline
operation. A custom language/interpreter adds security and maintenance obligations.
Cedar offers upstream Rust policy evaluation but its guest dependencies and costs
are unproven. Casbin remains an embedded alternative; OpenFGA remains a later
relationship-graph candidate. Integration, credentials, durable storage, enforcement
and recovery remain first-party responsibilities regardless of engine.

## Verification and reconsideration

B83 compares semantics and exact guest feasibility; B84 qualifies presets,
elevation, permission explanations and policy lifecycle. M27/B85–B87 qualify
enterprise identity/fleet authorization; M28/B88/B89 qualify the declared workload/
data-center authorization profile. See the [roadmap](../05-delivery/authorization-roadmap.md)
and [architecture](../02-architecture/identity-and-authorization.md).

Reconsider on excessive memory/latency, unavailable runtime dependencies,
unbounded graphs, semantic mismatch, unsupported revocation or maintenance/license
constraints. Pin a successful dependency only after measured evidence and review.
Preserve M7's existing scope, all release roots, ordered Phase 4, deadlines and
256 MiB cold idle. Approval of this plan accepts no engine port or security result.
Sources: [S115](../01-research/sources.md#s115),
[S116](../01-research/sources.md#s116), [S117](../01-research/sources.md#s117).
