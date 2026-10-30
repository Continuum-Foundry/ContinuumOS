# P3-09 supervision and resource ceilings

Status: Done under local Codex self-review for the declared experimental subset.
Prerequisite: P3-08's retained local review passes. This is an experimental bounded
runtime profile, not a persistent service manager or frozen SDK.

## Scope and contract

The original Rust kernel loads a trusted static Rust supervisor from the ISO and
runs it at CPL3. Its children are created by user calls and scheduled dynamically.
The kernel validates trusted current identity, structural resource ceilings and
publication. The user supervisor implements launch/observe/kill/join and retry
control. Kernel faults retain the existing emergency path. Details and the
conservative lifetime-credit tradeoff are in
[ADR-0025](../../04-decisions/0025-prepaid-subtrees-and-quiescent-process-reaping.md).

Dispatch IDs 27–33 are experimental. `process_create(pointer,96)` takes twelve
little-endian u64 words: image selector, application mode, principal, session,
service boolean, grant ceiling, memory ceiling, subtree units, threads, priority,
inherited-handle count, reserved zero. Images 1/2 are trusted ISO children; image
0 is reserved for kernel bootstrap. Identity must match the current caller;
service and inheritance are zero, threads one, priority zero, memory 1048576.
Grant bits may only narrow. A successful return is a fresh Prepared token. Mode
is an application argument and grants no authority. No user return IP, selector,
stack address, privileged flags or executable bytes appear in this request.

`process_start(token)`, `process_terminate(token)` and `process_join(token)` require
direct-parent or bootstrap-supervisor control. Join returns the original exit
code after teardown, or WouldBlock until reaped and descendants joined. Raw fault
vectors accompany the correlated kernel lifecycle record. `process_state(token)`
queries Prepared=0, Running=1, Stopping=2, Reaped=3 for self or supervisor targets.
`process_first_child(parent)` returns the lowest outstanding child token or zero
for self or the trusted supervisor; it includes reaped children awaiting join.
`supervisor_restart()` obtains a bounded retry permit, with WouldBlock for backoff
and QuotaExceeded for quarantine. Ordinary child callers are denied. Old tokens
return StaleEpoch; tokens do not become authority for other parents.

Each epoch has eight precharged units including its supervisor. A child must
reserve all units it can delegate, before publication. Spent units never return
within the epoch, including after join. Resources reclaimed physically can be
reused by another admitted owner, but cannot mint new lifetime credits. Actual
limits and the reserve proof are listed in ADR-0025 and `config/m3-profiles.json`.
The fixed resource ceilings constrain the backend and registry before commit,
including duplicate/receive handle installation, new object/timer/pair creation,
allocation/map admission and atomic message/attachment enqueue.

## Development and final validation

Keep the host awake and run serially in the existing signer/xorriso context:

```powershell
. ./tools/Enter-Dev.ps1
cargo xtask test --suite m1 --case supervisor
python tools/Review-P3-09.py --supervisor <printed-directory>
pwsh -NoProfile -File tools/Test-P3-09.ps1
python tools/Review-P3-09.py --acceptance <printed-acceptance.json>
```

The registered case builds, independently audits ELF/codegen, packages three
static modules into a standalone optical ISO and executes the bounded QEMU
guest. It is not a compile-only matrix. `tools/Check-P3-09Flavors.ps1` separately
performs metadata-only checks with already generated trusted descriptors for
ordinary/boundary/scheduler/capability/channel/wait/supervisor features. Its
placeholder build ID is never packaged or booted. Final acceptance reruns the
ten prerequisites, full M1 and separate regression case builds/guest launches.

The four supervision epochs cover normal exit, process faults, panic/OOM reports,
prepared-child kill, CPU-hog termination with healthy progress, individual resource
ceilings, precharged descendant fan-out, explicit identity/authority/thread/priority
and inheritance rejection, stale targeting, two retries/quarantine, parked-child
kill, parent death and supervisor fault with queued authority and prepared/live
orphans. A kernel continuation never survives a wait or a process death. Admission
stays closed after supervisor failure until explicit trusted bootstrap; the last
epoch remains closed. The harness's `m1.ready` means experiment completion, not
production service readiness after supervisor failure.

Preserve startup errors, host-preflight failures and guest deadlines distinctly.
Do not change deadlines or count interrupted runs as acceptance. Full local logs,
binaries, ISO payloads and generated source copies remain ignored under build/work.
Promote compact reviewed traces, hashes, launch context and review only after
[retention checks](../evidence-retention.md). M3 final stress, policy liveness,
independent ABI, guest UI, reload and whole-guest efficiency remain later work.


## Accepted local observations — 2026-10-30

P3-09 passes local self-review for bounded user-mode supervision: 29 dynamic admissions/reaps across four bootstrap epochs, exact lifecycle outcomes, prepaid ancestor quotas, authority/priority/inheritance denial, parked/orphan/supervisor teardown, two retry permits and quarantine. All 73 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel/wait regressions pass. Four epochs settle all object/frame/table/stack/pin/wait/tree inventories exactly. One thread/process, equal priority and eight lifetime units per epoch are the admitted subset; physical reclamation does not refund credits. Persistent supervision, current policy/session enforcement, independent SDK freeze and guest UI/reload remain open; P3-10 is next.

The final probe records 193 dispatches, 21 successful joins, four parked-owner kills, two retry permits and one quarantine denial. Conservative fixed-runtime precharge is 10583088 bytes, below the 16 MiB ceiling; this is not whole-guest idle memory. The combined raw/artifact consumer rejects 174 evidence mutations. See the [compact evidence](../../../evidence/phase-3/P3-09/2026-10-30/README.md); complete generated regression artifacts remain local.
