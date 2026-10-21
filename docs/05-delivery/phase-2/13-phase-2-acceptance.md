# P2-13 — Run and review the Phase 2 acceptance gate

Issue type: Story. Epic: P2. Status: Not started. Owner: Unassigned.
Depends on: P2-01 through P2-12 Done, including mandatory selected-path fixes.
Suggested labels: `phase-2`, `acceptance`, `evidence`.
Backlog: M2 gate, early B32. Requirements: hosted R04/R08/R09/R15, R19 design, R18;
preparatory R03/R14/R16/R17. Test families: M2 subsets of T02/T05/T07/T09/T10/T13/T14.

## User story

As a project reviewer, I need one reproducible acceptance bundle proving the hosted
UI milestone and its limits before isolated guest processes and desktop integration.

## Implementation scope

1. Aggregate the real Phase 2 commands and required scenarios in a bounded acceptance
   harness/runbook. Pin source, locks, assets/fonts, profiles, timeouts, seeds and
   golden tolerances before execution. Missing/skipped/timed-out required cases fail.
2. Build the final selected hosted path and freestanding link probe from clean
   output directories with offline locked inputs. Inspect identities/symbol closure
   and run the exact tested hosted artifact, not a previous preview binary.
3. Review both candidates' preserved evidence, rights/feature manifests and P2-09's
   provisional decision. Rerun selected-path conformance, reload and performance
   cases against final source; comparison prototype findings remain distinguishable.
4. Demonstrate same-process hosted L0/L1 continuity with synthetic unsaved text,
   selection, focus and window geometry, including failure preservation and 100
   compatible cycles. Use process lifecycle/state traces in addition to screenshots.
5. Review portable/host isolation and changes to existing tooling. Rerun Phase 0;
   if shared kernel/build/package/launch inputs changed, run the affected M1 regression
   gate as well. Do not substitute a UI test for an ISO/guest acceptance result.
6. Retain compact reviewed evidence, exact commands/results/limitations and failed
   attempts. Update story/index/handoff, milestone/backlog/validation and relevant
   specifications only after review; keep later requirements and ADR gates open.

## Required acceptance matrix

| Scenario | Required outcome |
| --- | --- |
| Current prerequisites and clean artifacts | current Phase 0 pass, locked inputs, real final hosted build and selected freestanding link closure |
| Slint/custom comparison | same corpus/profiles, all comparison axes, failures/limits and provisional decision recorded |
| Deterministic renderer fixtures | repeatable layout/paint within fixed tolerances; correct clips/alpha/color/scales and resource lifetimes |
| Keyboard and pointer workflows | editor/search/settings/list tasks, visible focus, correct hit testing/capture and no stuck input |
| Text subset | graphemes, shaping/bidi/fallback, logical/visual selection and composition policy pass the declared corpus |
| Semantics and readability | independent names/roles/state/actions, virtual list navigation, high contrast/reduced motion and scale assertions |
| Typed actions and context design | button/shortcut/script/semantic parity; stale revisions, invalid/expired context and unsupported versions rejected in hosted model |
| Token reload | visible new resource generation in the same host process; document/window/selection/focus state preserved |
| Layout reload | supported structural change with stable IDs; deterministic removed-focus/gesture/composition policy |
| Invalid/failed update | corrupt/stale/oversize/missing-resource/build/validation/headroom failures preserve the old editable generation |
| Reader/resource retirement | in-use buffers/resources remain alive; delayed release and queue limits have bounded outcomes |
| Repeated reload | at least 100 compatible commits with per-cycle state proof and no unexplained settled resource growth |
| Damage and idle | incremental/full redraw conformance, bounded damage, no static perpetual repaint and measured declared timer costs |
| Final hosted metrics | optimized p50/p95/p99, deadline misses, input/submission, idle CPU/wakeups, memory/peaks and reload latency with raw samples |
| Harness negatives and cleanup | stale/missing results, mismatched build/profile, early exit and timeout fail; no owned helper process remains |
| Documentation/regressions | local links/schema/governance/hygiene pass; required Phase 0 and affected M1 checks pass |

## Acceptance criteria

- [ ] Every required row has exact artifact/source identities, reproducible commands,
  raw supporting outcomes and explicit review; none is silently waived.
- [ ] Provisional selection rests on comparable evidence, selected target link closure
  and resolved distribution terms; M4 guest confirmation remains open.
- [ ] Hosted renderer, keyboard, text/semantics, L0/L1 continuity, idle/damage and
  repeated-reload criteria pass against the final selected build.
- [ ] Metrics and failure records state actual results and limits; same-process host
  continuity is never described as unchanged kernel boot ID or guest uptime.
- [ ] Compact evidence retention, source/notices/contracts and relevant regression
  checks pass; generated binaries/full runs/private signing material stay local.
- [ ] Story/index/status and roadmap/handoff agree with a recorded reviewer conclusion.
  Any mandatory blocker leaves M2 incomplete with a named next experiment.

## Completion boundaries and next work

Done establishes the hosted M2 subset of R04/R08/R09/R15 and R19 interface design,
not the full product requirements. No guest render/reload, secure capability system,
native assistive consumer, durable storage or release reproducibility follows.
B37 remains partial until isolated authority and replacement tests pass; B32 grows
with later gates. M3 must establish ELF/syscalls, IPC/handles, scheduling/context
and quotas; M4 confirms this exact toolkit path in the guest and proves resource
reload without kernel/VM reset. M5 still owns compiled behavior replacement and
state transfer. Optional inference remains after that first meaningful guest demo.
Apply the [epic evidence rules](README.md#evidence-and-story-completion) and
[validation strategy](../validation.md). Local self-review is labeled as such;
do not imply independent approval or publish Jira issues as part of this gate.
