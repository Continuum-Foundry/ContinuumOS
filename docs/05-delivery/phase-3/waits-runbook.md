# P3-08 waits and elapsed deadlines

Status: P3-08 Done under local Codex self-review for the declared experimental subset. P3-07's local
review is the prerequisite. Public ABI IDs remain experimental until P3-12.

## Integrated runtime and local acceptance

The implementation now includes `wait-test` guest execution, a checked HPET
elapsed service, real scheduler parking and FIFO wake admission, manual events,
timer arm/cancel, process/channel readiness and bounded channel caller helpers.
The current six-wave probe runs 18 tasks. It exercises all three registration
boundaries for signal/enqueue/close, simultaneous cancellation/delivery and late
cancellation, delayed IRQ delivery, bounded request cycles, a parked-owner kill,
peer death and pressured bytes/authority transfer. Earlier development snapshots
below are retained as history; their exclusions describe those earlier slices.

The implementation and wire decisions are in
[ADR-0024](../../04-decisions/0024-elapsed-clock-and-bounded-wait-completion.md)
and the `waits` section of `contracts/runtime-v0.json`. Public dispatch IDs 17–26
are experimental. `clock_now` returns elapsed nanoseconds; `clock_resolution`
returns ceiling HPET tick resolution, not wake granularity. `event_create` uses
the process handle and a boolean; `event_set` uses an event handle and a boolean.
`timer_create` uses the process handle; `timer_arm` uses a timer handle and a
24-byte pointer/length; `timer_cancel` uses the timer handle. `wait_many` uses a
32-byte pointer/length. `channel_send_until` / `channel_receive_until` use an
endpoint plus a 24-byte pointer/length. Register validation and reserved/version
checks remain shared with existing dispatch, with no caller-selected return state.
The legacy startup record is unchanged, including its zero clock-resolution word;
the admitted wait profile queries `clock_resolution` explicitly. Startup feature
advertisement and independent SDK compatibility are not frozen by this story.

Wire records contain little-endian u64 fields, independently encoded by the
caller. A wait request is `[items_pointer,count,absolute_deadline,cancel_event]`;
each 32-byte item is `[handle,interest,required_bytes,required_attachments]`.
Interests are signal=0, exit=1, readable=2 and writable=3; non-writable capacity
fields must be zero. Empty/oversized/duplicate sets reject before registration.
Timer arm is `[absolute_deadline,period_or_zero,reserved_zero]`. A channel attempt
is `[original_descriptor_pointer,absolute_deadline,cancel_event]`, referencing
P3-07's unchanged 32-byte send / 48-byte receive descriptor. `u64::MAX` means an
infinite operation deadline and cannot be an armed timer deadline. Other values
are relative to this boot's HPET epoch, never wall time. Cancel handle zero means
no cancellation; a nonzero handle must be a queryable event.

Completion is recorded before a mutating caller can return and reset readiness;
supervisor polling later admits the completed task without changing its outcome.
The signal/reset-before-admission and public cancellation-pulse regressions verify
that notifications do not depend on a scheduler turn between those operations.
Cancellation uses a manual-reset event: a controller must retain the request
until the caller confirms its outcome. A reset deliberately changes future
registrations, including a caller helper between an attempt and its next wait;
it does not erase a completion already recorded for a registered waiter.

The no-std helpers in `experiments/user-images/wait_api.rs` compose deadline-
checked atomic attempts and readiness waits using one absolute deadline. They
retain caller buffers on the owning user stack while the kernel retains only
copied watch handles; no caller span/pin or kernel continuation survives parking.
An accepted send moves authority exactly once. Timed-out/cancelled attempts do
not move it. Receive failures preserve queue/install state; partial output bytes
remain unspecified. The final check is at the precommit decision after fallible
work; it is not a promise that service effects occur before the deadline.

IRQ-off clock comparison and the deliberately delayed wait use HPET timestamps
while delivered uptime ticks remain unchanged. Counter wrap, fractional
accumulation, overflow and periodic phase/coalescing are covered by host tests.
Guest wake observations measure readiness observation to **dispatch preparation**,
before diagnostic output and IRETQ; they do not measure first user instruction or
input/display latency. Observer cost is measured separately using 100 reads. No
cross-machine clock subtraction or hard-real-time target is accepted.

Run final-source acceptance serially with the host awake in the existing signer/
xorriso context; no Windows power/security setting changes are required:

```powershell
pwsh -NoProfile -File tools/Test-P3-08.ps1
python tools/Review-P3-08.py --acceptance <printed-acceptance.json>
# Development probe only, not the final story gate:
. ./tools/Enter-Dev.ps1
cargo xtask test --suite m1 --case waits
python tools/Review-P3-08.py --waits <printed-waits-directory>
```

