# P4-19 — Run and review the Phase 4 acceptance gate

Issue type: Story. Epic: P4. Status: Not started. Owner: Quality/Project review.
Depends on: P4-01 through P4-18 Done, including correctness/boundedness remediation.
Backlog: M4 gate, B32 review. Requirements: M4 R04/R08/R09/R13/R17;
bounded R03/R10/R14/R15/R18 and R19 groundwork.
Test families: T01/T02/T03/T04/T05/T06/T07/T08/T09/T10/T11/T13, M4 T14 subset.
Labels: `phase-4`, `acceptance`, `evidence`.

## User story

As a project reviewer, I need one reproducible final-source gate proving actual
guest UI and authenticated no-reboot resource reload before starting M5 behavior work.

## Implementation scope

1. Aggregate implemented story commands into a bounded serial acceptance runner
   and runbook. Lock exact source/dirty hashes, profiles/limits, seeds/counts,
   dependencies, clocks and timeout/outcome rules. No required skip/missing record
   passes; reuse audits only for proven identical inputs, never stale guest results.
2. Build clean kernel, independent services/clients and exact standalone optical
   ISO with selected fonts/resources. Execute that artifact in dev-enabled and
   dev-disabled configurations; keep ISO/hosted build paths separately available.
3. Run the required matrix below with complete current M1 and M3 regressions,
   affected M2 corpus/portable/hosted tests, separate affected build flavors and
   host lifecycle failures. Preserve signer/xorriso context and original deadlines.
4. Review unsafe device/IRQ/DMA/mapping paths, buffer/data-reader lifetimes,
   current authority and cryptographic integration. Independently verify codecs,
   pixels, logical state, semantics and inventories. Confirm only the actually
   tested custom toolkit configuration and exact versioned desktop subset.
5. Promote compact sanitized evidence and update stories/index/handoff/backlog/
   milestone/validation/contracts/ADR evidence consistently. Label local self-review
   honestly and retain every target miss and remaining later-gate obligation.

## Required acceptance matrix

| Scenario | Required outcome | Primary story |
| --- | --- | --- |
| Prerequisites/profiles | Fresh offline Phase 0/signing/doctor and locked actual devices, bounds, clocks, source identities | P4-01 |
| Runtime/service admission | Real reclaiming UI heap, finite service graph/restarts, control reserve and exact teardown | P4-02 |
| Device/IRQ/DMA boundary | Scoped grants, negotiation, malformed queues, lost IRQ, safe reset/quarantine and explicit DMA trust | P4-03 |
| Actual framebuffer output | Exact ISO shows checked 720p/1080p pixels; stride/format/map negatives and serial recovery pass | P4-04 |
| Surface/compositor ownership | Independent clients, alpha/overlap/exposure, no in-use mutation, stale/unauthorized commits deny | P4-05 |
| Physical input | Real VM keyboard/pointer, ordering/modifiers/reset/overflow, current transforms and capture lifetime | P4-06 |
| Selected guest runtime/fonts | Real UI/text images and fonts, corpus render/edit, parser/OOM containment, no host fallback | P4-07 |
| Shell/editor workflow | Keyboard-complete launcher/editor/window operations, visible focus and exact unsaved-work baseline | P4-08 |
| Semantics/actions | Independent semantic assertions and checked UI/shortcut/automation parity within M4 scope | P4-09 |
| Trusted setup/login seed | Synthetic protected focus/capture/injection/clipboard denials and no sensitive diagnostic leakage | P4-10 |
| B61 display foundations | Explicit output/generation/geometry codecs; 720p/1080p at declared scales; paint/input/semantic agreement | P4-11 |
| Real dev transport | Actual bounded bidirectional transfer, framing pressure/reset/disconnect and separate serial | P4-12 |
| Authentication | Exact-target reviewed construction/provisioning; bad credentials, tamper, replay, downgrade, expiry/restart deny | P4-13 |
| Authority separation | Trusted M3 caller/session binding; forged roles/owners, cross-app and dev-to-admin/protected escalation deny | P4-13 |
| Plan/blob validation | Complete checked generation; corrupt/oversize/incompatible/semantic-invalid/code payloads reject before publish | P4-14 |
| Candidate headroom | Admission counts old/new/readers/caches/queues; OOM/timeout/validator death keep current UI and control reserve | P4-14 |
| Visible L0 and L1 demo | Host-built token/asset and layout source changes become visible inside the still-running guest | P4-15 |
| State and identity continuity | Unsaved text/history/selection/scroll/window/focus preserved, same app/boot/VM lifecycle and continuous uptime | P4-15 |
| Coherent data retirement | No mixed paint/input/semantic generation, no reader-freed old resource, bounded leases and retained roots | P4-15 |
| Transition faults/retry | All applicable pre/post publish faults, lost ACK, status query, duplicate/reorder/cancel outcomes pass | P4-16 |
| Damage/idle correctness | Partial equals full redraw; static/hidden work bounded; clock/caret and observer overhead counted | P4-17 |
| Guest metrics | CPU/frame/input/reload distributions and actual presentation observation; targets reported honestly with remediation | P4-17 |
| Repeated reload stability | At least 100 consecutive compatible reloads per baseline, exact work and no unexplained settled growth | P4-18 |
| Whole-guest resources | Reconciled cold-idle/active/peak/old-new bytes, shared-once accounting, unchanged 256 MiB target outcome | P4-18 |
| Final regressions/integrity | Complete M1/M3, affected M2/build flavors, exact ISO provenance and deliberate runner failures | P4-19 |
| Disabled development | No mutating endpoint/authority by default in the disabled configuration; ordinary desktop/serial still work | P4-12/P4-13/P4-19 |

