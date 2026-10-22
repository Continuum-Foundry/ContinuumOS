# P2-07 — Close exact freestanding UI compile and link feasibility

Issue type: Story. Epic: P2. Status: Done. Owner: Unassigned.
Depends on: P2-05 and P2-06 Done.
Suggested labels: `phase-2`, `freestanding`, `dependency-audit`.
Backlog: B02 follow-up, B07/B36 target hooks. Requirements: R03/R18; early R14/R16.
Test families: T02/T13 feasibility; no guest execution or process ABI acceptance.

## User story

As a guest integration engineer, I need exact portable dependency and symbol
closure evidence so a hosted winner does not hide an impossible guest backend.

## Implementation scope

1. Reuse the independent Phase 0 experiment pattern, isolated from the production
   kernel. Audit both candidates' actual scene/text/raster code, locked feature
   graphs and generated code against the freestanding target and Rust toolchain.
   Building an unused library import does not satisfy this story.
2. Compile and attempt final links exercising real layout/text/raster APIs and the
   candidate's platform adapter. Resolve allocation with a real bounded arena owned
   by the standalone probe, with alignment/exhaustion behavior; no null allocator
   or dummy success hooks may conceal missing runtime requirements.
3. Inventory every final allocator, clock/event, font, memory/math, input, clipboard,
   semantics and presentation hook. Separate probe-provided memory/platform values,
   portable code and M3/M4 runtime obligations. A compile-only platform fixture is
   labeled as such and cannot establish working input, clocks or guest presentation.
4. Inspect final ELF/symbols for unresolved imports, unexpected std/OS linkage,
   TLS/unwinding, segment permissions and emitted FP/SIMD. Record required allocator
   and context policies. Existing M1 forbids UI math in its kernel path; M3 must
   establish user FP/SIMD policy before actual UI execution.
5. Record exact build/feature/symbol outcomes for custom, Slint AOT and any proposed
   interpreter configuration separately. A failed guest interpreter can have a
   compiled replacement path, but that path still needs its own feasibility evidence.
   Refresh dependency/notices records only for real inputs, without adding UI crates
   to the production kernel by default.

## Acceptance criteria

- [x] Both candidates have reproducible real-API compile/final-link attempts with
  complete feature/input/symbol manifests; Phase 0 library compilation is not reused
  as final-link evidence.
- [x] At least the eventual selected path has final symbol closure with explicit
  bounded probe allocation/platform provisions; unresolved candidates are disqualified
  or block selection until a bounded follow-up resolves them.
- [x] No dummy allocator, unresolved-symbol suppression or host-only dependency
  creates a false freestanding success result.
- [x] Required guest allocator, clocks, single-threading, FP/SIMD and platform hooks
  are listed as M3/M4 obligations, with no claim they exist in the current kernel.
- [x] Exact pass/fail evidence, license/notices inputs and remaining backend risks
  feed the comparison; no probe link is described as a guest UI boot/render result.

## Tests and evidence

Inspect both final-link attempts and preserved failures. Exercise the probe arena's
alignment, limits and exhaustion on host as appropriate; inspect linked ELF outputs
independently and reject injected missing hooks or accidental host-feature linkage.
Retain selected symbol/section excerpts and manifests; full compiler/disassembly
output and binaries remain ignored local artifacts.

## Exclusions and handoff

No production heap, guest UI execution, process loader, scheduler or FP context
implementation. P2-08 receives comparable portability outcomes; P2-09 may choose
only a path whose actual target configuration and remaining runtime work are clear.
Sources: [Phase 0 dependency limits](../phase-0.md#dependency-feasibility),
[UI comparison](../../01-research/ui-toolkit-comparison.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

## Local acceptance — 2026-10-22

[The recorded review](../../../evidence/phase-2/P2-07/2026-10-22/README.md) passes
all five criteria: both actual candidate paths close final symbols using the
owned bounded arena, all 26 ELF mutations reject, each injected missing hook/std
case fails, and both unchanged hosted matrices pass after Slint extraction.
Exact target/build/source/notice graphs and generated AOT IR identities are retained.
The interpreter input is unavailable offline and receives no target credit.
Use the [runbook](freestanding-ui-runbook.md) and its explicit M3/M4 hook inventory.
This is local self-review, not guest execution or selection. P2-08 is next.
