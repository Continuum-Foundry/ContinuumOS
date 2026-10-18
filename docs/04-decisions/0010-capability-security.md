# ADR-0010: Least-authority handles and explicit trust boundaries

Status: Proposed. Date: 2026-10-17.

## Context and decision

Apps, migrations and updates need access without universal privilege. Enforce
process-local handles with rights, attenuation, explicit transfer and quotas.
Use brokered leases/epochs where authority must change during replacement. Publish
a threat-specific trust model and distinguish developer authority from release updates.

## Alternatives

Ambient filesystem/global-service access is convenient but makes authority harder
to audit. Language-only isolation requires stronger whole-program guarantees than
our mixed native/device stack provides. Capability names alone do not enforce security.

## Consequences

Confused-deputy checks, stale handles, transfer teardown and revocation semantics
must be implemented. DMA and privileged brokers remain important trust boundaries.
No claim of formal verification or complete hostile-app containment is made.

## Gate and reconsideration

M3 tests forged handles/rights/process isolation. M4 tests dev-channel authority and
permission changes. Before hostile code/public networking, obtain independent review
and address VM/IOMMU/runtime limitations. Revisit policy when multi-user support arrives.

Evidence: [S19](../01-research/sources.md#s19), [S21](../01-research/sources.md#s21),
[security spec](../02-architecture/security-and-trust.md).
