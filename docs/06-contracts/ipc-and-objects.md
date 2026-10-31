# Draft contract: kernel objects and IPC

Status: design draft, version 0.1; freeze only after M3 interoperability tests.
No syscall numbers or compiler-dependent layouts are frozen by this document.

P3-01's [implementation-level runtime draft](runtime-v0.md) supplies startup,
register/pointer/error rules, explicit message bytes, allocation/lifetime policy,
trusted provenance and per-story test gates. Its [single contract source](../../contracts/runtime-v0.json)
generates draft SDK offsets/names only; guest enforcement and independent ABI
interoperability remain P3-02–P3-12 work.

## Kernel API families

| Family | Initial operations | Required semantics |
| --- | --- | --- |
| Discovery | `abi_query`, `object_info` | Supported ABI/features and object type/rights; no private kernel pointer exposure |
| Handles | `close`, `duplicate`, `transfer` | Attenuation; explicit in-transit lifetime; no forged/stale access |
| Processes/threads | `create`, `start`, `terminate`, `join` | Quotas, parent supervision, entry validation and exit status |
| Memory | `create`, `map`, `unmap`, `protect` | Overflow/alignment checks; W^X and cross-process authority |
| Channels | `create_pair`, `send`, `receive` | Bounded atomic bytes+handle transfer; queue backpressure |
| Events/waiting | `wait_many`, `signal`, `cancel` | Monotonic deadlines; closed-peer/cancel outcome; bounded wait set |
| Timers | `create`, `arm`, `cancel` | Monotonic time; defined overrun/coalescing behavior |
| Devices | `map_resource`, `bind_irq`, DMA mediation | Granted ranges only; lifetime and reset ordering |
| Diagnostics | `log`, restricted debug query | Bounded data, rate limits, permissions |

## Caller identity and authority

M3 must specify trusted caller provenance and principal/session binding alongside
the handle ABI; see [identity and authorization](../02-architecture/identity-and-authorization.md).
Payload identity/owner/role fields are untrusted. Specify sender attribution when
channels are transferred and prevent stale process/session identifiers from being
rebound to a different caller. Owner metadata is not a substitute for a rights-
bearing handle or service-side scope check. No identity encoding is frozen here.

## Initial kernel message limits

Planned M3 ceiling: 4 KiB inline payload, up to 8 attached handles, 64 queued
messages/channel and a byte quota; all subject to M3 measurement. P3-07's admitted
experimental subset is specified in the [channel runbook](../05-delivery/phase-3/channels-runbook.md):
eight pairs, eight messages/16384 bytes/sixteen attached references per direction.
The [local acceptance review](../../evidence/phase-3/P3-07/2026-10-29/README.md) passes this subset. Bulk objects are referenced through
shared memory with bounds/access rights. Validate the entire message before accepting
any attached authority. On failure sender retains handles unless the ABI explicitly
defines a different complete outcome.

Message metadata uses fixed-width integer encodings, explicit lengths and a request
ID. The service envelope contains protocol ID, major/minor, operation ID, required
features, route epoch and request ID. Little-endian is the initial wire order;
encoding helpers must work without unsafe casts to native structs. Reserved bits
must be zero on send and handled according to the version policy on receive.

## Errors and waits

Minimum typed error set: `BadHandle`, `WrongType`, `AccessDenied`, `InvalidArgument`,
`BufferTooSmall`, `WouldBlock`, `TimedOut`, `Cancelled`, `PeerClosed`, `OutOfMemory`,
`QuotaExceeded`, `Unsupported`, `StaleEpoch`, `IncompatibleProtocol`, `InternalFailure`.
Do not convert all errors to a boolean. Expose exact byte/handle needs where safe.

Wait registration and readiness observation need an atomicity model to prevent lost
wakeups. A cancellation request is separate from confirmed operation cancellation.
Document whether a timeout consumed a queued message. Avoid ambiguous partial results.

## Shared memory

Grant an immutable snapshot or a specifically owned mutable buffer. An immutable
description contains object ID, offset, length, format/version and expected rights.
Every range is checked with overflow-safe arithmetic. For a mutable ring, freeze
producer/consumer indices, memory barriers, wrap rules and destruction behavior in
a separate implementation-level specification before use.

## Handle and epoch lifecycle

Use slot generation checks to reduce stale-handle reuse and specify generation-wrap
behavior before exhaustion becomes possible. A brokered service handle has a logical
service ID and epoch, not a raw process endpoint persisted across builds. Revoking a
route does not revoke all direct channels automatically; clients/proxies and resource
owners enforce epochs. Resource leases are fenced before a new writer is granted.

## Acceptance fixtures

Independent encoder/decoder implementations round-trip every message type and reject
overflow, truncation, unsupported mandatory fields and invalid rights. Test transfer
with full queue/peer death, duplicate close, mapping races, cancelled waits and stale
epochs under replacement. Publish real ABI numbers only after these pass.

P3-06's [bounded implementation](../05-delivery/phase-3/handles-memory-runbook.md)
adds process-local rights checks and retained memory lifetimes. Its draft scalar
object query and 32-byte little-endian map request are recorded in runtime-v0;
no Rust layout is serialized. Closing the last handle preserves mapped backing
until the last mapping is gone, including after creator death. This implements
no universal revocation, public transfer or device/IRQ/DMA grant.

## P3-08 wait/deadline subset

P3-08 passes local self-review for bounded waits, elapsed HPET deadlines, events/timers, cancellation and blocking channel helpers. The 18-task/six-wave guest proves registration-boundary notifications, immutable completion, pressured transfer ownership, bounded request cycles, parked-owner termination and exact settlements. All 64 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel regressions pass. General supervision, policy/session liveness, independent SDK freeze and guest UI remain open; P3-09 is next. See the [acceptance evidence](../../evidence/phase-3/P3-08/2026-10-30/README.md). This adds R10/R14/R18 and T03/T06, the timer subset of T04 and runtime T09 groundwork; it accepts no desktop latency or whole-guest idle-memory target.

## P3-09 lifecycle and quota interaction

The [supervision profile](../05-delivery/phase-3/supervision-runbook.md) uses process-local typed handles for existing objects and separate monotonic tokens for authenticated parent/supervisor control. Child bootstrap installs only its own process/thread handles; nonempty inherited-handle lists reject. Prepaid subtree credits remain spent after close, transfer, exit and join, so retained objects cannot erase an ancestor charge. Reaping removes waits before registry mappings, queued references and owned backing. Retained transfers keep their existing reference lifetime; an instance token never retargets a recycled process slot. Policy/session liveness remains P3-10.


P3-10's experimental [policy profile](../05-delivery/phase-3/policy-runbook.md)
adds a user broker/provider with exact process-bound scoped assignments, finite
session leases, current binding queries, mandatory use-time epoch/revision/deadline
checks and exact single-use forwarding tickets. The broker owns policy; kernel
sender attribution alone remains historical. Policy/session removal and broker
restart do not universally revoke existing direct handles or mappings.

## P3-11 synthetic route profile

The [draft route profile](../../contracts/routes-v0.json) and
[runbook](../05-delivery/phase-3/routes-runbook.md) separate writer fencing,
instance grant and route publication. The broker-owned counter checks exact
forwarded caller/instance/epoch and current policy at every commit. Pending work,
result retention, high-water marks and lost-reply outcomes are bounded explicitly.
This bounded profile passes local self-review and does not freeze an independent SDK.
