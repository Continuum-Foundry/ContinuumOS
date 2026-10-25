# P3-14 — Run and review the Phase 3 acceptance gate

Issue type: Story. Epic: P3. Status: Not started. Owner: Unassigned.
Depends on: P3-01 through P3-13 Done, including required remediation.
Suggested labels: `phase-3`, `acceptance`, `evidence`.
Backlog: M3 gate, M3 B32 subset. Requirements: M3 R03/R10/R14/R18;
preparatory R05/R06/R09/R17/R19. Test families: T01/T02/T03/T06/T11/T13,
runtime T09, timer/context T04 and synthetic route-fault T08 subsets.
T14 covers synthetic provider enforcement only; full replacement parity remains M5.

## User story

As a project reviewer, I need one reproducible final-source gate proving isolated
processes and interoperable contracts before guest desktop and live-update integration.

## Implementation scope

1. Aggregate actual story commands/scenarios into a bounded M3 acceptance runner
   and runbook. Lock source, dependencies, SDK/ABI versions, profiles, CPU features,
   scenario seeds, sample counts, timeouts and resource limits before execution.
   Missing, skipped, timed-out or misclassified required cases fail the gate.
2. Build final kernel, independent user clients/services and separately bootable ISO
   from clean locked offline inputs. Audit ELF/link closure, image permissions and
   artifact identities; execute that exact ISO and packaged user images. Record
   host lifecycle/serial correlation without treating boot IDs as authority tokens.
3. Rerun the complete required M3 matrix below on final source, reusing immutable
   artifact audits only for identical hashes. Include current prerequisites, the
   complete M1 regression gate and affected M2 shared/portable/build regressions.
   Historical reviews and compile-only results cannot substitute for guest execution.
4. Independently review the exercised boundary with codec/ELF/inventory/decision
   assertions separate from production implementations. Review assembly/unsafe
   transitions, locks/lifetimes and trust/remaining-direct-authority limits explicitly.
   Record local self-review as such; do not imply independent human/security approval.
5. Promote a compact sanitized evidence bundle with tested inputs, commands, exact
   artifacts, per-case outcomes, selected raw records, metrics and failed attempts.
   Update stories/index/handoff, milestone/backlog/validation and changed contracts/
   ADR evidence only after review; keep M4/M5/M7 and desktop efficiency gates open.

## Required acceptance matrix

