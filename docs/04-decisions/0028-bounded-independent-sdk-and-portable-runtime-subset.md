# ADR-0028 — Bound the independent SDK and portable runtime closure

Status: Proposed. Date: 2026-10-31. Gate: P3-12 local interoperability and runtime
review, followed by the separately required M3 and project reviews.

## Context

Earlier user fixtures compile private kernel policy and route modules directly.
They test mechanisms but cannot close independent SDK interoperability. The M2
link arena does not reclaim allocation and has never executed a display backend
in the guest. A broad ABI freeze or UI runtime claim would exceed that evidence.

## Decision

Use a standalone no-std SDK package with explicit bytes, selected generated
dispatch/error IDs, manually owned handles, distinct time/epoch domains and a
reclaiming one-thread heap backed by one 64 KiB memory object. Copy only public SDK/example/model
inputs into three clean locked/offline build roots. Build one unchanged minor-0
client and separate minor-0/minor-1 services. The service header codec and Python
byte/semantic reference are separate from the production codec. Provider examples
are packaged derivatives of the prior tested policy/route logic; they are not
claimed to be independently invented policy implementations. Independent expected
bytes and semantic outcomes are required to detect a shared implementation error.

Exercise accepted and incompatible version/features, moved authority including
rejected application messages, actual sender provenance, waits, current policy
and old/new route fences in the optical ISO guest. Execute selected editor-model,
grapheme segmentation and pinned libm without a font, shaper, raster or display
backend. Accept only measured reclaim/zero/FP/failure hooks and exact teardown.

Freeze only the table in [sdk-v1](../06-contracts/sdk-v1.md) when matching final
source/contract/artifact hashes pass review. Keep supervision, later desktop
families and full text/presentation runtime explicitly outside this SDK freeze.
Retain all development failures locally and compact evidence under the normal
budgets; no upload or distribution follows.

## Alternatives and consequences

Sharing kernel modules with consumers is cheaper but leaves the independent
boundary untested. Freezing all planned operations would assign behavior to
unsupported families. Executing the hosted raster pipeline now would pull M4
resource, device and presentation obligations into M3. The bounded subset creates
some explicit example-code duplication and a deliberately small allocator; its
independent consumer and semantic tests must remain aligned when contracts change.

M4 receives real monotonic clocks/waits and tested portable hooks, with immutable
font/resource lifetimes, input, shaping/raster/presentation and full dependency
instruction review still required. M5 receives mandatory provider checks and
fresh replacement authorization; checkpoints cannot carry authority. No state
migration, durable activation, universal revocation or safe old-code retirement
is accepted here. Revisit the allocator and public ABI only with new independent
compatibility and guest evidence, rather than changing frozen fields silently.

## Local gate outcome — 2026-11-01

The [matching source/contract/guest review](../../evidence/phase-3/P3-12/2026-11-01/README.md) passes for the bounded subset. The selected contract is frozen by its exact SHA-256; this ADR remains Proposed pending project review and the later M3 gate. Guest presentation and reload remain open.