The script reruns all ten prerequisites, complete M1 and separate memory, loader,
boundary, scheduler, capability, channel and wait build/package/guest commands.
Early direct `cargo check` invocations checked the six Rust binary flavors using
the prior probe's trusted generated descriptors and IDs. These metadata checks
did not link, package or launch guests and do not replace the registered xtask
case commands, which still combine build/package/guest execution. Their local
results are `build/m3/P3-08/integration/early-flavors.json`. Full logs/source copies remain
ignored; promote only a compact reviewed bundle following retention policy.

## Initial internal slice (historical development snapshot)

`kernel/src/wait.rs` provides allocation-free policy only: an HPET counter epoch,
typed absolute instants, 32 generation-bound wait records with up to eight item
observations each, immutable completion, explicit cancellation request/confirmation,
termination cleanup and level periodic/one-shot timer arithmetic. The module has
no unsafe code. It does not store object references or user pointers. The trusted
adapter must supply observations in a single exclusion interval and preserve
watched identity/order; this module alone cannot prevent lost wakeups.

`kernel/src/objects/waiting.rs` adds internal rights-bearing manual-reset events
and timers, plus bounded watch sets for event/timer signal, process/thread exit
and channel readable/writable pressure. QUERY authorizes readiness observation;
channel directions additionally require RECEIVE/SEND. CONTROL authorizes event
set/reset and timer arm/cancel. CREATE on the caller's process authorizes creation.
New event/timer handles carry BASIC, CONTROL and TRANSFER; duplicates attenuate
normally. No public syscall IDs are added yet. Exact watched handle nonces and
owner generations prevent close/reuse from retargeting a wait. Watch sets hold
no object references, so an exact watched handle closing reports Closed even
when a duplicate or queued transfer keeps the object alive. Retained process
queries can observe exit without rebinding to a reused owner slot.

Send readiness checks requested inline-byte and attachment capacity without
reserving either. Receive readiness keeps queued messages drainable after peer
closure; it reports Closed only after that queue drains. Existing atomic
send/receive commit rules remain in force. Event readiness is level-triggered;
observing it never resets the event. Multiple wait records may observe that level.

The tests cover arithmetic wrap/overflow and fractional accumulation, delayed
elapsed observation, all combinations of closed/cancel/deadline/readiness,
immutable completion, capacity/stale identity, multiple level waiters, timer
coalescing/rearm/cancel and synthetic cycle record cleanup. Synthetic records do
not prove guest deadlock recovery or control-task scheduling. Object tests add
rights/type/duplicate/bounds denials, exact stale-watch behavior, manual reset,
timer cancel/rearm, process exit/reuse and non-consuming channel-pressure/close
observations. No public ABI, scheduler parking, blocking channel operation or hardware clock
adapter is implemented by this slice. Those remain required below.

Reproduce the internal checks with workspace-local Rust and the enrolled signer:

```powershell
. ./tools/Enter-Dev.ps1
$env:RUST_TEST_THREADS = '1'
$env:RUST_MIN_STACK = '16777216'
cargo test --locked --offline -p continuum-kernel --lib
cargo check --locked --offline -p continuum-kernel --lib --target x86_64-unknown-none
```

These are host tests and a freestanding library compile, respectively. They are
not a compile-only kernel flavor matrix, guest boot or story acceptance.

## Local development checks — 2026-10-29

Base: `fbfb89100fa19a488df8ded620a9ffaaa9483af9` on `main`, with this dirty
internal slice and the owner's three pre-existing signing-policy edits preserved.
No host policy was deployed. All 63 kernel host tests pass, including nine new
wait-policy tests and seven object-readiness tests. The freestanding library check
passes. Final logs remain local in
`build/m3/P3-08/policy-core/object-host-tests-4.log` and
`build/m3/P3-08/policy-core/object-freestanding-check.log`. The earlier object-test
compile failure from private sibling imports remains in `object-host-tests.log`;
the imports were corrected before the successful runs. No P3-08 wait syscall,
elapsed-clock hardware adapter or blocking guest result is claimed.

