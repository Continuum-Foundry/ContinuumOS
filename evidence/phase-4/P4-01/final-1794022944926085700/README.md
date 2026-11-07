# P4-01 prerequisite/profile acceptance and M3 follow-up

Status: passed local Codex self-review for P4-01's prerequisite and host planning
subset. The final serial run began 2026-11-06 America/New_York (2026-11-07 UTC).
This is not independent human approval or guest desktop acceptance.

Source: freshly pulled `main`, commit `15e5113b112c179ac9dbe7ad1426a67a2be9f301`,
with dirty implementation hashes in [source-lock.json](source-lock.json) and the
local patch identity in [provenance.json](provenance.json). Three preexisting
Windows signing/policy edits were preserved and excluded from our staged payload.
Full generated logs, binaries, patches, firmware variables and ISOs remain local;
only this compact bundle is retained. No commit, remote publication or policy/power
change is part of this work.

## Results

The [acceptance](acceptance.json) and [final reviewer](review.json) name the exact
three serial stages and artifact identities. There is no stage skip, historical
regression reuse or automatic retry in this final run.

| Coverage | Reviewed result |
| --- | --- |
| Fresh prerequisites | Ten commands, including offline Phase 0, signing/compiler preflights and doctor |
| Complete M1 | 33 rows, 42 launches, ordinary and negative cases, repeated clean optical ISO boot |
| Kernel/runtime | 97 host tests; ordinary, boundary, scheduler, capability and later metadata flavors separately; owned-memory/loader/IPC/wait/supervision/policy/routes/SDK/runtime and selected portable regressions |
| Long M3 stress | Ten guests, five trace-off/on pairs; 100 waves each, 2000 worker lifecycles, 1000 route/fence cycles, 100000 measured IPC round trips and 10000 warmups total; exact settled inventories |
| M3 aggregate | All 23 rows and five deliberate evidence/runner failures |
| P4 host profile | 163 leaf mutations, independent semantic consumer, malformed/unknown/duplicate/oversized input rejection |
| Draft codecs | 1345 paired byte cases, including 541 mandatory negative cases; literal fixtures and independent offset consumer |
| Deadline consumer | Ten mutations of actual build/package/launch/profile metadata reject |
| Host lifecycle | Four real child outcomes (success/nonzero/missing-final/timeout), eight prelaunch rejections; twelve reports, all owned children reaped |
| Device plans | Hash-pinned QEMU capability inventory; 720p/1080p enabled/disabled-dev deltas checked on the host |
| Hosted M2 | All sixteen rows on a fresh signed selected artifact; freestanding link/audit, hook/std negatives, source guards and historical selection verification |
| M2 shared context | Fourteen negative cases; historical UI source/dependency/legal pins preserved |

[Stress guest identities and measurements](stress-guests.json) record all ten
ISO/kernel/build/serial hashes and per-guest bounds/counters. Measured final host
case durations range from 45389 to 51188 ms; these include the case's measured host
interval and are not a guest performance target. Selected original serial records
are in [selected-serial.txt](selected-serial.txt); full raw distributions remain
local and were reopened by the independent reviewers. Hosted performance misses
are preserved in [metrics](m2-metrics.json) and their reviews, not converted to
desktop performance passes.

## Repair and deadline interpretation

Backing classification now scans handles/maps/queued references once per fresh
read-only snapshot, instead of rescanning for each object. The old object-centric
calculation remains an independent oracle across aliases, moves, wrapped queues,
generations/reuse, creator death and teardown. Authority and lifetime semantics
are unchanged. Dispatch timing after a blocked wait now starts at recorded wait
completion; an explicit oracle distinguishes long blocking from a real excessive
ready delay. Every actual sample and the 320-ms bound stay present. Failures now
record kind, actor, wave and value before assertion.

The owner explicitly allowed a deadline change if needed. Repeated original-bound
failures persisted after accounting optimization. The versioned
[`m3-stress-tcg-v2` decision](../../../../docs/05-delivery/phase-3/stress-deadline-profile-v2.md)
sets 60000 ms only for the two long stress flavors. Ordinary, SDK and separate
public-supervision guests remain 30000 ms; normal/negative host children remain
1000/100 ms and deliberate negative guest remains 1 ms. Counts, settlements and
guest-local 320-ms latency checks remain unchanged. Trusted package flavor,
build-input profile hash and independent metadata review constrain the scope.

`original_limit_acceptance` is **false**. This does not replace the historical
[strict M3 acceptance](../../../phase-3/P3-14/2026-11-06/README.md), relabel a
30-second failure, or grant a standing waiver. A new 60-second timeout, guest bound
failure, wrong exit or missing completion still fails.

## Findings and preserved unsuccessful work

The [initial failed-attempt bundle](../2026-11-06-initial/README.md) is immutable.
[Development findings](development-findings.json) identify the original-limit
timeouts, intentional reviewer-repair interruption, unexpected guest panic,
invalid image-hash invocation, rapid timestamp directory collision and later M2
context failure. The selected failed cases/manifests/tails in this bundle retain
their failed status. Full raw logs remain at the recorded local paths.

An exact optimized ISO diagnostic completed in 24120.695 ms and used 22937.5 ms
of QEMU process CPU time; [diagnostic context](cpu-diagnostic.json) and
[raw review](cpu-diagnostic-review.json) are diagnostic only. Phase timestamps
show variable cost, not a proven scheduling, power, thermal or security-tool cause.
The owner reported the PC mostly idle. A later read-only host snapshot is context,
not proof of the earlier failure's cause. The old panic's exact sample was not
available; the dispatch-origin defect does not prove that panic's cause.

The first complete M3/host-plan pass then failed late in M2's shared-context byte
check. The [scoped typed-TOML proof](../../../../docs/05-delivery/phase-2/shared-build-context-review.md)
verifies original pinned Git blobs and permits only two named shared root files:
an unchanged-settings comment and the exact additional kernel-stress profile.
All historical UI/source/assets/locks/notices keep their byte pins. Missing local
history, unrelated changes or type changes reject. The entire gate was rerun on
the repaired final tool source; the earlier M2 attempt stays failed.

## Accepted scope and handoff

P4-01 locks host experiment inputs, clocks, sample counts, bounds, workload cells,
device plans, negative oracles and explicit draft field/lifetime rules. Runtime
ceilings remain `implemented: false` until P4-02 admission/reclamation experiments.
Device help proves availability in pinned QEMU, not a guest driver. Draft host
dev authentication is a trusted Boolean model, not cryptography. No driver,
renderer, physical input path, authenticated transport, guest reload or desktop
efficiency target is accepted. New desktop IDs remain unfrozen; existing SDK hash
`debe57866ed308f46ff2f48f5065e739c7457d0f28d0e8dccc93d42abd4c891b` is unchanged.
P4-02 is next; P4-02 through P4-19 remain Not started.

Reproduce with `pwsh -NoProfile -File tools/Test-P4-01.ps1` in the approved
signer/xorriso context, serially with the host awake. Review the emitted path with
`python tools/p4_acceptance.py review --acceptance <path>`.
[Normalization](normalization.json) records sanitized local paths and original
artifact hashes. [Documentation hashes](documentation.json) bind final prose;
`SHA256SUMS.json` covers the retained members only. Repository retention budgets
are 4 MiB, 200 files and 256 KiB per file. No full local generated run is uploaded.
