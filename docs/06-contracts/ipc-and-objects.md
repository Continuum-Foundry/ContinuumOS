# Draft contract: kernel objects and IPC

Status: design draft, version 0.1; freeze only after M3 interoperability tests.
No syscall numbers or compiler-dependent layouts are frozen by this document.

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

Proposed: 4 KiB inline payload, up to 8 attached handles, 64 queued messages/channel
and a byte quota; all subject to M3 measurement. Bulk objects are referenced through
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