## Acceptance criteria

- [ ] Every matrix row passes its declared correctness/boundedness criteria on
  final source; no hosted screenshot, synthetic pipe or compile-only result replaces
  actual guest execution, device transfer or visible publication.
- [ ] Both L0 and L1 demos preserve complete supported logical state with unchanged
  boot identity/continuous uptime and no hidden app/VM replacement or snapshot restore.
- [ ] Corrupt/incompatible/unauthorized/replayed updates, disconnect/retry, synthetic
  sensitive-input attacks and all applicable transition faults meet their fixed outcomes.
- [ ] Guest metrics and 100-cycle resource stability are independently reviewed;
  all target misses and exact profile limitations stay visible without target waiver
  or blanket R08/R09/R15/security-completion claims.
- [ ] Exact guest toolkit/hooks and tested codec subset are confirmed through evidence;
  new ADR status is changed only through its explicit technical/project review gate.
- [ ] Compact evidence, final review and traceability agree; M5 behavior replacement,
  M6 durability, M7 accounts/install and M22 display delivery remain open.

## Tests and evidence

Deliberately corrupt a packaged resource hash, expected logical checkpoint, input/
semantic outcome, boot-continuity record, required completion and runner timeout.
The aggregate reviewer must reject each, reap owned processes and distinguish host
preflight, guest failure, interruption and expected rejection. Keep actual command/
artifact IDs, display captures, selected raw records, independent assertions and
full local logs with compact reviewed retained summaries. Run documentation,
governance and repository-hygiene checks; never copy secrets or full build trees.

## Exclusions and handoff

M4 accepts its guest desktop/live-assets subset. M5 receives model/widget/window
identities, current authority rules, generation/release boundaries and measured
headroom to implement stateful behavior replacement and safe old-code retirement.
M6 receives explicit ephemeral limits; M7 receives synthetic protected-input evidence;
M22 receives B61 codecs/transforms and unsupported topology obligations. No guest
compiler, GPU, real accounts, durability, physical-PC or publication claim is made.
Read [M4](../milestones.md#m4--guest-desktop-and-live-assets) and
[retention](../evidence-retention.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
