# ADR-0004: Explicit versioned interfaces across replaceable components

Status: Proposed. Date: 2026-10-17.

## Context and decision

Independent component builds cannot safely share arbitrary native Rust layout.
Use bounded messages, explicit encoding, protocol versions, feature negotiation,
typed handles and versioned logical checkpoints. Keep source APIs and wire/ABI
versions separate. No raw Rust object graph or process-local numeric handle is
persistent state.

## Alternatives

Native dynamic plugins offer convenient calls but require stable ABI, allocator,
unload and callback rules. One whole-system binary avoids compatibility but loses
the main live-development goal. Full WIT/Component Model is a later option.

## Consequences

DTOs, schemas, converters and compatibility fixtures are real work. Copied control
messages are acceptable initially; optimize only measured bulk paths. A protocol
requires semantic compatibility, not just matching field sizes.

## Gate and reconsideration

M3 builds clients/services independently. Tests reject unknown required features,
malformed messages, stale epochs and incompatible checkpoints. Revisit binding
generation when handwritten interfaces become a demonstrated maintenance burden.

Evidence: [S02](../01-research/sources.md#s02), [S22](../01-research/sources.md#s22),
[contracts](../06-contracts/ipc-and-objects.md).
