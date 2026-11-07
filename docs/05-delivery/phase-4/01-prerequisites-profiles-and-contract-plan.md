# P4-01 — Recheck prerequisites and lock the guest desktop plan

Issue type: Story. Epic: P4. Status: Done. Owner: Build/Quality.
Depends on: P3-14 and P2-13 Done; current Phase 0 recheck.
Backlog: B01/B02 follow-up, B32 setup. Requirements: R03/R09/R14/R18.
Test families: T01/T02/T09 setup/T13. Labels: `phase-4`, `prerequisites`.

## User story

As a platform developer, I need current inputs and explicit runtime/device gaps
so a guest desktop cannot be assembled from unverified hosted assumptions.

## Reviewed completion

The final source-locked serial gate and aggregate reviewer pass local Codex
self-review. See the [compact acceptance bundle](../../../evidence/phase-4/P4-01/final-1794022944926085700/README.md).
It starts at freshly pulled `main` commit `15e5113`, preserves the owner's three
unrelated signing/policy edits, and records every dirty implementation input.
Full generated logs, binaries, source patches and ISOs remain local.

Ten fresh prerequisite commands, complete M1 (33 rows/42 launches), all 97 kernel
host tests, separate ordinary/boundary/scheduler/capability and later metadata
flavors, ten long stress guests, runtime/SDK/portable regressions, all 23 M3 rows
and five aggregate negative probes pass. Hosted M2 passes all sixteen rows on a
fresh selected artifact; existing hosted performance misses remain findings.

The [versioned stress decision](../phase-3/stress-deadline-profile-v2.md) uses
60000 ms only for `stress-test`/`stress-trace`. Ordinary/SDK/public-supervision
guests retain 30000 ms, every fixed sample count and the 320-ms guest-local bounds
remain, and `original_limit_acceptance` is false. Read-only backing classification
now scans references once and is checked against the original object-centric
oracle. Dispatch timing after blocking starts at recorded wait completion; a
host oracle preserves genuinely excessive ready delays. Exact old panic samples
were unavailable, so that earlier panic's cause is not claimed as proven.

The fixed host experiment profile passes 163 independent leaf mutations. Draft
codecs pass 1345 paired cases, including 541 mandatory negatives; ten forged
deadline metadata cases reject. Four real bounded child outcomes and eight
prelaunch rejections pass with owned-process cleanup. Device inventories and
enabled/disabled-development launch plans pass host checks. UUID output paths
prevent the observed rapid negative-fixture timestamp collision.

The [M2 shared-context proof](../phase-2/shared-build-context-review.md) preserves
historical UI/legal pins and checks original Git blobs against current typed TOML,
allowing only the added kernel stress profile and an unchanged-settings comment.
Fourteen rejection cases run before expensive M2 acceptance. Earlier failed and
interrupted runs are retained in the bundle's development findings; none passes
retroactively. Variable host/TCG completion cost remains an unresolved finding.

Runtime/device/UI/transport gaps have named story owners in the
[source hook map and runbook](prerequisites-runbook.md). P4-02 receives proposed,
explicitly unimplemented runtime ceilings for admission/reclamation experiments;
P4-03 receives scoped device/IRQ/DMA experiments. Only the exact existing SDK
subset stays frozen. These host codecs and plans accept no guest driver, display,
input, cryptographic session, reload or desktop performance target.

## Implementation scope

1. Inspect actual source and dirty inputs, strict M3 acceptance, M2 selection and
   retained performance misses. Re-run Phase 0/offline prerequisites, signing and
   doctor, complete current M1 and the M3 gate before extending their boundaries.
   Record missing historical local-only artifacts separately from fresh results.
2. Map every selected UI hook to implementation and an owning P4 story. Audit real
   image/page/heap/table/process limits, lifetime launch credits, trusted bootstrap,
   CPU instructions, immutable fonts and clocks. Identify why the existing small
   SDK probe cannot simply become a long-lived desktop.
3. Inventory exact framebuffer, input and dedicated dev-channel devices available
   on pinned QEMU/OVMF. Propose device/IRQ/DMA trust and backend experiments, separate
   serial recovery, dev-disabled profile and failure injection before driver code.
4. Lock one-vCPU 2 GiB capacity 720p/1080p software profiles, scale/effect/text cells,
   sample counts, warmups, idle interval, fault/reload counts, seeds, resource bounds
   and monotonic clocks. Preserve targets in the epic; host and guest clocks never
   form an unexplained subtraction. Fix numeric growth/fairness/cleanup tolerances.
5. Write draft surface/input/semantic/output/dev codecs and lifetime/version plan,
   test oracle ownership and actual serial acceptance commands/runbooks. Separate
   stable SDK subset from experimental device/desktop extensions. No IDs freeze
   until independent fixtures and real clients exercise the exact subset.

## Acceptance criteria

- [x] Fresh prerequisites, M1/M3 and affected portable/hosted M2 baseline pass with
  exact source/artifact identities; a failed prerequisite blocks extension.
- [x] Every missing runtime/device/UI/transport hook has a named story and test;
  no probe allocator or synthetic policy is described as a complete desktop service.
- [x] Profiles, deadlines, sample counts, tolerances, capacity and target metrics
  are machine-readable and independently validated before measurement runs.
- [x] Draft boundary/lifetime plan, threat assumptions, actual device selection
  and disabled-development configuration have reviewed negative-test coverage.
- [x] Bounded runner rejects invalid profiles, missing tools/artifacts, unexpected
  exit, missing completion and timeouts, and reaps all owned processes.

## Tests and evidence

Retain prerequisite/lock/profile hashes, hook gap map, baseline reviews and bounded
runner failures. Prove commands actually exist; host preflight timeout, guest failure
and interrupted run are separate outcomes. Planning alone runs no guest acceptance.

## Exclusions and handoff

No driver, UI, cryptographic selection or guest performance result is accepted here.
P4-02 receives an admitted runtime budget and P4-03 receives scoped device experiments.
Read [runtime hooks](../../../experiments/ui-link/hooks.json),
[M3 contract](../../06-contracts/runtime-v0.md) and [SDK](../../06-contracts/sdk-v1.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
