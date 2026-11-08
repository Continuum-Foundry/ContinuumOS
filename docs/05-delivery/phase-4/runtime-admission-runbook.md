# P4-02 runtime admission

Status: passes [local Codex self-review](../../../evidence/phase-4/P4-02/final-1794169067563019600/README.md) for the exact bounded runtime subset. This is not external approval or desktop rendering acceptance.

The versioned [runtime profile](../../../config/p4-02-runtime.json) specializes
P4-01's proposed capacities. Ordinary `cargo xtask build --profile dev`,
`cargo xtask iso --profile m1` and `cargo xtask run --profile vm-baseline` select
`desktop-runtime`; the old no-feature binary remains a minimal metadata build.
Every historical M1/M3 test flavor is built separately without that feature.
The SDK's public IDs, maximum 64 KiB memory-object request and 64 KiB optional
SDK heap stay unchanged. Desktop heap backing is a trusted launch allocation,
not a new public memory call or an independently frozen desktop ABI.

## Admission and ownership

Seven named roles have one private image/address space and one owning execution
thread each. Supervisor, compositor, input, shell, editor, update and policy have
2/48/2/16/64/32/4 MiB fixed private heaps respectively. These are runtime scaffolds:
they run allocation/health work, not compositor/input/editor/security features.
One trusted static payload is linked into three independently audited boot modules;
role bindings come from the kernel's finite role table, never the payload.

Each owner receives only self-process, self-thread and timer handles. The runtime
attenuates bootstrap handles before user entry: process/thread retain CONTROL
only (256), and the timer retains QUERY plus CONTROL (257). CREATE, DUPLICATE and
TRANSFER bootstrap rights are closed before publishing the read-only record.
Temporary duplication stays below the declared live-handle ceiling; the final
graph holds exactly 21 handles and 21 objects. The runtime
dispatcher admits ABI query, bounded diagnostic strings, checked exit/yield,
clock queries, timer arm and wait. Other known object/IPC/supervision operations
return AccessDenied. Unknown operations and the two legacy fixture allocation
IDs return Unsupported, preserving the frozen SDK error contract. No endpoint,
device, update, policy-administration or account
authority exists yet. Session 1001 and principals 400–406 are synthetic trusted
bootstrap bindings, not real accounts or authentication.

The graph is a separate trusted bootstrap envelope, not a change to M3's public
eight-unit subtree. Its 24 prepaid lifetime units reserve three for recovery;
each role admits at most its initial owner and two retries. Successful preparation
spends a unit even if a later owner fails before the graph starts. Reaping restores
physical backing, slots and handles; it never refunds consumed lifetime units.
No resource replacement creates helper processes. The 100 cycles exercise heap reuse; live resource reload remains P4-15.

The sum of role heaps is 168 MiB. A 2 MiB per-role overhead envelope covers loaded
segments/BSS, user and kernel stacks, configuration, table banks and metadata.
All image pages and heap backing are charged once before publication. The adapter
checks physical availability for 182 MiB plus a withheld 16 MiB recovery reserve,
then limits its dynamic frame pool to 182 MiB. Fixed kernel image/table/stack
storage is separately counted and already excluded from eligible physical RAM.
The 384 MiB active/staging profile is an upper proposal; this story admits only
the smaller fixed graph and no staging service buffers.

Desktop builds explicitly use a 128 MiB per-process backend ceiling, 128 tables
per bank, two banks, 16 owners, 256 allocation slots and 128 map records per owner.
Actual launched owners are further constrained to 16 lifetime allocations and 16
live mappings; their handle tables have a 12-live-handle ceiling with three grants.
The fixed runtime storage check is 32 MiB. Legacy test features retain 64 MiB
per-process, 64-table banks and their original fixed bounds. This changes no M3
guest deadline, frozen SDK operation or process-thread policy.

Desktop and ordinary graph builds use the existing `kernel-stress` optimization
profile (`opt-level = 3`). This profile name does not select a longer guest
deadline: those builds remain 30000 ms. Only the two named M3 stress flavors
retain the already approved revision-2 60000-ms limit.

Private RW heap mappings retain authority until complete owner teardown. There
are no shared heaps, callbacks, signals or additional threads. Killing the owner
first leaves its continuation, restores kernel CR3 and clears the scheduler slot;
then waits, handles, tables and backing are destroyed. No cross-process data/code
retirement or universal revocation claim follows.

## Allocation and lifecycle

The extent allocator stores at most 512 sorted live blocks, scans at most 513
gaps and immediately makes adjacent freed gaps reusable. Alignment is at most
4096; no bump-only arena, per-allocation kernel request or metadata allocation is
used. Every returned region is zeroed. Realloc allocates/zeros/copies before freeing
the old block; failure leaves the old bytes and allocation intact. Fragmentation
may deny a request despite aggregate free space; callers must admit fallibly.

