# ADR-0005: Transactional replacement instead of arbitrary code unloading

Status: Proposed. Date: 2026-10-17.

## Context and decision

The machine must remain up while supported code changes. Start candidates in new
processes/instances, quiesce old work, migrate logical state, fence writers, publish
one route/resource generation, health-check and retire old execution safely.
Kernel/boot changes require reboot initially. Permit one active update globally.

## Alternatives

Naive shared-library unload leaves active stacks/callbacks unsafe. Whole-VM reboot
violates live-update intent. General kernel patching is substantially more complex.
Preserving every old process forever avoids unsafe reclamation but leaks resources.

## Consequences

Stable brokers/supervisor become a recovery dependency. State ownership, durability,
request replay and irreversible effects must be explicit. Post-activation rollback
is not guaranteed for every update class.

## Gate and reconsideration

M5 demonstrates editor logic replacement preserving unsaved state and boot identity.
M6 injects failures through durable activation. Consider more granular/native patching
only after process-level correctness and a measurable need.

Evidence: [S14](../01-research/sources.md#s14), [S15](../01-research/sources.md#s15),
[S17](../01-research/sources.md#s17), [update model](../03-live-update/model-and-guarantees.md).
