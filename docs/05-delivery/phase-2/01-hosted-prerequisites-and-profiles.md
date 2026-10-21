# P2-01 — Recheck prerequisites and define hosted test profiles

Issue type: Story. Epic: P2. Status: Done. Owner: Codex (local implementation/review).
Depends on: P1-07 Done and Phase 0 acceptance recheck.
Suggested labels: `phase-2`, `build`, `hosted-ui`.
Backlog: B01/B02 follow-up, B32 setup. Requirements: R03/R18; early R09/R16.
Test families: T02/T13 and T09 setup; no UI or guest performance acceptance here.

## User story

As an OS developer, I need a reproducible hosted experiment environment so toolkit
comparisons use the current accepted baseline and declared measurement conditions.

## Implementation scope

1. Inspect source/dirty state, the P1-07 review and historical evidence availability.
   Rerun the Phase 0 commands in its runbook, including clean/link/path-with-spaces,
   overrides and dependency probes. Record current results without replacing old runs.
2. Check local toolchain, offline Cargo caches and development-signing enrollment,
   expiry and executable runner. Run M1 doctor against pinned inputs and review
   the accepted timer/memory boundaries. A host/sandbox block is an explicit result.
3. Inventory hosted window/presentation, tracing and image-comparison facilities.
   Pin actual dependencies/features and notices before use; separate host adapters
   from portable code. Missing cache/assets or acquisition authority is a blocker,
   not permission to enable networking, install global tools or change host policy.
4. Establish optimized comparison and diagnostic profiles with 1280×720/60 Hz,
   1920×1080 as a separate profile, scales 1.0/1.5/2.0, fixed workloads and clocks.
   Record CPU/OS, timers, sampling intervals, warm-up and instrumentation overhead.
5. Add the smallest bounded hosted/headless runner and result manifest convention.
   Validate configuration before launching; record exit/timeout/cleanup distinctly.
   Document real commands and output paths in a new local Phase 2 runbook.

## Acceptance criteria

- [x] Current Phase 0 results, source identity and P1-07 review boundary are recorded;
  any failure blocks dependent implementation rather than reusing stale success.
- [x] Required local tools and signed host execution work. Any concrete blocker has
  reproducible failure evidence and keeps this prerequisite gate incomplete.
- [x] Host-only versus portable dependencies, versions/features and rights records
  are explicit; no guessed pin or silently fetched component enters the experiment.
- [x] Both resolution profiles, scale matrix, clocks and measurement rules are fixed
  and reproducible; no guest metric is inferred from host process counters.
- [x] Invalid profiles, unavailable inputs and runner timeout fail clearly; launched
  processes are reaped and documented commands actually exist.

## Tests and evidence

Exercise the runner's valid configuration, invalid size/scale, missing input,
nonzero exit and forced timeout. Retain compact prerequisite/profile manifests,
selected current Phase 0/doctor results and cleanup outcomes. Full generated output
stays local. Do not rerun the complete M1 VM matrix for planning alone; later source
changes determine whether that regression suite is needed.

## Exclusions and handoff

No toolkit selection, kernel change or rendered-scene claim. P2-02 receives a locked
host experiment profile, an operational bounded runner and the current blockers.
Sources: [Phase 0](../phase-0.md), [handoff](../agent-handoff.md),
[development signing](../windows-development-signing.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation commands and measurement rules are in the
[hosted prerequisite runbook](hosted-prerequisites-runbook.md). The
[local acceptance review](../../../evidence/phase-2/P2-01/2026-10-20/README.md)
records the current prerequisite recheck and 23 lifecycle cases. This is local
Codex self-review, not independent approval. P2-02 is the next bounded story.
