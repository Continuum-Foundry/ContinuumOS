# P4-18 — Stress reloads and reconcile whole-guest resources

Issue type: Story. Epic: P4. Status: Not started. Owner: Quality/Runtime/Graphics.
Depends on: P4-17. Backlog: B17/B32 stability.
Requirements: R04/R09/R10/R13/R17/R18. Test families: T03/T05/T06/T07/T08/T09/T11.
Labels: `phase-4`, `stress`, `accounting`.

## User story

As a reviewer, I need repeated guest reloads and reconciled resource inventories
to establish a sustained development loop with bounded resource use.

## Implementation scope

1. Run at least 100 consecutive compatible L0/L1 publications in one uninterrupted
   guest at each locked 720p/1080p baseline, alternating generations while editing,
   scrolling and moving windows. Preserve exact logical work and continuous boot/
   uptime; independent expected state must not be derived from actual output.
2. Interleave bounded malformed plans, auth/session expiry, disconnect/retry,
   staging exhaustion, slow readers and controlled worker failures using P4-16's
   matrix. Keep successful-cycle count distinct from rejected attempts/fault runs.
3. Inventory all guest physical residency/commitment: kernel and static reserves,
   loader/modules, stacks/page tables, service/app heaps, shared backing, framebuffers,
   glyph/layout/effect caches, queues and old/new generations. Count shared bytes
   once globally, attribute owners and show truly free RAM and excluded device space.
4. Measure cold idle before reloads, active peaks, retained-generation pressure and
   fixed quiescent checkpoints. Compare the unchanged 256 MiB target; explain every
   retained difference under locked numeric tolerance and verify full teardown.

## Acceptance criteria

- [ ] Each baseline completes at least 100 compatible reloads without reboot/app
  replacement, lost accepted edits or unexplained monotonic settled growth.
- [ ] Fault/rejection runs preserve supported continuity and bounded recovery;
  successful/rejected/failed/interrupted counts cannot be conflated.
- [ ] Whole-guest resident/committed totals reconcile with physical inventories;
  cache, static/module and shared/device accounting plus host memory are explicit.
- [ ] Cold-idle target, active/reload peaks, headroom denials and fixed growth limits
  have measured outcomes; a target miss is retained as a finding, not relabeled passed.
- [ ] Lifecycle/lease/buffer/resource inventories settle exactly or within justified
  predeclared bounded cache tolerance; recovery/control progress meets fixed bounds.

## Tests and evidence

Retain compact per-cycle identities/counters/state hashes plus selected raw records,
full local metrics and independent inventory review. Reject planted leak/double-count,
missing cycle, changed boot ID, failed publication and stale-source evidence. Complete
bounded runs serially with unchanged deadlines and preserve all interrupted attempts.

## Exclusions and handoff

No M5 behavior-replacement stress, power-loss persistence or M22 reconfiguration
cycles. P4-19 receives complete final-source stress and target findings. Read
[requirements](../../00-overview/requirements.md#performance-targets-and-accounting).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
