# ADR-0010: Least-authority handles and explicit trust boundaries

Status: Proposed. Date: 2026-10-17.

## Context and decision

Apps, migrations and updates need access without universal privilege. Enforce
process-local handles with rights, attenuation, explicit transfer and quotas.
Use brokered leases/epochs where authority must change during replacement. Publish
a threat-specific trust model and distinguish developer authority from release updates.

M3 also establishes the [principal/session/ownership model](../02-architecture/identity-and-authorization.md)
and a bounded role-policy broker with synthetic users. Scoped role assignments may
authorize explicit capability grants within the broker's delegation ceiling;
neither ownership nor an administrator role gives apps ambient authority. Trusted
launch/IPC provenance supplies caller identity. Account authentication, durable
role administration and multi-user sessions have a mandatory M7/B41 gate after M6,
required by the approved M8 installer/accounts amendment. Reserve narrow installer
and account-administration grants at M3; validate trusted input before credentials.

## Alternatives

Ambient filesystem/global-service access is convenient but makes authority harder
to audit. Language-only isolation requires stronger whole-program guarantees than
our mixed native/device stack provides. Capability names alone do not enforce security.

## Consequences

Confused-deputy checks, stale handles, transfer teardown and revocation semantics
must be implemented. DMA and privileged brokers remain important trust boundaries.
No claim of formal verification or complete hostile-app containment is made.

## Gate and reconsideration

M3 tests forged handles/rights/process isolation, caller/role spoofing, cross-owner
denial, explicit sharing, same-user app least authority and brokered session/policy
invalidation. Direct-handle revocation limits must remain explicit. M4 tests
dev-channel authority and permission changes; M5 tests replacement reauthorization.
The M7 accounts gate tests durable account/role policy, login/session lifecycle,
interrupted initial-owner setup and authorized recovery; B44 tests account/ownership
preservation through repair/reinstall. M5 excludes credentials and privileged
installation handles from app checkpoints and rejects active writer replacement.
Before hostile code/public networking, obtain independent review
and address VM/IOMMU/runtime limitations. Revisit policy at the multi-user gate.

Evidence: [S19](../01-research/sources.md#s19), [S21](../01-research/sources.md#s21),
[security spec](../02-architecture/security-and-trust.md).
