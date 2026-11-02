# P3-13 — Bounded stress and resource accounting pass locally

**P3-13 is Done under local Codex self-review for this declared subset.**
P3-14/M3 exit remains open. [Criterion and unsafe review](review.json),
[runbook](../../../../docs/05-delivery/phase-3/stress-runbook.md) and
[ADR-0029](../../../../docs/04-decisions/0029-bounded-runtime-stress-observation.md) define the scope.

P3-13 passes local Codex self-review for bounded SDK stress and resource accounting. Ten continuous guests complete 100 waves each: 2000 worker lifecycles, 1000 route/fence cycles, 100000 measured IPC round trips and 10000 warmups, with healthy control progress and exact settled inventories. A separate public supervision guest preserves 29 admissions across four epochs. The final-source regressions pass 95 kernel host tests, ten prerequisites, complete M1 (33 rows/42 launches), prior runtime/SDK/portable checks and 14 separate metadata flavors. The optimized native observer retains the original 30000-ms deadline and all validation; regression reuse requires identical measured source and compiler hashes. Only the existing exact SDK contract subset remains frozen. P3-14 is next; M3 exit, guest UI/reload and desktop efficiency targets remain open.

## Counts, faults and reclamation

Five fresh one-vCPU Q35/TCG/OVMF guests per instrumentation profile run the fixed
seed 20261025, 100 two-worker waves and 100 seven-decision route cycles apiece.
The healthy control process remains alive throughout each guest. Expected outcomes
cover OOM/handle exhaustion and reuse, pressured authority transfer, cancellation
and actual parked-owner termination, hardware page faults, timer-preempted CPU hogs,
queued peer/broker death, combined pressure and deferred death during real live
map/transfer copy pins. Public supervision independently preserves its original
four-message/prepaid quotas over 29 admissions and four bootstrap epochs.

Every staged/death/settled inventory is independently reconciled: physical backing
is counted once, foreign mapping quota charges are separate, and owner/orphan rows
reconcile to global resident and committed totals. All 100 settled baselines per
guest are identical. Final owned frames, objects, handles, waits, timers and pending
work are zero. There is no positive retained-object/frame tolerance.
The three immutable verified-image plans retain 1584 bytes, below the
4096-byte ceiling, charged within the kernel image. Preallocated tables/stacks,
allocator metadata, observer buffers and unused capacity remain charged through
the containing image; active subsets are not added twice. Full accounting series
are retained for repetition zero of both profiles; every repetition was reviewed.
The allocatable eligible pool is separate from reserved/unmanaged capacity;
only its unallocated frames are counted as genuinely free eligible RAM. Reserved
capacity is not a claim that those bytes are allocated or currently reclaimable.

## Measured intervals

Guest HPET nanoseconds, converted to milliseconds here; nearest-rank quantiles:

| Trace | IPC samples | p50 ms | p95 ms | p99 ms | max ms |
| --- | --- | --- | --- | --- | --- |
| off | 50000 | 0.2807 | 0.5766 | 1.2451 | 3.9407 |
| on | 50000 | 0.3100 | 0.6629 | 1.3695 | 33.5816 |

All 100000 measured samples are retained in `off-*/ipc.csv` and `on-*/ipc.csv`;
10000 warmup exchanges are checked separately and excluded from these quantiles.
The maximum conservative interrupt-off bound is 93.7249 ms, below the fixed
320-ms ceiling. Dispatch/wake samples retain every 32nd observation, with complete
observation counts and maxima; their quantiles do not describe an unsampled tail.
Timer lateness, queued expiry, per-guest rates, phase spans and both diagnostic/clock
cost probes are in the review. Each guest settles through at least 64 actual halts,
64 ticks and 320 ms; export then runs interruptibly after quiescence.

The median paired trace-on/off active guest elapsed ratio is
1.006685. Both profiles retain mandatory
timing/outcome instrumentation; trace-on adds read-only inventory observation.
Host VM elapsed ranges from 15259 to 23342 ms, checked against the unchanged
30000-ms deadline. Host and guest intervals remain separate. These are elapsed
TCG observations on a 2-GiB capacity profile, not guest CPU/RSS or desktop efficiency results.

## Exact source, observer and regressions

Base commit: `be11413128531cb38e114cca7024841a1834f85e` plus the measured dirty input hashes in
[source provenance](source-provenance.json). Public SDK workers are built locked/offline
without kernel sources. A source/compiler-keyed host cache supplies one exact audited
ELF to all roles and both profiles; each ISO's extracted bytes and compiler inputs
are checked. The guest immutable-plan cache is a separate mechanism.
The frozen SDK contract remains `debe57866ed308f46ff2f48f5065e739c7457d0f28d0e8dccc93d42abd4c891b`.

The complete final-source P3-12 regression gate passed before stress. The later
stress gate reused it only after verifying the complete measured source set and
compiler hash and rerunning its independent raw review. It includes 95 kernel
host tests, ten fresh prerequisites, full M1 (33 rows/42 launches), all prior runtime
regressions, independent SDK codecs/runtime and affected portable checks.
Fourteen separate metadata flavor checks supplement the actual separate guest builds.
The optimized native observer passes 29 host tests; its one pre-existing font test
is outside this runtime gate. The final report checks its exact executable SHA-256.

The same observer source is built in release mode because each 20-ms poll validates
the complete growing serial tree. Polling, validation and deadlines are unchanged.
An earlier debug-observer final attempt timed out after 66 waves despite identical
guest build IDs. Timeout alone does not establish a root cause. The optimized
observer experiment passes all ten fresh repetitions; no interrupted or failed run
counts as acceptance. [Development findings](development-findings.json) and
[selected failure traces](failure-selection.json) retain failures, artifact hashes
and remediation, including the host stack requirement and script path error.
The focused reviewer rejects 46 mutations per stress guest (460 total); public
supervision also reruns its original independent outcome/mutation oracle.

## Unsafe-code review and limits

Registered fixture hooks preserve shared dispatch, checked return state and copy
observers. No public operation selects a return IP, selectors or privileged flags.
Actual nested timer interrupts request death during map/transfer copy pins; commit
finishes, pins reach zero and trusted reap follows. No registry borrow or kernel
continuation crosses a task switch. Single-CPU observer arrays are bounded and
overflow fails. Inline-only channel traffic skips authority collection only after
successful zero-attachment commits; pin quiescence remains mandatory.

MXCSR is captured once in the guarded syscall-entry prefix before TS, validated
before each operation and confined by actual instruction audits; full FP saves
remain eager at switch/termination. Integer-only entry omits STMXCSR. Constant-zero
segment bases are measured with FSGSBASE disabled, zero GDT bases, restored null
selectors and no exposed TLS/MSR writer. Immutable launch plans cannot admit mutable
or replaceable module bytes; all per-launch quota/generation/staging checks remain.
The final-source scheduling/capability/runtime regressions cover these changes.

No desktop cold-idle 256-MiB, frame/input latency, guest display, state migration,
live reload, durability, physical hardware or broad hostile-code certification is
accepted. Full serial streams, source copies, logs, disassembly, ELFs and ISOs stay
local under ignored build/work. Selected records identify their complete local
hashes and selection rules. Owner signing/display-roadmap edits are preserved and
excluded from the staged task payload; only the ADR count and explicit acceptance
appendices touch shared owner files. This bundle grants no upload permission.
