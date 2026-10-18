# ADR-0001: Rust as the initial systems language

Status: Proposed. Date: 2026-10-17.

## Context

The user wants modern language ergonomics without a C/assembly-heavy foundation.
The compiler must emit freestanding code before any OS runtime exists.

## Decision

Use Rust for the original kernel, native services and portable UI core. Prefer
stable tooling, explicitly controlled allocation/panics and a small audited
architecture-specific boundary. Choose a separate userland target/ABI during M0/M3.

## Alternatives

Embedded Swift offers a higher-level feel but needs target/platform-hook validation.
Zig offers direct control with a different safety model. C#/Go require substantial
runtime integration. C/C++ are viable but less aligned with the user direction.

## Consequences

Rust does not eliminate unsafe hardware code, context-switch instructions or service
protocol design. Ownership across a wire is a protocol obligation. Avoiding first-party
C does not eliminate C in upstream firmware or host tools.

## Gate and reconsideration

Phase 0 evidence (2026-10-18): Rust 1.99.1 on Windows MSVC links independent
kernel/userland `x86_64-unknown-none` probes with different code models/layouts.
Two clean outputs match. [Evidence and limitations](../05-delivery/phase-0.md).
This validates a build path, not boot/runtime behavior; status remains Proposed.

M0/M1: compile/link a freestanding image, boot diagnostics and exception handling.
M3: correct user entry/return and allocator behavior. Reconsider if the target/tooling
path blocks the agreed scope or a demonstrated Swift spike materially improves it.

Evidence: [S01](../01-research/sources.md#s01), [S04](../01-research/sources.md#s04),
[language evaluation](../01-research/language-evaluation.md).
