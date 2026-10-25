# P3-02 — Bounded allocation and owned address spaces pass locally

**P3-02 is Done under local Codex self-review.** This accepts an internal memory
backend, not independent approval, ring-3 enforcement or the M3 milestone. The
loader, syscalls/public rights, scheduler and guest UI remain later work.

Source base: `c2caf9b9bb6496ab66c7968463e1357f7698875a` plus exact dirty
[baseline inputs](baseline-source-inputs.json) and [source provenance](source-provenance.json).
Use the [memory runbook](../../../../docs/05-delivery/phase-3/owned-memory-runbook.md).
Complete generated logs, binaries, ISOs, source patch and disassembly stay local.

## Runs and exact artifacts

- Fresh prerequisites: `build/m2/P2-01/prerequisites-1792954554479`; all ten
  offline/signing/compiler/Phase 0/tool checks pass. The reused directory name
  does not make this historical P2 evidence.
- Complete M1: `build/m1/suite-1792954788819797200-48584`; all 33 rows and 42
  launches pass their expected classifications: 20 ready, nine expected fatal,
  eleven expected rejection, two deliberately broken before entry. All owned
  processes were reaped at the unchanged 30000-ms limit. No failed launch was
  retried or given a longer deadline. See [M1 review](m1-review.json).
- P3-02: `build/m3/P3-02/acceptance-1792954511261`; the four serial acceptance
  commands pass. [All 22 kernel host tests](host-tests.txt) pass, including
  every page-zero and intermediate-table failure stage, metadata exhaustion,
  realloc, stale ownership, pin/shared interleavings and generation retirement.
- Guest memory: `build/m1/owned-memory-1792957065353319400-41156`; an additional
  offline bounded ISO boot passes, with real CR3 switching, active protection
  publication, alias checks, every failing 0–15-page construction quota and
  **1000 poisoned-page create/map/unmap/destroy cycles**. Frame, mapping,
  allocation, table, requested/committed/resident and pin inventories settle
  exactly to zero. Kernel stacks are bound/cleared/guarded but are not executed.

Memory-test build ID:
`92a36437b5c324abeecbfd4fc56d1c6a7622e660cfbca368e19553cd39b24259`.
Kernel SHA-256:
`35e5e6d755f2de8e03078d91e0e724706806b765a4e98f7fde935b7e2579aba4`.
Launched memory ISO SHA-256:
`7716d43ca02721629851a454c8c2289935d531f3164f791363c7ba3e7f05ee1b`.

Normal M1 build ID:
`6cc3cb2ff8cad68d1ecedc64d3131b82b68528b6b8de468f8fc19717e7aa87e2`.
Five consecutive normal-kernel ISO boots and the second clean build boot pass.
Clean kernel bytes match; exact ISO hashes are
`b12b6887ab02d3e3ba3cbf7211b640a2167349174fa91f106106e3f509f0e8c5` and
`0c261eb580dbcfec451ce9d99760fa85eb4ec3cd5954ed39eba97cf407e07905`.
Only the declared session bytes differ; this is not release reproducibility.

## Criterion review

| Criterion | Conclusion / evidence |
| --- | --- |
| Private roots, U/S, W^X/NX, guards and aliases | Pass: complete nonzero table-entry dumps reconstruct separate 20/16-page user trees with 6/4 table pages; physical backing respects M1 reservations. Raw walks, kernel-stack guards and all four RX backing aliases agree. Independent [ELF audit](elf-audit.json) checks RX/R/RW loads and entry. |
| Reclaiming aligned allocation/free/realloc | Pass: zeroed eager runs, checked rounding and exact owner/generation tokens; allocate-first realloc preserves old storage on failure and zeroes grown/padding bytes. P2's arena is absent. |
| Failed mutations and construction | Pass: host staged table/metadata/zero failures and guest quota failures preserve root, references, accounting and frames. No fallible step follows publication. |
| Stabilized copies and shared backing | Pass: pins deny mutation/free/teardown; foreign references deny owner death until removed. Removing one alias cannot reclaim another mapping's backing. |
| M1 actual guards/RO/NX/#DF and reservations | Pass: complete current M1 gate and raw launch/artifact review. Ring-3 cross-process faults/public rights remain P3-04/P3-06. |

The [review](review.json) independently parses launched ELF bytes and raw hardware
entry words and verifies artifact identity, raw sequence/session correlation,
cleanup and settlement. Eleven corrupt-evidence mutations reject. This is an
independent implementation of checks within the same agent's self-review.
All 43 original serial streams are retained byte for byte under `launches/`,
with launch/configuration/artifact/cleanup contexts. Nothing was truncated or
normalized. [Development findings](development-findings.json) preserve earlier
compiler/sandbox/audit issues and superseded passing guest experiments.

## Limits and handoff

Fixed runtime metadata/table objects/padding and 16 guarded kernel-stack slots
reserve **9,854,976 bytes** of the 16-MiB kernel dynamic-storage ceiling, even at
zero live spaces. This excludes M1's existing tables/bitmaps and user backing.
It is a checked storage inventory, not whole-guest cold-idle measurement. The
256-MiB desktop target remains unaccepted. Sparse maps can exhaust 64 tables per
bank; contiguous-run fragmentation can return OOM, and realloc needs both runs.

Pins currently run with IF clear; P3-04 adds bounded preemption masking and
copy-fault recovery. Shared backing currently makes owner teardown Busy; P3-06
must add independent memory-object ownership and public rights. These primitives
do not claim user-mode containment or general process death semantics. No general
kernel heap, user allocator/loader, FP execution, scheduler or ABI freeze exists.
[ADR-0018](../../../../docs/04-decisions/0018-bounded-owned-memory.md) remains Proposed.
The guest fixture uses the accepted M1 CPU pin; P3-05 owns the user CPU/FP gate.
No dependency, compiler pin, signer enrollment or host policy changed.

Next: **P3-03**, trusted static ELF loading and launch context. No later story,
remote transfer, PR or publication is accepted by this bundle.
