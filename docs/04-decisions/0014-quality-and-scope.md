# ADR-0014: Stage by correctness evidence, then measured efficiency

Status: Proposed. Date: 2026-10-17.

## Context and decision

The desired OS spans many engineering disciplines. Use gated milestones, explicit
support matrices, requirement traceability and fixed workloads. Prove boot/isolation/
update/storage invariants before adding physical hardware breadth. UI beauty and
accessibility evolve alongside the core rather than waiting for kernel completeness.

## Alternatives

A feature checklist without gates can produce a convincing but unsafe demo. Tuning
microbenchmarks before correctness hides architectural bottlenecks. Claiming broad
support from one VM boot undermines the project's usefulness.

## Consequences

Some exciting features are deliberately deferred. Targets remain hypotheses until
measured on declared profiles. Fault injection and state continuity are necessary
tests, while low-impact cosmetic edits do not need an elaborate new test suite.

## Gate and reconsideration

Every milestone ends with reproducible evidence and known limitations. Change
performance budgets after recording measurements and rationale. Scope changes such
as using an existing kernel or hidden reboot require an explicit decision because
they alter core user intent.

Evidence: [requirements](../00-overview/requirements.md),
[validation](../05-delivery/validation.md), [milestones](../05-delivery/milestones.md).
