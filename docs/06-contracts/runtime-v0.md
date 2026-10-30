# Experimental M3 runtime contract

Status: implementation draft, revision 0, P3-01 through P3-06. The internal memory
backend and staged loader exist; the bounded P3-04 execution experiment passes local self-review.
No executing user ABI is frozen.
[runtime-v0.json](../../contracts/runtime-v0.json)
is the single source for generated SDK field offsets and operation/error names.
`python tools/m3_runtime.py generate <local-output.rs>` generates compiler-local
names, not syscall numbers or serializable enum layouts. The P3-04 xtask user-image builder generates experimental
dispatch/error constants and the process_exit trampoline from that same source; P3-12 freezes only independently exercised IDs.
The [profile](../../config/m3-profiles.json) fixes experiment limits. Revisions to
either input require new hashes and review before measurements.

## Target, startup and privilege boundary

P0's actual `userland-link` uses `x86_64-unknown-none`, small code model, static
relocation, ELF64 little-endian ET_EXEC at `0x400000`, SysV C entry, no red zone,
no std/heap/syscalls. The target's default kernel code model is overridden for
userspace. Its integer transform entry takes a u64 and returns a u64; it has never
been executed. P2's selected UI link probe also starts at `0x400000`, with separate
RX/R/RW loads, core/alloc/libm and a non-reclaiming 64 MiB arena. Neither artifact
is a P3 user executable: P3-03 supplies a new startup stub and P3-05 audits actual
emitted instructions before any FP/SIMD-dependent code runs. Keep both probes intact.

P3-03 accepts trusted-package static ET_EXEC, x86-64, no dynamic loader, TLS,
relocations or overlapping page-rounded loads. Require checked file/memory extents,
zero-fill BSS, entry in file-backed RX bytes, W^X/NX and user/supervisor permission
at every page-table level. Initial user range is `[0x10000, 0x80000000)`; null,
kernel/direct mappings and arithmetic wrap are rejected. Page size is 4096 bytes.
Image, BSS, heap, stacks and shared mappings all charge process commitment.

Startup uses a 64 KiB zero-filled user stack with an unmapped page on each side.
At the entry function RSP mod 16 is 8: a synthetic return address leads to an RX
user trampoline that invokes `process_exit`. RDI points to an immutable 64-byte
startup block; all other GPRs are zero, DF is clear, user IF is set, IOPL is zero,
FS/GS bases are zero and no kernel pointer is exposed. The loader establishes
the call-shaped stack; it does not CALL across privilege levels. P3-04 enters via
IRETQ with kernel-controlled selectors, RFLAGS and canonical RIP/RSP. No SYSRET
fast path is planned at this gate. A forged return trampoline or invalid stack
fault terminates the process, without a kernel unwind.

P3-03 implements only staged memory and trusted bindings; see the
[loader runbook](../05-delivery/phase-3/static-loader-runbook.md). Its staged
return page remains UD2, preserving the P3-03 inspection contract.
That staging test remains non-runnable. P3-04 explicitly substitutes the generated
process_exit trampoline for executing launches; see the [boundary runbook](../05-delivery/phase-3/syscall-boundary-runbook.md). Startup R/NX page is `0x7ffd0000`, return RX page is
`0x7ffd1000`, and initial RSP is `0x7fff0000 - 8`. Features, bootstrap handle and
clock are zero; no live channel or grant is encoded. Trusted descriptors are exact
ELF SHA-256/size pairs compiled into the kernel, never accepted from module data.
The coherent optical image/loader is the trust root; module memory stays reserved.
Image bytes are eagerly copied; failure reclaims all staged process state before
publishing any binding. The fixed boot launcher owns synthetic principals/sessions
and internal grant ceilings. Child launch must authenticate supervisor delegation,
derive principal/session/class from a live generation and narrow grant/resource
ceilings in P3-09/10. Payload identity/role/handle bytes never mint authority.

Startup bytes, in JSON field order, are explicitly encoded little-endian:

| Offset | Field | Bytes | Meaning |
| ---: | --- | ---: | --- |
| 0 | magic | 8 | Literal bytes `CNTM3RT0`, not an integer cast |
| 8 | revision | 4 | 0; mismatch rejects launch |
| 12 | length | 4 | 64; shorter/longer unsupported at revision 0 |
| 16 | features | 8 | Supported experimental features, no authority |
| 24 | bootstrap_handle | 8 | Process-local channel handle granted at launch |
| 32 | page_bytes | 4 | 4096 |
| 36 | reserved | 4 | Zero |
| 40 | clock_resolution_ns | 8 | Zero means elapsed clock unavailable until P3-08 |
| 48 | reserved_tail | 16 | All zero |