Existing capability and channel guest regressions pass serially, with separate
flavor builds, unchanged 30000-ms deadlines, emulator-test exit 33 and confirmed
reaping. Capability regression: 18 tasks/six waves, exact settlement, local run
`build/m1/capabilities-1793332763640914100-32324`, build ID
`e6015dce2397d9d1d40d920ffd45eed86f3b807c7fe56775f397a6990bf97d13`.
Channel regression: six tasks/two waves, exact settlement, local run
`build/m1/channels-1793332925251014900-34592`, build ID
`ec23e63778d2356007cb1f5ea6d23998abe673fb2f82e4ed90e67a996e1c39eb`.
Each run retains its actual ISO/kernel hashes, launch arguments, source inputs
and complete serial. The runtime precharge remains 10559512 bytes. These are
regressions of the existing nonblocking subset, not execution of the new waits.
Complete M1 and the remaining flavor/guest matrix have not been rerun for this
development slice. Full P3-08 acceptance remains required.

All ten serial prerequisite checks pass in
`build/m2/P2-01/prerequisites-1793332023352/prerequisites.json`, including all 29
active host harness tests and Phase 0 clean-build/source-path checks. The first
attempt, `prerequisites-1793331577726`, failed its host child-process fixture at
the unchanged 3000-ms deadline, before guest execution. Its full failure is in
`work/acceptance/1793331815510/host-tests.log`. The second attempt uses the
established `RUST_TEST_THREADS=1` / `RUST_MIN_STACK=16777216` settings from P3-07.
Both attempts remain local; the later pass does not prove the timeout's cause.
These are development checks, not a retained story acceptance bundle.

## Proposed execution contract

Use the already selected 64-bit HPET main counter, with its validated period,
for elapsed monotonic time. Delivered LAPIC ticks retain their existing boot
uptime meaning. Reading elapsed time must not depend on interrupt delivery.
Convert with u128 arithmetic, reject backward/ambiguous counter observations,
and fail closed on nanosecond exhaustion. Report ceiling period resolution;
deadline comparisons use absolute elapsed nanoseconds, never wall-clock values.
Late delivery checks the counter again before returning a timeout or dispatching
a waiter. It cannot move an absolute deadline into the future.

One CPU and one thread per process remain the admitted subset. Readiness checks,
wait registration, cancellation and completion run with interrupts masked.
No user pointer, pin, registry borrow or kernel stack continuation may survive
parking. Resume from a saved validated user frame, using the existing FP context
mechanism. Finish the syscall's shared return validation before capture.

Level readiness is non-consuming: a wake invites a retry and reserves neither a
message nor queue space. Channel send/receive retain P3-07's atomic commit points.
A timeout or confirmed cancellation before commit transfers no bytes/authority;
an accepted message cannot be recalled by cancelling a later wait. Service effects
are outside kernel message acceptance. Manual-reset events wake every registered
waiter; reset affects future observations. A periodic timer coalesces missed
expirations and advances from its original schedule, with checked arithmetic.

Resolve each serialized observation in this order: terminated owner, invalid or
closed watched handle, requested cancellation, expired deadline, ready item in
input order, registration. A recorded completion is immutable. A later cancel
request cannot replace success. Process termination removes registrations before
object/backing destruction and never resumes the dead owner. Wait handles must
retain generation identity; closing and reusing a slot cannot retarget a wait.

Bound wait records by scheduler capacity and items per record by a fixed small
limit; reject empty, oversized and duplicate-item sets before registration.
Use deterministic FIFO runnable admission for simultaneous wakes; level events
may wake multiple owners. Application wait cycles are resolved by explicit
deadlines/cancellation/termination, without automatic general deadlock detection.
An empty runnable queue uses the atomic `sti; hlt; cli` sequence only after all
mutable borrows and pins have ended. Check deadlines/readiness after every wake.

## Required validation before acceptance

Run the ten prerequisites serially in the existing local signer/xorriso context.
Keep their complete logs under ignored build storage. Test already-ready and
registration boundaries, invalid/reused handles, waiter/item exhaustion,
multiple waiters, cancellation acknowledgement and immutable completion, timer
rearm/cancel/coalescing/overflow and delayed delivery. Then exercise actual CPL3
blocking send/receive, peer death, kill during wait, two/three-party cycles and
healthy control-task progress, with exact transfer/wait/resource inventories.

Build ordinary, boundary, scheduler, capability and channel flavors separately
and run their current guest regressions. Run complete M1 acceptance and retain
failed attempts separately with unchanged deadlines. Measure HPET observer cost,
idle halt/wake and wake latency entirely within the guest. No cross-clock
subtraction, hard real-time, desktop/input/frame or whole-guest idle claim follows.
Promote a compact reviewed evidence bundle only after independent raw/artifact
checks and repository hygiene/governance/docs audits pass. The focused guest
command above exists and passes development review; full story acceptance now passes local self-review. See the
[retained evidence](../../../evidence/phase-3/P3-08/2026-10-30/README.md).
