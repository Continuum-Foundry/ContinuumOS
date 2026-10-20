# ADR-0013: Traceable reproducible builds and distinct output artifacts

Status: Proposed. Date: 2026-10-17.

## Context and decision

Build through a host CLI/xtask with pinned toolchain, dependency graph, loader,
firmware and ISO recipe. Separate kernel/userland/UI/dev/release targets. Emit ISO,
optional disk image, update bundles, symbols, hashes, manifest and provenance.
No toolchain versions/hashes are invented before the feasibility build.

## Alternatives

Manual commands are acceptable for a one-off experiment but not repeatable delivery.
Unpinned network downloads create moving inputs. Shipping only an ISO hides source,
symbols and compatibility metadata needed for debugging live updates.

## P1-07 packaging experiment — 2026-10-20

The [blocked local gate](../../evidence/phase-1/P1-07/2026-10-20/README.md) produced
byte-identical normal kernels in two clean target directories, including a path
with spaces. Both actual ISOs booted, but the first whole-ISO comparison rejected
generated GPT GUIDs/CRCs beyond the deliberately different host sessions. The
recipe now fixes volume/file dates and the experimental GPT disk GUID. Repackaging
the original trees then matched every ISO byte after session normalization.
This isolated result is not release reproducibility or completed M1 acceptance:
Smart App Control blocks the current host tests and the corrected full gate must
be rerun. The [runbook](../05-delivery/phase-1/acceptance-runbook.md) defines the
comparison and records that real ISO hashes remain distinct. Status stays Proposed.

## Consequences

Timestamp/order/path determinism and license inventory are continuing work. A
reproducible binary can still contain a bug; an attestation is evidence about a build,
not automatic security certification. Windows and Linux build parity must be tested.

## Gate and reconsideration

Phase 0 evidence (2026-10-18): workspace-local Rust/LLVM, existing MSVC/SDK,
locked host/guest experiments, manifest/hash output, two clean probe builds and a
source path with spaces were tested. [Runbook/evidence](../05-delivery/phase-0.md).
Native Windows is the tested host; Linux parity and release/ISO reproducibility
are untested. All architectural/release decisions remain Proposed.

M0 records host feasibility. M8 compares two clean builds, explains differences,
boots the exact released ISO offline and verifies dev authority is disabled. Change
packaging tools when portability or reproducibility evidence justifies it.

P1-02 evidence (2026-10-18): separate original-kernel dev builds, UEFI optical
packaging, extracted-payload hash checks, per-process boot artifacts and bounded
offline smoke/negative tests extend xtask. The final ISO digest is recorded after
packaging; the embedded build ID hashes prior inputs, avoiding circular hashing.
See the [runbook](../05-delivery/phase-1/minimal-iso-runbook.md).
Native MSYS2 xorriso needs its documented execution context and explicit Windows
path translation. ISO byte reproducibility and Linux parity remain untested;
this decision stays Proposed.

Evidence: [S61](../01-research/sources.md#s61), [S62](../01-research/sources.md#s62),
[S64](../01-research/sources.md#s64), [build spec](../05-delivery/build-and-release.md).