The bootstrap channel supplies explicit initial handles and service discovery.
There is no ambient global namespace, host syscall, environment, argv, filesystem,
randomness, credentials or inherited host handle. Principal/session identifiers
can be exposed as diagnostic data through discovery; they cannot select authority.

## Syscalls and pointer copying

P3-04 owns the first `SYSCALL` entry: RAX operation, arguments in RDI, RSI, RDX,
R10, R8, R9. Return RAX is zero for success or a positive typed experimental error
ID; RDX is a value/required capacity only where the operation declares one, zero
on other errors. Unknown operation/features return `Unsupported`. Error names are
in JSON; P3-04 experimental positive error IDs follow that explicit table. No negative pointer/error overloading.
RCX/R11, RAX/RDX and memory are clobbered; remaining GPRs preserved. User stubs
declare those clobbers explicitly. No SIMD result registers or native Rust values
cross this boundary. P3-05 preserves FP state independently of the syscall ABI.

Entry must save user RSP/GPRs to supervisor-only per-thread storage before enabling
interrupts, select the owned kernel stack and never trust incoming RCX/R11 as a
kernel address/flags. IA32_FMASK clears IF/DF/TF/AC on entry. P3-04 explicitly tests
entry from hostile stack, noncanonical RIP/RSP, privileged flag bits and nested
interrupts. Before IRETQ, validate user RIP/RSP and construct allowed RFLAGS;
malformed state kills the caller. Fatal kernel entry-stack faults use M1's IST.
No rendering, allocation or blocking IPC occurs on entry/IRQ stacks.

Each pointer is `(u64 address, u64 length)` with operation-specific maximum, checked
add, user-range, page presence and permissions. Zero-length spans ignore address
and touch nothing. Reject output aliases with request/descriptor input spans;
copy request metadata once into bounded kernel storage before dispatch. Single
CPU, no shared page-table mutation during a copy: disable task preemption for the
bounded copy and hold the address-space mapping guard; IRQs cannot unmap. P3-04
adds a copy-fault recovery site so an invalid user page becomes `InvalidArgument`,
not kernel fatal. No request stores a user pointer across a wait. P3-02/P3-04 supply copy pins with effective hardware permissions and IRQ exclusion.
P3-06 adds rights-bearing ownership for retained memory objects; copy all small
control bytes, never serialize a native struct by casting it.

`abi_query` is the first experimental operation, callable without a bootstrap
handle. It writes a bounded explicit record: revision, features, page size, exact
message/handle/wait limits and elapsed-clock availability. Too-small output returns
`BufferTooSmall` with required bytes and writes nothing. Different major revision
rejects; unknown mandatory features reject; unknown optional features are ignored
only when explicitly marked optional. Reserved fields must be zero at revision 0.
All future records have explicit length/revision checks before field access.

The register API uses scalar arguments or pointers to explicit operation records.
All pointer/length pairs are bounded before copy, no output is modified before
admission completes, and unsupported flags return `InvalidArgument`. Concrete
record field definitions join JSON when their owning story implements them.

