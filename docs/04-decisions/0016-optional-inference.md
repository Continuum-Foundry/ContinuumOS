# ADR-0016: Inference is an optional isolated extension

Status: Proposed. Date: 2026-10-17. Added in the user-approved reconciliation.

## Context and decision

The external plan adds AI beyond the original desktop/live-update scope. Preserve
an AI-free OS. After M5, optionally test a small supported model in a constrained
inference service, using a candidate such as Burn's CPU/no-OS path. Outputs are
proposals through typed actions. Core release gates do not require inference.
This track now has M21/B39 as its named gate. Its B06/B21/B37 prerequisites may
pass before M15; numeric ordering does not require waiting for the physical-PC
program. Neither M8 nor M14 requires it, and it does not require M20 Wasm.

## Alternatives

A mandatory large model expands boot/memory/runtime/driver dependencies before live
development works. Kernel-resident inference broadens privilege and failure impact.
Hosted inference is not an assumed network prerequisite.

## Consequences

Operators, conversion, model license, FP/SIMD, quotas and memory/latency need integration
work. Report cold/warm/active memory; weights and temporary buffers must not hide in
caches or device allocations.

## Gate and reconsideration

B39/E10: propose one bounded action; test invalid proposals, worker crash, unload/reload,
pressure and editor responsiveness. Expand to a language model or accelerator only
with a measured workload. “Better than Defender” is no established requirement or claim.

Evidence: [S80](../01-research/sources.md#s80),
[architecture](../02-architecture/actions-and-optional-ai.md).
