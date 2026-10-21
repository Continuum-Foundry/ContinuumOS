# P1-07 — Run and review the Phase 1 acceptance gate

Issue type: Story. Epic: P1. Status: Done. Owner: Codex (local implementation/review).
Depends on: P1-01 through P1-06 Done, including deferred integration cases.
Suggested labels: `phase-1`, `acceptance`, `evidence`.
Backlog: M1 gate and early B32 harness. Requirements: R01/R02/R03/R18;
partial R10/R16/R17 evidence. Test families: M1 portions of T01/T02/T03/T04/T12/T13.

## User story

As a project reviewer, I need a repeatable acceptance bundle that proves the
declared kernel baseline and exposes every remaining limitation before Phase 2/3.

## Implementation scope

1. Finish the proposed `cargo xtask test --suite m1` aggregator. Require all declared
   cases, source/artifact identity checks, explicit fault expectations and bounded
   cleanup. A missing, skipped or timed-out required case fails the gate.
2. Build the final normal kernel and ISO from locked inputs in two clean directories.
   Compare the kernel and ISO outputs, including file order/timestamps/path effects.
   Explain any difference with evidence; do not advertise byte reproducibility when
   differences remain. Both actual packaged artifacts must pass cold boot.
3. Cold-boot the final normal ISO five consecutive times, each in a new QEMU process
   with a fresh firmware variable store and distinct boot-session identifier. Run
   offline with no host shares, other guest disks, kernel injection or snapshots.
4. Require ordered entry, metadata, exception, memory and timer checkpoints before
   emitting the final `m1.ready` event. Verify all records reference the expected
   kernel build and per-run session identity. No frontend/loader message substitutes.
   Each stage checkpoint occurs exactly once; unexpected reset/duplicate entry fails
   even when the host session ID is unchanged.
5. Aggregate the isolated failure cases from P1-02 through P1-06. Fault-test ISO/kernel
   hashes and injection configuration must remain separate from normal artifacts.
6. Retain a sanitized evidence index, reproducible commands, hashes/symbols, original
   logs and limitations. Update story statuses, M1/backlog/validation and handoff only
   after explicit review. ADR status changes require their own documented review.

## Required acceptance matrix

| Scenario | Required outcome |
| --- | --- |
| Five final normal ISO cold boots | all ordered checkpoints, unchanged identity within each run, progressing uptime and expected test exit |
| Second clean build | kernel/ISO comparison reported; that exact ISO also cold-boots |
| Missing/corrupt packaging inputs | host preflight failure, no false boot success |
| Broken loader/kernel configuration | bounded real firmware/loader failure, retained logs, no kernel-ready claim |
| Boot metadata rejection | explicit reason and no later readiness markers |
| Invalid opcode, page fault, panic | actual expected fault and symbolized context; no reset counted as success |
| Memory ownership/exhaustion | reservation/zeroing assertions and honest bounded failure |
| Guard/write/NX and emergency-stack tests | actual guest faults with correct diagnostics and no unexplained reset |
| Timer progress/masking and clock reads | delivered-event assertions, monotonic samples and stable session identity |
| Idle wakeup | halt resumes on the configured interrupt with continuation intact |
| Harness negative cases | missing/stale markers, mismatched build/session IDs, early exit and timeout all fail |
| Regression/documentation | M0 acceptance, documentation links and existing schema fixtures pass |

## Acceptance criteria

- [x] Every row has raw evidence and an explicit result; no required case is waived silently.
- [x] Final offline boots use the exact final ISO hashes and pinned firmware/configuration.
- [x] Timeout policy is fixed in the VM profile before the suite runs. Extending a
  deadline after a failure creates a separately identified run and preserves the failure.
- [x] The harness distinguishes expected fault exit, normal success, firmware failure,
  timeout, unexpected crash/reset and host cleanup. No VM process remains running.
- [x] Source identity, notices, assembly boundary and all upstream trust assumptions
  are reviewable; no whole-OS security or release reproducibility claim is implied.
- [x] Review finds story criteria satisfied and updates the index/handoff consistently.
  If any criterion is blocked, M1 remains incomplete and the blocker is named.

## Completion boundaries and next work

Successful acceptance proves the declared experimental VM boot/memory/timer baseline.
It does not prove UI behavior, process isolation, scheduling, live updating, durable
storage, safe rollback, physical hardware support, Secure Boot or the desktop memory
target. Test-device exit is not physical shutdown support. B06 and B32 remain partial.

Next is the bounded Slint/custom hosted comparison in M2. M3 must still establish
user-mode isolation and complete scheduling/context policy before guest UI/services.
Use the [epic evidence rules](README.md#evidence-and-story-completion) and existing
[validation strategy](../validation.md) to write the review.

Implementation: aggregate harness and final readiness checkpoint implemented on
`codex/p1-07-phase-1-acceptance`. The [complete local review](../../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md)
passes all six criteria: 33 rows, 42 launches and 28 Rust host tests. Five normal
cold boots and the second clean ISO boot pass; the only ISO differences are the
declared session bytes. The owner-authorized development signer/policy resolves
host execution. The initial GPT and host-policy failures remain in the immutable
[blocked review](../../../evidence/phase-1/P1-07/2026-10-20/README.md).
M1's declared experimental baseline passes local Codex self-review, not independent
approval. See the [runbook](acceptance-runbook.md) for scope and repeatable commands.