| Operation family | Argument/result plan and lifetime |
| --- | --- |
| `abi_query` | `(out_ptr, capacity)`; success returns bytes written, too-small returns required bytes |
| `object_info` | P3-06: `(handle)` returns `(kind << 32) | rights`; no object identity or pointers |
| `handle_close`, `handle_duplicate` | `(handle)` / `(handle, rights_subset)`; duplicate returns new handle, close is not universal revocation |
| `process_create`, `process_start` | `(launch_record_ptr, bytes)` / `(process_handle)`; immutable trusted image/grants, suspended until start; no caller-selected principal |
| `process_exit`, `process_terminate`, `process_join` | `(reason)` / `(process_handle, reason)` / `(process_handle)`; exit never returns, nonblocking join returns retained exit reason or `WouldBlock` |
| `thread_create`, `thread_start`, `thread_exit`, `thread_join`, `thread_yield` | `(record_ptr, bytes)` / `(thread)` / `(reason)` / `(thread)` / `()`; nonblocking join, bounded same-process guarded stack/entry/context, no authority amplification or caller-selected scheduling privilege |
| `memory_create` | P3-06: `(self_process, bytes, backing_rights)` returns zero-filled granted memory |
| `memory_map`, `memory_unmap`, `memory_protect` | P3-06 exact scalar/request shapes are in `objects_boundary.arguments`; retained mapping ceilings, checked ranges/aliases/pins and transactional rollback |
| `channel_create` | `(out_ptr, capacity)` writes two handles atomically; fixed profile queue limits |
| `channel_send`, `channel_receive` | `(channel, record_ptr, bytes)`; nonblocking, records describe spans and transfer/result arrays, all-or-nothing consumption |
| `wait_many` | `(items_ptr, item_count, deadline_ns, cancel_handle)`; copy bounded handle+interest records before sleeping, return RDX `(index << 32) \| ready_bits`, no output pointer retained and no message consumed |
| `event_create`, `event_signal` | `()` / `(handle, set_bits, clear_bits)`; event is level-triggered, overlapping set/clear masks reject |
| `wait_token_create`, `wait_cancel` | `()` / `(cancel_handle)`; monotonic cancellation token, signal right required, stale handle denied, no reset |
| `timer_create`, `timer_arm`, `timer_read`, `timer_cancel` | `()` / `(timer, deadline_ns, period_ns)` / `(timer)` / `(timer)`; read atomically consumes pending count/readiness or returns `WouldBlock`, cancel clears future arms and pending readiness |
| `clock_read` | `()` returns elapsed ns only after P3-08, before then `Unsupported` |
| `diagnostic_log` | `(bytes_ptr, byte_count)`; 256-byte maximum, per-process rate/quota, no privileged queries |

Handles require type-appropriate rights for every operation; process launch/terminate
and address-space mutation additionally require explicit supervisor grants. Closing
the last handle destroys an unstarted process; a started process retains scheduler/
supervisor refs until termination/reaping. Join can be repeated by authorized
observers until their process handles close; it does not free another observer's data.
Kernel-backed wait registrations are not serialized futures or Rust stack frames.

The SDK implements deadline-bounded channel/join calls by polling the nonblocking
operation, `wait_many` on level readiness, then retrying with the same absolute
deadline and cancellation handle. Check the deadline at every retry; readiness is
advisory, so another thread consuming a message causes another bounded retry.
Zero cancel handle means no token; otherwise the caller must hold wait rights.
Cancellation tokens charge ordinary handle/object quotas and can be signaled by
an explicitly authorized peer thread/process. An already cancelled token rejects
future blocking registrations. No blocked operation retains a user buffer pointer.

## Allocation, ownership and teardown

P3-02 implements this internal subset in the [owned-memory runbook](../05-delivery/phase-3/owned-memory-runbook.md)
and [ADR-0018](../04-decisions/0018-bounded-owned-memory.md). Page tables use two
fixed 64-page banks per space, charged in full with metadata and guarded kernel
stack storage. Data backing uses reclaiming contiguous page runs; fragmentation
can return OOM. A mutation rebuilds the inactive bank before atomic CR3 publication.
Executable backing has only RO/NX supervisor data aliases; staging writes require
zero mapping references. Legacy internal shared references return Busy on owner death; P3-06 object-held
allocations instead detach and remain globally charged until the last reference. Copy pins
block all mutations of the pinned space; P3-04 adds preemption/copy-fault handling.
No public ABI IDs, user allocator or ring-3 execution follow from this subset.

P3-02 uses preallocated bounded kernel tables and a reclaiming 4 KiB page allocator;
no general kernel Rust global heap is required. Kernel dynamic page storage has a
16 MiB ceiling, separate from existing reserved kernel/boot metadata and accounted
in full. Reserve metadata, frame ownership and rollback capacity before publishing
any object. Table exhaustion returns `QuotaExceeded`; physical exhaustion within
quota returns `OutOfMemory`. No allocation while holding an IRQ/scheduler lock.
No retry loop, panic or admission based only on 2 GiB VM capacity.

Objects own references, not process IDs. Process owns its page-table root, image,
user allocation/mappings, handle table, threads and supervisor relationship.
Thread owns its guarded kernel stack, user stack commitment, wait state, saved
GPRs and aligned FP context. Memory objects own zeroed frames and mapping refs;
channel queues own copied bytes and in-transit object refs. Shared frames are
reported once physically, with separate per-process mapping charges. Global quotas
cover even objects whose originating process has died. Frames are zeroed before
cross-owner reuse. Process death removes runnable/waiting threads, cancels waits,
resolves transfers, removes mappings and drops references before reclaiming tables
and stacks. The current executing kernel stack is reclaimed only after switching
to a surviving scheduler context. P3-09/P3-13 test inventory restoration.