| Scenario | Required outcome | Primary story |
| --- | --- | --- |
| Current prerequisites and clean exact ISO | Current Phase 0/signing/doctor pass, clean locked kernel/user artifacts, actual optical boot and bounded cleanup | P3-01/P3-14 |
| Memory ownership/allocation | Private zeroed pages, guards, W^X/NX/alias denial, reclaiming allocation, partial-failure unwind and preserved M1 reservations | P3-02 |
| Static ELF admission | Real image/startup mappings match audit; malformed, unsupported and over-budget images reject before publication | P3-03 |
| Trusted launch/child ceiling | A/B/service bindings, no forged principal/session/role or unlisted inherited authority, stale instance rejection | P3-03/P3-09/P3-10 |
| User entry/syscall boundary | Correct registers/flags/return, typed discovery/errors, invalid return state and hostile pointers safely rejected | P3-04 |
| User fault containment | Page/guard/illegal/privileged instruction, panic and OOM terminate only offender; healthy guest task continues | P3-04/P3-09 |
| Preemption and CPU isolation | Non-yielding tasks progress within bounds; integer/FP/SIMD/control state preserved/initialized; unsupported features deny | P3-05 |
| Handles and memory objects | Forged/stale/wrong-type/over-right handles fail; attenuation/sharing/close/map races and zero-on-reuse pass | P3-06 |
| IPC bytes and transfer | Bounded explicit encoding, all-or-nothing authority, full-queue/table/bad-copy/peer-death outcomes, no reference leaks | P3-07 |
| Caller provenance after delegation | Current actual sender attributed through transferred channels; payload identity/role cannot select trusted caller | P3-07/P3-10 |
| Waits/deadlines/cancellation | No lost wakeups; elapsed monotonic deadlines, timeout/cancel/close races, overrun and idle/wake policies pass | P3-08 |
| Deadlock/backpressure/resource exhaustion | Fixed queue/wait cycles resolve by declared timeout/cancel/kill; kernel/control service continues with reserved capacity | P3-08/P3-09/P3-13 |
| Supervision and teardown | Exit/fault/hung child, orphan, join, quota/subtree pressure and crash-loop cleanup bounded; stale IDs cannot target replacements | P3-09 |
| Cross-owner and same-user least authority | A cannot access B without grant; shared read succeeds/write fails; ungranted second app of A denied | P3-10 |
| Scoped policy and confused deputy | Wrong-scope/unauthorized roles, child amplification and service borrowing fail; authorized narrow operations succeed | P3-10 |
| Reserved privileged operations | App/development claims cannot perform synthetic installer/account operations or acquire unrelated administration grants | P3-10 |
| Brokered expiry/revocation/restart | Expired session, removed role, stale policy, preview/use change and restart fail closed; direct-authority survival limits recorded | P3-10/P3-11 |
| Proxy route and writer fencing | Authorized switch, fresh grants, rejected old endpoint writes/stale replies; duplicate/reorder outcomes match bounded result records | P3-11 |
| Route transition faults | Kill service/proxy/broker at every defined fence/grant/publication/commit/reply boundary; one writer or explicit no-writer state | P3-11 |
| Independent SDK/version conformance | Separate clean client/service builds and independent codec/semantic consumer; unchanged compatible client works, incompatible features reject | P3-12 |
| Selected runtime hooks | Real allocation/reclaim/startup/failure/math context in nonrendering portable guest workload; M4 presentation/input/resources stay open | P3-12 |
| Sustained lifecycle and accounting | At least 100 launch/reap and 100 synthetic route cycles, combined faults, reconciled resource totals/peaks, fixed fairness bounds and no unexplained settled growth | P3-13 |
| Regression and evidence integrity | Complete M1 pass; affected M2 checks; source/ABI/schema/status consistency, real bounded runner failures/cleanup and compact reviewed bundle | P3-14 |

## Acceptance criteria

- [ ] Every required row passes on final identified guest artifacts; no hosted policy
  model, synthetic success record or compile/link result substitutes for enforcement.
- [ ] Independent client/service/codec and actual optical-boot evidence verify the
  tested ABI subset, fault containment and absence of a stale-writer bypass.
- [ ] Full identity/authority matrix, structural quotas, child ceilings, brokered
  invalidation and declared remaining direct authority pass explicit review.
- [ ] Sustained faults/cycles and resource/latency reports satisfy fixed runtime bounds;
  desktop performance/security/durability limitations remain explicit and unchanged.
- [ ] Current Phase 0, complete M1 and affected M2 regressions pass; host children are
  reaped and every required timeout/missing-record case is classified as failure.
- [ ] Compact evidence, review, public contract versions and story/index/handoff
  updates agree; no account, UI/reload or release gate is marked complete by M3.

## Tests and evidence

In addition to the matrix, deliberately break a user image, expected process outcome,
artifact hash, required completion record and runner timeout. The aggregate checker
must reject each and clean up the VM. Retain exact source/dirty hashes, packaged
user images and ISO hashes, ABI/SDK manifests, launch arguments, selected serial
records, independent assertions and measured distributions. Keep earlier failures
and final rerun identities distinguishable; full build trees/logs remain local.
Run documentation, governance and repository-hygiene checks under the retention policy.

## Exclusions and handoff

M3 accepts only its isolated-runtime/interface subset. M4 must still prove selected
UI guest rendering/input/resources and authenticated no-reboot L0/L1 publication.
M5 must prove state migration, reauthorization, one-writer behavior transfer and safe
old-process retirement; M6 durability/recovery and M7 real accounts/install remain
separate. Preserve the standalone ISO and 256 MiB cold-idle desktop target. Sources:
[M3](../milestones.md#m3--isolated-processes-and-stable-interfaces),
[validation](../validation.md), [identity plan](../../02-architecture/identity-and-authorization.md),
[objects contract](../../06-contracts/ipc-and-objects.md),
[retention policy](../evidence-retention.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
