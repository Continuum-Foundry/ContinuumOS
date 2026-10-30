# P3-09 — Bounded user-mode supervision passes locally

**P3-09 is Done under local Codex self-review for its declared experimental subset.**
This is not external approval, a persistent service manager or the M3 exit. See
[criterion and unsafe review](review.json), the [supervision runbook](../../../../docs/05-delivery/phase-3/supervision-runbook.md)
and [ADR-0025](../../../../docs/04-decisions/0025-prepaid-subtrees-and-quiescent-process-reaping.md), which remains Proposed.

P3-09 passes local self-review for bounded user-mode supervision: 29 dynamic admissions/reaps across four bootstrap epochs, exact lifecycle outcomes, prepaid ancestor quotas, authority/priority/inheritance denial, parked/orphan/supervisor teardown, two retry permits and quarantine. All 73 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel/wait regressions pass. Four epochs settle all object/frame/table/stack/pin/wait/tree inventories exactly. One thread/process, equal priority and eight lifetime units per epoch are the admitted subset; physical reclamation does not refund credits. Persistent supervision, current policy/session enforcement, independent SDK freeze and guest UI/reload remain open; P3-10 is next.

The trusted Rust supervisor runs at CPL3 from an actual standalone optical ISO.
Its copied create/start/terminate/join calls control dynamically scheduled children.
Kernel emergency handling remains separate. Children derive the current parent's
principal/session and narrow grants; bootstrap installs only their own typed
process/thread handles. Nonempty inheritance, service-role selection and higher
priority/resource ceilings reject. Monotonic instance tokens never retarget a
recycled process slot; only parent or trusted supervisor controls a child.

## Observations and provenance

Source base: `65e161b016d4a98f15d21d3f4b261e43486e1c0e` plus [dirty input hashes](source-provenance.json).
The measured source includes the owner's three pre-existing signing-policy edits;
those files remain outside this change. No host policy or power setting changed.
Final documentation hashes are separate from the measured runtime/build snapshot.

Build ID: `53a8154a0ec04afe389e7f2541daba3fb6e5c2785ac198ad7b46f028983c64a8`.
Kernel SHA-256: `930b9a52dc3bea343dd0a8c1a3625d7faec8259127a5812c32b95d1c348ad345`.
Launched ISO SHA-256: `c8089092ee26163bf337459855bb74ea24eb7046534424d7f1e6700c8d57795f`.
Raw serial SHA-256: `d30557d17680fd4c334368ea83f40126a7331880861d1ff3c82e9c001ea5393b`.

The final guest records 29 admissions/reaps, 21 joins, 193 dispatches,
four parked-owner kills, two retry permits and one quarantine denial. Four exact
settlements leave zero objects/handles/mappings/backing/frames/spaces/allocations/
tables/stacks/pins/waits/tree nodes. Conservative fixed-runtime precharge is
10583088 bytes, not whole-guest idle memory. CPU hogs are actually preempted
while healthy tasks complete. Prepared tasks never dispatch before start;
stopped/reaped generations never dispatch or resume a waiter.

All raw supervisor serial bytes are retained in numbered `supervisor-serial-*.txt`
chunks; concatenate numerically to recover the serial hash. [Launch/package context](launch-context.json)
records one-vCPU Q35/TCG/OVMF, isolated optical boot, unchanged 30000-ms guest
bounds and reaped host children. Full binaries, ISOs, disassembly, regression
logs and generated source copies remain local under ignored build/work. Fresh
checkouts do not contain those full artifacts.

All 73 host tests, ten fresh prerequisites, complete 33-row/42-launch M1 and
separate narrower regression flavors pass. Early seven-flavor metadata checking
is developmental and does not replace those guest builds/runs. The combined
raw/artifact consumer rejects 174 evidence mutations.
[Development failures](development-findings.json) remain explicit and do not count
as acceptance. No interrupted run, changed deadline or host timeout is counted.

## Limits and handoff

Each epoch prepays eight lifetime units including its control owner, permitting
at most seven child admissions. Delegation spends a parent's remaining units;
exit, free, close, receive and join do not refund credits. Physical resources
still reclaim exactly. This keeps retained transfers and dead-parent lineage
charged without claiming reusable elastic quotas. Per-unit backing/object/handle/
mapping/message/timer/wait bounds and reserved control capacity are in ADR-0025.
Diagnostics and validated-call counts are bounded.

One CPU, one thread per process, equal priority, immutable named ISO images and
empty inheritance are the admitted profile. Supervisor failure closes admission
and terminates descendants; only explicit trusted bootstrap after complete
settlement can restore a fresh session. A restart preserves no application work.
The last epoch remains closed; `m1.ready` means experiment completion, not live
service readiness. Persistent recovery, broker policy/session liveness, SDK freeze,
SMP, guest UI/reload and desktop efficiency remain open. P3-05's native x87 debug
metadata and unmasked-SSE limitations remain. P3-10 is next.