P3-02's allocation backend returns zeroed page runs, rounded up from a nonzero
size; accept power-of-two alignment from 1 through 2 MiB, check size rounding and
alignment overflow, and charge the complete run. Free requires the exact live
allocation token/owner, not a caller-supplied size. Realloc allocates/admit-first,
copies `min(old,new)` bytes, zeroes the grown tail and frees old storage only on
success; failure preserves the old allocation. Zero-size allocation/realloc-to-zero
has the explicit result `None`/free, never a dereferenceable zero-byte allocation.
The eventual userspace small-allocation free list may split/coalesce inside its
granted heap, with metadata charged there; corrupted user metadata cannot select
kernel frames. No unsupported alignment silently falls back.

Userspace eventually uses a reclaiming per-process allocator on granted RW/NX
memory; a 32 MiB heap is contained in the 64 MiB process commitment ceiling.
Large UI/font/render staging needs a later measured profile with explicit headroom;
the link-probe arena is never installed as the guest allocator. Fallible service
admission precedes infallible library allocations. Rust allocation error/panic
aborts only that process with a bounded exit reason; no unwinding across the ABI.
Kernel corruption/panic remains fatal with M1 diagnostics. The supervisor and
allocator must not depend on allocating in their OOM/reporting path.

## Handles, messages and authority

Handles are opaque process-local 64-bit values, low 32 bits slot, high 32 generation;
zero is invalid. P3-06 consumes a boot-wide monotonic nonzero u32 nonce on every
installation; after u32::MAX, new installations fail until reboot. Object slots
retire on u64 generation exhaustion. Apply the same no-reuse
rule to process/session/object identities and policy/route counters; exhaustion
fails closed rather than wrapping authority. `close` drops exactly one reference;
duplicate requires the duplicate right and a subset of source rights. Wrong type,
stale handle and denied rights retain distinct errors. P3-06 defines generated
right bits in JSON when implemented; device/IRQ/DMA remain `Unsupported`.

P3-07 copies up to 4096 payload bytes and eight attached handles atomically. Each
transfer descriptor encodes source handle u64, requested-rights u64, mode u32
(`move` or `duplicate`, IDs added with implementation), reserved-zero u32. Require
transfer rights, and duplicate rights for duplication; reject repeated source slots.
Check rights, destination admission, queue slots/bytes and refs before enqueue;
failure retains all source authority and consumes no message. Move invalidates
sender handles only at enqueue commit. Receive validates output capacity before
dequeue, installs all receiver handles atomically, and otherwise consumes nothing.
Peer death releases queued refs; no authority disappears into a partial transfer.
P3-08 adds bounded waits on full queues; no unbounded allocation on backpressure.

The 48-byte service envelope follows JSON field order: protocol u32, major/minor
u16 each, operation u32, required_features u32, payload_bytes u32, handle_count u16,
reserved-zero u16, request_id u64, route_epoch u64, policy_epoch u64. Declared payload
length excludes the header; the total inline control payload including header must
fit 4096 bytes. Truncation, unknown required fields/features, trailing bytes, wrong
reserved bits and count/length overflow reject. Independent codec fixtures and
actual guest interoperability are P3-07/P3-12 obligations, not generated-offset tests.

Kernel receive metadata is separate from untrusted service bytes: sender process
generation, principal, app/service identity and authority-session generation are
captured at send acceptance, even after channel transfer. Claimed owner/role/session
does not grant access. Broker checks resource scope, delegation ceiling, current
session/policy epoch and expiry at preview and execution; same-user app B needs its
own grant. Replacement gets a new binding and fresh grants. No checkpoint restores
credentials or privileged authority. P3-10 exercises users A/B, two apps of A,
and a service principal; product accounts/login remain M7.
Broker restart fails closed until trusted policy/session context is restored;
cached role strings, owner metadata and earlier successful requests cannot reopen
authority. Development-update scope never implies installer/account administration.

P3-11 proxy requests carry request/route/policy epochs. A broker-owned writer lease
is fenced before granting the next writer. Stale queued work/replies get explicit
`StaleEpoch`; committed effects are not undone by cancellation. Role/session
invalidation stops new grants and brokered operations at their mandatory check;
direct duplicate handles/shared mappings survive until explicit teardown. No
universal revocation, exactly-once delivery or live state-transfer claim.

## Scheduling, locks, CPU state and clocks

