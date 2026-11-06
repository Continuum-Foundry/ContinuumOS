# P4-02 — Admit long-lived UI services and reclaiming heaps

Issue type: Story. Epic: P4. Status: Not started. Owner: Kernel/Runtime.
Depends on: P4-01. Backlog: B13/B16 runtime follow-up.
Requirements: R03/R09/R10/R14/R18. Test families: T02/T03/T06/T09/T11.
Labels: `phase-4`, `runtime`, `allocation`.

## User story

As a UI service, I need enough explicitly admitted memory and a supported service
lifecycle to render repeatedly without exhausting a test arena or launch credits.

## Implementation scope

1. Define a trusted, finite desktop service graph: supervisor, display/compositor,
   input, shell/editor and update/security roles. Assign least authority, per-role
   heap/pages/handles/queues and reserved recovery/control capacity before launch.
2. Adapt the reclaiming user allocator for actual UI/font/surface workloads. Bound
   fragmentation, alignment, realloc/copy and OOM behavior; admit large allocations
   fallibly before publication. Count image/BSS/stacks/page tables and shared backing.
3. Reconcile process-slot and prepaid subtree lifetime ceilings with long-lived
   services, bounded restarts and 100 resource reloads. Reload must not spawn hidden
   helper processes or refund consumed lifetime credits. Revise experimental profile
   capacities explicitly where evidence requires, preserving frozen SDK semantics.
4. Launch services through trusted image/binding mechanisms, not fixture-selected
   return addresses or payload-created identities. Provide bounded failure/reaping,
   restart quarantine and ordinary boot without installed acceptance observers.

## Acceptance criteria

- [ ] Real admitted UI-sized allocation/reallocation/free workloads execute in
  guest processes and settle exactly; no bump-only probe allocator is reused.
- [ ] Capacity denial and partial-start failure unwind without consuming recovery
  reserves or granting authority; source/profile changes have independent review.
- [ ] Service exit, panic, OOM, hung worker and exhausted restart/lifetime budget
  have bounded outcomes while a healthy control task and serial diagnostics progress.
- [ ] Ordinary image boots its trusted service graph without test hooks; separate
  boundary/scheduler/capability and affected later build flavors pass regressions.
- [ ] Resource ownership and remaining direct-mapping authority are explicit;
  no shared-process threads, universal revocation or unsafe retirement is assumed.

## Tests and evidence

Exercise heap fragmentation, zero-on-reuse, max alignment, failed realloc preserving
old bytes, table/slot pressure, subtree admission failure, restart quarantine and
complete teardown. Audit actual user ELFs and CPU requirements. Retain before/peak/
settled inventory and trusted launch grants; run affected kernel and M3 regressions.

## Exclusions and handoff

No desktop rendering or behavior replacement yet. P4-03 receives bounded role
resources; P4-07 receives a reclaiming runtime proven at its declared capacities.
Read [supervision](../phase-3/09-supervision-quotas-and-process-lifecycle.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
