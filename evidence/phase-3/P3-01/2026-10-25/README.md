# P3-01 — Prerequisites and runtime contract gate passes locally

**P3-01 is Done.** This is Codex implementation/local self-review, not independent
approval or M3 runtime acceptance. No guest heap, loader, isolation, scheduling,
IPC, user FP execution, live update or desktop performance is implemented here.

Source base: `b346f9426639aa22222b5e45a2e2f44a88bc4cfc` plus the exact dirty
[accepted inputs](acceptance.json), [baseline inputs](baseline-source-inputs.json)
and [source provenance](source-provenance.json). Full generated logs, source copies,
SDK output, executables, ISOs and interrupted runs remain ignored and local.
Use the [runbook](../../../../docs/05-delivery/phase-3/runtime-prerequisites-runbook.md).

## Current runs and artifacts

- Prerequisites: `build/m2/P2-01/prerequisites-1792948703470`; all ten current
  signing/compiler/offline metadata/doctor/Phase 0/source-path/dependency rows pass.
  The reused script's directory name does not make this historical P2 evidence.
- Complete M1: `build/m1/suite-1792948961946196100-42912`; all 33 rows and 42
  distinct offline Q35/TCG/OVMF launches pass their expected classifications.
  [Raw/artifact review](m1-review.json): 20 ready, nine expected fatal, eleven
  expected rejection, two deliberately broken before entry. The broken-image
  timeouts are expected failures, never successful boots. All owned QEMU processes
  were reaped at the fixed 30000 ms deadline; no VM retry/limit extension occurred.
- P3: `build/m3/P3-01/acceptance-1792950911525948100`; all **42 checks pass**.
  [Acceptance](acceptance.json) includes 20 invalid-input rejections, independent
  startup/message layouts and offsets, deterministic target-compiled SDK output,
  baseline/source guards and eight actual host-child/CLI lifecycle checks.
  Each lifecycle reports `reaped`; nonzero exit, missing final record and timeout
  are distinct failures. The admitted Phase 0 ELF is hashed, never executed.

Normal M1 build ID:
`7b7d3f114be7c45d20d160979b29529fa05b283cbd5e7c6ad1136f6771a37097`.
Five consecutive normal boots and the second clean ISO boot pass. Both clean
kernel byte streams match; exact ISO hashes are
`39dc254180972b1a5d5e21f9d6a3bbf79446bb0717ce9a9b1d5f3fe9d8f1d3c7` and
`9d768eab26ee872a4ecf4bfd700f04b8d948dd85eaa5952e5eca3f4fca7e9e87`.
The only ISO difference is the declared session field, not release reproducibility.
Every launch's complete original serial bytes are retained as `launches/*-serial.txt`,
with identity/configuration/artifact/cleanup context alongside them; no raw serial
bytes were normalized or truncated. Full command logs and binaries remain local.

## Criterion review

| P3-01 criterion | Conclusion / evidence |
| --- | --- |
| Current prerequisites and baseline | Pass: [prerequisites](prerequisites.json), [M1 result](m1-result.json), raw launch review |
| Startup/syscall/object/IPC/SDK draft | Pass: [contract](../../../../docs/06-contracts/runtime-v0.md), single JSON source, generated no-std SDK compiles; no ABI numbers frozen |
| Scheduling/locks/IRQ/FP/deadline owners | Pass: single-CPU bounded policies and required P3-02–12 experiments; HPET elapsed-time proposal must pass P3-08 |
| Locked profiles/limits/stress/measurement | Pass: exact input hashes in acceptance; VM capacity and runtime memory remain separate |
| Bounded runner and negative outcomes | Pass: invalid CPU/profile/limits/image/tool/config and actual child faults reject; all eight children reaped |

The [review record](review.json), [hook gap map](hook-gap-map.json) and
[input/history inventory](runtime-input-inventory.json) preserve every actual M2
runtime obligation, including frozen Slint gaps. Historical reviews were read;
their raw bytes were not newly reverified. Current M1 is freshly rerun. Selected
UI artifacts remain historical final-link observations, never guest execution.

## Findings and next boundary

[Development findings](development-findings.json) retain the interrupted sandbox
attempt, overlapping host-fixture deadline failure and subsequent mapped-image
signing-test failure. The signing test now corrupts a separate never-executed
signed copy; the executed artifact is preserved. The final prerequisite and M1
runs use that exact fix. No certificate, enrollment, host policy or Cargo lock
changed. No failed guest launch was retried to produce the passing matrix.

The runtime draft selects bounded kernel tables/page storage, reclaiming owned
allocation, explicit startup/message bytes, trusted provenance and fail-closed
policy. It assigns eager FP isolation to P3-05 and tested elapsed deadlines to
P3-08. These remain engineering defaults requiring their owning guest tests;
the probe arena is not a runtime allocator. ADRs remain Proposed. The 256 MiB
cold-idle target and all guest CPU/input/UI targets remain unaccepted.

Next: **P3-02**, bounded allocation and owned user address spaces. P3-02–P3-14
remain Not started. This bundle grants no remote transfer, PR or publication.