P3-05's [experimental mechanism](../05-delivery/phase-3/scheduler-runbook.md) uses one CPU, one equal-priority FIFO round-robin queue, 5 ms nominal
quantum, maximum 32 live threads. New/woken threads join the tail; no priority
inheritance or ambient service boost. With N runnable threads and no declared
critical section, each receives execution within N+1 dispatch opportunities.
The acceptance ceiling is 64 quanta including interrupt masking; retain actual
elapsed lateness and longest masked interval. This is a test bound, not a hard
real-time promise. Uncooperative integer spinners must be preempted.

Single-CPU structural locks are preemption guards with balanced nesting, IRQ-safe
guards additionally save/restore IF. Lock order: process registry, address space,
handle/object table, channel/wait object, scheduler run queue. Never acquire earlier
locks while holding later ones. Syscall data copies finish before queue locks;
never sleep/allocate/call userspace/log under structural locks. IRQ touches only
bounded pending bits/tick data and the final scheduler lock; deferred work acquires
other locks later. No spin waiting for a task preempted on the same CPU. Wait
registration and condition recheck are atomic under the object lock, enqueue before
unlock; waiter completion transitions once under the same lock. P3-08 races wake,
cancel, expiry, close and process death against that one transition.
Each registration pins the referenced object with its checked rights. Closing
one source handle does not revoke that pin; explicit cancellation or owning-process
death removes it. Peer closure is a distinct readiness/error condition. Do not
bind a waiting registration to a reused handle slot or session identity.

M1 kernel integer-only FP exclusion remains unchanged. User baseline for P3-05 is
x87/SSE/SSE2, eager FXSAVE/FXRSTOR with 16-byte-aligned 512-byte per-thread state,
fresh x87 control word `0x037f`, MXCSR `0x1f80`, cleared data/XMM state. Require
CPUID FXSR/SSE/SSE2 before launch and correctly configure CR0/CR4. AVX/AVX2/AVX-512,
XSAVE/OSXSAVE and lazy FP switching are unavailable. Audit compiler/library code
and CPU dispatch against actual CPUID and target flags; P2 UI probes are not
admitted before that audit. P3-05 tests register patterns, rounding/masks, forced
preemption and cross-process leakage. FP exceptions terminate the user process;
kernel FP use remains an audited failure. No UI code runs in IRQ context.

P3-08's proposed elapsed source is the existing explicitly mapped 64-bit HPET main
counter with validated femtosecond period. Delivered LAPIC ticks stay diagnostic
uptime only. Reject a profile with missing HPET, invalid period or unsupported
32-bit counter; no silent tick/TSC fallback. Convert with checked u128 intermediate,
round deadline ticks upward, and clamp samples monotonically. Counter wrap/backward
or nanosecond u64 exhaustion fails the clock and rejects new timed waits; cancel
existing timed waits with a distinct clock-failure diagnostic (`InternalFailure`).
No wall time or host/guest subtraction. P3-08 must test long IRQ-masked intervals,
conversion boundaries, rollover injection and timer delivery lateness before
claiming elapsed deadlines. The clock choice remains experimentally unaccepted.

Use absolute unsigned ns deadlines in this boot epoch, `u64::MAX` for infinite,
zero for poll. Readiness observed at registration wins; once blocked, first committed
ready/cancel/timeout transition wins. Timeout consumes no message. Cancellation
acknowledges a wait result, not rollback of a service effect. Periodic timer wakeups
coalesce; return an overrun count from elapsed counter intervals, advance from the
original schedule and reject overflow. Record lateness; never count delivered IRQs
as every elapsed period. P3-08 owns this proof and its serial diagnostic extension.

## Experiment and handoff gates

All capacities/counts/clocks are in the locked profile, not implicit success claims.
Stress: 1000 lifecycle/fault/transfer cycles; IPC 1000 warm-up + 10000 round trips
per each of five fresh repetitions; settle for 64 quanta. Report live kernel objects,
committed/shared/free pages, queue/handle/wait/timer storage, stacks and FP state,
before/peak/after; require zero unexplained objects/frames after quiescence. Any
retained cache must get a separately reviewed budget before use. Reclaiming tables
remain allocated capacity and are charged; zero leak does not mean zero footprint.
Use paired tracing on/off, raw nearest-rank distributions and timeout/loss counts.
Host `perf_counter_ns`/QPC is lifecycle time; guest HPET time is accepted only at
P3-08. Keep instrumentation overhead and VM/host memory separate. The 256 MiB
cold-idle desktop target remains, with no P3 desktop acceptance.

