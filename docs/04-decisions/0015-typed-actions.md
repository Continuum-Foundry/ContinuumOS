# ADR-0015: Typed actions shared by UI, automation and optional AI

Status: Proposed. Date: 2026-10-17. Added in the user-approved reconciliation.

## Context and decision

Apps need operations usable from widgets, shortcuts and assistive/automation clients.
Define versioned typed actions with capability requirements, expected revisions,
effects, idempotency, deadlines and explicit undo support. Keep authority in the
kernel/resource owner; discovery or an action name cannot create privileges.
An optional assistant can propose the same operations.

## Alternatives

Separate UI/automation paths drift in behavior and permissions. Screen scraping misses
native semantics. A general workflow engine is unnecessary before a useful editor;
start with a minimal SDK/registry over existing service IPC.

## Consequences

Preview and execution require separate checks against current state. Context
provenance, expiry, cancellation and multi-step partial failure become contracts.
No generic atomicity or undo promise for cross-provider external effects.

## Gate and reconsideration

M2 defines a small interface; M5 proves identical authorized UI/automation results
through compatible replacement. R19/T14 cover stale revisions, forged authority and
retries. Expand workflows when real apps justify them. Inference is not a prerequisite.

Specification: [actions architecture](../02-architecture/actions-and-optional-ai.md),
[contract](../06-contracts/actions-and-context.md).
