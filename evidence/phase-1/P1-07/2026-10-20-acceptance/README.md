# P1-07 acceptance — local review passes

**P1-07 is Done; M1's declared experimental VM baseline passes local acceptance.**
This is Codex's local self-review, not independent approval, a release or a usable
desktop OS. The earlier [blocked review](../2026-10-20/README.md) and its failed
GPT comparison remain immutable. No push, PR, remote build or release occurred.

## Final run and review

`build/m1/suite-1792538416392547700-58840` passed the complete aggregate command
with all **33 required rows and 42 distinct VM launches**. The post-run review
checks every actual ELF/ISO hash, current source hash, fresh firmware-variable
copy, fixed 30-second VM deadline, offline launch arguments, raw version-5 record
identity/sequence, ordered terminal readiness and process reaping. No case was
waived, timeout extended or failed VM retried automatically.

Outcomes: **20 ready**, **9 expected fatal**, **11 expected rejection**, and
**2 deliberately broken before entry**. The latter classes are not successful
boots. Five consecutive normal-kernel boots and the second clean normal build's
exact ISO boot reached final readiness and emulator exit 33. Normal kernels use
explicit `acceptance-exit` configuration, with no Cargo fault-test feature.

All nine host rows pass: formatting, full M0, dependency feasibility, kernel
library tests, doctor, prerequisite rejection checks, documentation, schemas and
governance. The current Rust tests comprise **14 xtask + 14 kernel-library tests**.
Dependency compilation does not establish guest runtime/link closure or select
Slint; the existing allocator-related link limits remain explicit.

The [review](review.json) records all rows and launch classifications. Per-launch
raw serial bytes and compact contexts retain exact ISO/firmware hashes, arguments,
exit/timeout results and host logs. Build inputs/manifests and selected verbatim
assembly excerpts are shared by build flavor. Generated binaries, complete logs,
LLVM dumps and source copies remain local. The review tool's own hash is recorded;
`review-source-inputs.json` records the prepared implementation/procedure revision.
Source base is local commit `70c3ff2` plus the recorded dirty input hashes.

## Two clean artifacts

- Normal build ID: `c654ea0362a6f110415ff4bb4b87c1d5798d00916bed9c73e0834e32f2affe89`.
- Identical kernel SHA-256: `c5fed2f0bc5bd85440c7ed6fb6a29c11b00479c7130e0c35f84e8c765338e8e8`.
- First exact ISO SHA-256: `b5b8bc350db1db4eb31cc354c4de8c2368bed54096aebc5952d7d211e010d050`.
- Second exact ISO SHA-256: `56be3b70ff1999e30e5d04627ef0f06e02eb52602ebfc7ce861b2fcd0668f164`.
- Normalized ISO SHA-256: `89c65d25648f67c9c422d6b5ac0a94423a61555b83e771313f1c60e88264f7b3`.

The full images differ only in the permitted 64 ASCII session bytes at offset
5281941. Both actual images boot; the second target path contains spaces. Fixed
volume/file timestamps and GPT identity remove the earlier unexplained changes.
Actual ISO hashes intentionally differ. This is bounded M1 comparison evidence,
not byte-identical release output or completed R16/M8 reproducibility.

## Acceptance criteria

| Criterion | Local conclusion |
| --- | --- |
| Every required row has raw evidence and a result | Pass: 33 rows, all 42 launches reviewed |
| Exact final offline ISOs and pinned profile | Pass: six final normal launches plus full regression matrix |
| Fixed timeout policy | Pass: 30000 ms in every VM manifest; no after-failure extension |
| Honest outcomes and cleanup | Pass: ready/fatal/rejection/broken distinctions, every owned QEMU reaped |
| Source identity, notices and architecture boundary | Pass within declared baseline; no new kernel dependency or assembly implementation |
| Consistent review and handoff | Pass: story, index, requirements, milestone, validation and agent handoffs updated |

The host execution blocker was resolved through the owner-authorized local
development signer and App Control policy, not an unsigned directory exception.
See the [completed signing review](../../../host/windows-dev-signing/2026-10-20-completed/README.md)
and [lifecycle runbook](../../../../docs/05-delivery/windows-development-signing.md).
The original canceled rollout and runner quoting failure are retained separately.

## Boundaries

This accepts only the pinned single-core Q35/TCG/OVMF boot, checked metadata,
exceptions, memory/guards and interrupt/uptime baseline. There is no general heap,
scheduler, user-mode isolation, desktop, persistence, live state transfer, physical
hardware qualification, Secure Boot result or measured cold-idle/performance claim.
The 256 MiB target remains a target. B06/B32 remain partial and ADRs stay Proposed.
The next bounded workstream is M2's hosted Slint/minimal-custom comparison.