Initial graph preparation is transactional. A failed image, grant or heap stage
releases its partial owner; a later-role failure reaps every earlier prepared owner.
Ordinary restart preparation failure quarantines that role while other owners
continue. Trusted bootstrap currently owns this finite graph and watchdog in the
kernel; the supervisor process is a control/health scaffold. This does not replace
the accepted M3 public user-mode supervision implementation or claim a general
desktop service manager. Device/route authority and richer supervision stay gated.

Owner residence is sampled with elapsed HPET across each real dispatch, including
kernel work. A role that accumulates 1 second without yield/parking is stopped;
retry backoff is 20 ms, with quarantine after two retries. This is an elapsed
watchdog, not validated CPU billing or a performance target. Actual ready delays
use wait-completion time for blocked owners and retain the 320 ms bound. A timer
wait that reaches its original 100 ms deadline reports TimedOut and the scaffold
handles that outcome; no timeout is converted into a successful wake.

Restart cleanup, backoff and preparation are separate scheduler steps. Each ready
owner gets a scheduling turn before a new admission; backoff does not busy-wait over them. If
admission cannot obtain an opportunity within 320 ms after backoff, it quarantines
the role instead of waiting indefinitely. Each cleanup retains its 320-ms bound.
Three immutable boot images are hash-verified and parsed once before user entry,
using the existing VerifiedImage contract. Every launch still checks its own
context, ceiling, tables, stack and cleanup transaction. Boot-module backing stays
reserved, RO/NX and without writable aliases; this cache admits no dynamic code.

The adapter never suspends a Rust borrow, copy pin or partially published image
to run a user process. Explicit preparation/cleanup samples and named failing
bounds remain diagnostics, not a hard real-time guarantee or proof of a timeout's
cause. The retained 414.333-ms ready-delay failure led to the scheduler-step repair;
the earlier empty-serial guest timeout and host-build timeout have separate,
unproven causes.

The versioned `desktop-wait-summary-v1` diagnostic policy keeps every wait result
(including TimedOut), resume/latency sample, heap completion, lifecycle, authority
and resource outcome. It omits redundant park/complete/wake formatting and routine
clock/arm confirmations. All operations, counters and bound checks still execute.
Earlier verbose-log deadline failures remain failed evidence; no deadline or heap
workload count was changed to obtain acceptance.

## Reproduction and review

Run serially in the approved signer/xorriso execution context with the host awake:

```powershell
pwsh -NoProfile -File tools/Test-P4-02.ps1 -Manifest config/p4-02-validation.json
```

The owner later requested slimmer current P4-02 validation on 2026-11-08,
superseding the earlier NEXT-time-only direction. The selection keeps both host
suites, five desktop guests, ordinary boot without acceptance observers and sixteen
separate metadata flavors, with boundary, scheduler, capabilities, waits and
SDK-runtime guest regressions. The current closeout carries completed new-behavior
components forward only after reopening their raw artifacts and verifying unchanged
guest implementation. The source-lock deltas are the selection JSON and two
host-reviewer files correcting the focused boundary adapter to the existing
P3-05/P3-12 exact 21/16 scratch/frame layout. Historical P3-04 keeps its default. Failed or
interrupted complete gates remain non-passes. This is story acceptance, not fresh
M1/M3 milestone acceptance. Fresh reproduction uses the command above and
`python tools/p4_runtime_acceptance.py review --acceptance <acceptance.json>`.
`-FullGate` remains the optional original complete serial process; it is not the
current story closeout requirement.
Individual guest diagnosis uses:

```powershell
. tools/Enter-Dev.ps1
cargo xtask test --suite m1 --case desktop-runtime
python tools/Review-P4-02.py --run <reported-M1-directory>
```

The independent reviewer reopens actual ISO/kernel/user ELF bytes and raw serial,
checks trusted bindings and every lifecycle, reconciles loaded pages plus stacks,
config and heap with physical inventories, and mutates criterion-closing fields.
`--development` skips current-source verification and is diagnostic only. A failed,
timed-out, interrupted, stale-source or missing-completion run cannot pass the gate.

The guest matrix includes image/grant/heap partial failures, a whole-graph abort,
large surface/font/control allocations, alignment, zero reuse, failed realloc,
fragmentation/coalescing, 512-slot pressure, 100 heap cycles per executed owner,
normal exit, explicit handled-OOM exit, panic, a real CPU hog, two replacements,
quarantine, healthy control progress and exact final backing/object/frame teardown.
Only the exact tested subset can be accepted. P4-03 receives bounded role resources;
P4-07 must still validate real toolkit/font workloads. Rendering, physical input,
authenticated development transport, live reload and the 256 MiB whole-desktop
target remain open.