| Obligation | Owner / required negative evidence |
| --- | --- |
| Owned pages, quota, user/supervisor/W^X, rollback | P3-02, overflow/exhaustion/reuse |
| Static loader, guarded startup, trusted bindings | P3-03, malformed ELF/forged launch |
| CPL3 syscall/copy/fault containment | P3-04, hostile pointers/flags/stack |
| Preemption, all GPR and FP isolation | P3-05, spinner/fairness/context leakage |
| Rights, mappings, anti-stale lifetimes | P3-06, same-user and cross-owner denial |
| Copied bytes/atomic handles/provenance | P3-07, full queue/death/partial admission |
| HPET, waits, deadlines/cancel/overrun | P3-08, masked IRQ/lost wake/wrap |
| Supervision and resource inventories | P3-09, OOM/process death/reap |
| Scoped policy/expiry/revocation | P3-10, spoof/epoch/broker restart |
| Proxy writer fence | P3-11, stale writers/in-flight outcomes |
| Independent SDK/codec and ABI freeze | P3-12, independent compiler/client fixtures |
| Sustained adversarial accounting | P3-13; final-source aggregate P3-14 |
| Font/resources/input/presentation/event loop | M4, every absent UI hook remains open |

The [hook inventory](../../experiments/ui-link/hooks.json) remains unchanged and
unimplemented in kernel. Allocation, entry/memory, FP/math and process failure map
to P3-02–09; custom single-thread ownership maps to P3-05/09. Font/text resources,
typed input, semantics, surfaces, clipboard and native services map to M4 or later.
Custom event time/pacing needs P3-08 then M4. Frozen Slint comparison has the same
allocation/FP foundations plus one owning execution thread, no IRQ/concurrent
entry, install-once context and callback retirement; its clocks/window/proxy/native
services and distribution terms remain frozen gaps, not M3 port commitments.

## P3-04 experimental operations

The [syscall boundary runbook](../05-delivery/phase-3/syscall-boundary-runbook.md) specifies IDs 0–5, per-operation buffer/value/error semantics, the private reclaiming page-run allocator, return-state validation, partial-copy behavior and exact assembly/unsafe lifetime review. R9 is revision zero and R8 reserved features zero. The 32-byte query carries diagnostic trusted IDs only; payload claims cannot select a process or authority. Staged UD2 and explicitly executable process_exit launch paths remain separate. Scheduling/FP state, public memory rights, general supervision and independent ABI freeze are later gates.

The P3-05 execution subset admits one owning thread per process (16 process
slots) with a 32-entry queue policy. Public/shared-process threads, priority
changes and blocking kernel continuations remain unsupported. Syscalls complete
before switching; IRQs in the kernel mark pending only. Scheduler allocation
calls admit at most 64 KiB. The P3-04 profile separately retains its TS guard.
See ADR-0021 and the scheduler runbook for eager-state, feature and observer policy.

## P3-06 experimental object/memory operations

The [handles/memory runbook](../05-delivery/phase-3/handles-memory-runbook.md)
and [ADR-0022](../04-decisions/0022-retained-memory-objects-and-handle-ceilings.md)
define operation IDs 6�12, type-specific rights, immutable backing/mapping ceilings,
independent handle/map refs, creator-death detach and exact exhaustion policy.
The contract JSON includes explicit 32-byte map fields and access/rights IDs.
The new profile requires a self-thread CONTROL handle for yield and a self-process
CONTROL handle as process_exit's second argument. Earlier P3-04/05 execution
fixtures retain their gated regression ABI only. In the rights-bearing profile,
provisional memory_allocate/free return Unsupported. Admission installs initial
self handles and explicitly grants immutable snapshots; no user-chosen identity
can install authority. New capacities are the locked objects_execution_subset,
within the larger planned M3 limits. Public device/IRQ/DMA and general supervisor
operations remain unsupported. ABI freeze and public transfer remain later gates.


## P3-07 implementation status

The [channel runbook](../05-delivery/phase-3/channels-runbook.md) defines the
experimental nonblocking profile; its [local acceptance review](../../evidence/phase-3/P3-07/2026-10-29/README.md) passes the declared subset. Enqueue reserves
queue-owned references. Actual destination handle-table and nonce admission
occurs on receive; a full destination table leaves the entire accepted message
queued. An accepted move is already committed and cannot be reclaimed by its
sender. Failed receive copies may modify output bytes but install no authority
and consume no message. Outputs are usable only on success. Bounded tracing
collects unreachable endpoint/queue cycles. Historical sender metadata never
proves current session or policy validity. The original P3-06 profile is retained.
