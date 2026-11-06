# P4-01 — Recheck prerequisites and lock the guest desktop plan

Issue type: Story. Epic: P4. Status: Not started. Owner: Build/Quality.
Depends on: P3-14 and P2-13 Done; current Phase 0 recheck.
Backlog: B01/B02 follow-up, B32 setup. Requirements: R03/R09/R14/R18.
Test families: T01/T02/T09 setup/T13. Labels: `phase-4`, `prerequisites`.

## User story

As a platform developer, I need current inputs and explicit runtime/device gaps
so a guest desktop cannot be assembled from unverified hosted assumptions.

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

- [ ] Fresh prerequisites, M1/M3 and affected portable/hosted M2 baseline pass with
  exact source/artifact identities; a failed prerequisite blocks extension.
- [ ] Every missing runtime/device/UI/transport hook has a named story and test;
  no probe allocator or synthetic policy is described as a complete desktop service.
- [ ] Profiles, deadlines, sample counts, tolerances, capacity and target metrics
  are machine-readable and independently validated before measurement runs.
- [ ] Draft boundary/lifetime plan, threat assumptions, actual device selection
  and disabled-development configuration have reviewed negative-test coverage.
- [ ] Bounded runner rejects invalid profiles, missing tools/artifacts, unexpected
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
