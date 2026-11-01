# Bounded independent SDK subset

Status: P3-12 bounded subset frozen under [local self-review](../../evidence/phase-3/P3-12/2026-11-01/README.md).
Exact contract SHA-256: `debe57866ed308f46ff2f48f5065e739c7457d0f28d0e8dccc93d42abd4c891b`. SDK package 0.1.0, protocol major 1/minor 1; syscall register
revision remains **0**. These version spaces are distinct. The machine-readable
[subset](../../contracts/sdk-v1.json), packaged identical contract, generated
constants and [byte fixtures](../../sdk/continuum-rt/fixtures.json) are synchronized
by `tools/sdk_contract.py`. This is a local experimental subset, not a native app
SDK release. All original source retains the repository's rights policy.

## Register and startup boundary

Use the target, register clobbers, IRETQ return validation and explicit 64-byte
startup encoding in [runtime-v0](runtime-v0.md). RAX is the operation, RDI/RSI/RDX/
R10 are arguments, R8/R9 must be zero. Success is RAX=0, operation result in RDX;
failure is a positive error ID with RDX=0, except discovery `BufferTooSmall`
returns required capacity 32. The stubs declare RCX/R11/RAX/RDX/memory clobbers;
the scheduler preserves the admitted x87/SSE2 state. No Rust enum, pointer-valued
error or native-layout struct crosses the boundary.

Startup is `CNTM3RT0`, u32 revision 0, u32 length 64, u64 feature bits (currently
zero), u64 bootstrap handle (zero in this launcher), u32 page bytes 4096, u32
reserved zero, u64 elapsed-clock resolution (zero here), and 16 reserved zeros.
An immutable trusted fixture bootstrap delivers self process/thread handles and
explicit endpoints separately. Fixed fixture control/roster addresses are not
SDK interfaces. The SDK accepts supplied handles; it never discovers authority
by reading a magic address or trusting a principal string. A normal Rust entry
return is converted by the entry adapter into rights-checked process exit.

## Selected operations

Pointer records below are explicitly little-endian, copied once and checked by
the kernel. `(p,n)` denotes pointer and byte capacity, not native struct layout.
Zero-length copied spans touch no address. User pointers must stay inside the
admitted user range and have the operation's permissions; input/output overlap,
overflow, invalid flags/enums, quotas and unsupported families fail explicitly.

| ID | Operation | Arguments; success RDX |
| ---: | --- | --- |
| 0 | abi_query | output pointer, capacity >=32; bytes=32: revision/page/principal/session as four u64 words |
| 1 | diagnostic_log | input pointer, length <=256, ASCII alphanumeric or hyphen; copied length |
| 4 | thread_yield | self thread control handle; zero |
| 5 | process_exit | reason, self process control handle; never returns on success |
| 6 | object_info | query handle; `(kind<<32) \| granted_rights` |
| 7 | handle_close | handle; zero |
| 8 | handle_duplicate | source, attenuated rights; fresh process-local handle |
| 9 | memory_create | self process, page-rounded bytes <=65536, backing rights; memory handle |
| 10 | memory_map | self process, memory handle, map record pointer, exact size 32; zero |
| 11 | memory_unmap | self process, exact mapped base; zero |
| 12 | memory_protect | self process, exact mapped base, access enum; zero |
| 13 | channel_create | self process, pair output pointer, exact size 16; zero |
| 14 | channel_send | endpoint, send record pointer, exact size 32; zero |
| 15 | channel_receive | endpoint, receive record pointer, exact size 48; byte count |
| 16 | channel_size | receive endpoint; `(head_attachments<<32) \| head_bytes`; empty is WouldBlock/PeerClosed |
| 17/18 | clock_now/resolution | no arguments; monotonic HPET nanoseconds/resolution |
| 19 | event_create | self process, signalled 0/1; event handle |
| 20 | event_set | event control handle, signalled 0/1; zero |
| 21 | timer_create | self process; timer handle |
| 22 | timer_arm | timer control handle, arm pointer, exact size 24; zero |
| 23 | timer_cancel | timer control handle; zero |
| 24 | wait_many | wait record pointer, exact size 32; zero-based ready item |
| 25/26 | channel_send_until/receive_until | endpoint, attempt record pointer, exact size 24; one atomic attempt, not an implicit reservation |
| 34 | context_live | exact 40-byte identity record pointer and size; 1 if exact current registry binding, otherwise 0 |

IDs 2/3 are legacy integer/scheduler fixture allocation, Unsupported in the
rights-bearing subset. IDs 27–33 retain their experimental supervision contract
and receive no independent SDK freeze here. Unknown IDs return Unsupported;
nonzero syscall revision returns IncompatibleProtocol; nonzero required register
features return Unsupported before dispatch. No public operation selects return
IP, selectors or privileged flags. Device, IRQ, DMA, accounts, credentials,
installation, files, network, wall-clock and presentation remain unimplemented.

## Encoded records and rights

Except the mixed-width attachment/header/startup, each field here is u64.

| Record | Ordered fields |
| --- | --- |
| map (32) | base, backing offset, byte length, access: Read=0/Write=1/Execute=2 |
| attachment (24) | source handle u64, rights u64, Move=0/Duplicate=1 u32, reserved zero u32 |
| send (32) | bytes pointer, byte length <=256, attachments pointer, attachment count <=4 |
| receive (48) | bytes pointer/capacity, handle-array pointer/capacity, metadata pointer/capacity >=64 |
| metadata (64) | byte count, handle count, sender process, process generation, principal, authority session, service 0/1, reserved zero |
| wait (32) | items pointer, count 1–8, absolute elapsed deadline or u64::MAX, cancel event or zero |
| wait item (32) | handle, Signalled=0/Exited=1/Readable=2/Writable=3, required bytes, attachments (last two zero except Writable) |
| timer arm (24) | absolute elapsed deadline (not u64::MAX), period or zero, reserved zero |
| channel attempt (24) | original send/receive descriptor pointer, same absolute elapsed deadline, cancel event or zero |
| context (40) | process, generation, principal, authority session, service 0/1 |
| service request payload (16) | fixture sequence matching header request ID, reserved zero |

Rights bits are QUERY=1, DUPLICATE=2, CLOSE=4, CREATE=8, MAP=16, READ=32,
WRITE=64, EXECUTE=128, CONTROL=256, TRANSFER=512, SEND=1024, RECEIVE=2048.
The object kind and backing ceiling constrain each right; a known bit does not
grant applicability to every object. Handles are process-local slot+nonce values,
zero invalid; numeric copying is not duplication. Close removes one handle only.
Mappings and duplicates retain backing until their own teardown. Only immutable
read backing can transfer in this bounded profile; RW sharing is not accepted.

Successful enqueue commits all bytes and queue-owned attenuated references. A
move invalidates the source then, even if the receiver later rejects its application
protocol. The receiver must close received authority it declines. Failed enqueue
keeps source ownership. Destination table admission occurs on receive; a failed
receive installs no handles and leaves the entire message queued, although output
bytes can have changed. Use outputs only after success. Level waits do not reserve
capacity. Public blocking helpers retry the same absolute deadline and cancellation
token; timeout/cancellation/wakeup never transfers ownership. Completion precedence
is owner termination, closed watched handle, cancellation, deadline, first ready item.

Error IDs 1–15 are BadHandle, WrongType, AccessDenied, InvalidArgument,
BufferTooSmall, WouldBlock, TimedOut, Cancelled, PeerClosed, OutOfMemory,
QuotaExceeded, Unsupported, StaleEpoch, IncompatibleProtocol, InternalFailure.
Service-policy conflict 16 is a synthetic provider result, not a syscall error.

## Versioned service bytes and authority

The 48-byte header is protocol u32, major u16, minor u16, operation u32,
required features u32, payload bytes u32, handle count u16, reserved zero u16,
request ID u64, route epoch u64, policy epoch u64. Canonical bytes never include
padding. The conformance protocol is 0x53444b, major 1; operations discovery=1,
echo=2, immutable transfer=3, with a 16-byte fixture payload. Requests carry a
fixture sequence matching the header request ID and a reserved zero word;
reserved or correlation mismatch is InvalidArgument before the operation. Echo
correlates this sequence through the reply request ID; the reply payload remains
status/minor/features, not arbitrary user data. Service minor 0
supports no feature bits; minor 1 supports bit 0. Unknown minor versions are
compatible when required features and payload schema are understood. Unknown
major/protocol or mandatory feature is IncompatibleProtocol before mutation;
bad reserved/length/operation/count fields are InvalidArgument.

The separate [160-byte synthetic route profile](../../contracts/routes-v0.json)
uses the P3-11 explicit word encoding and independent semantic matrix. Its
controller commands and in-memory resource are a conformance fixture, not general
service discovery, durable policy or state transfer. Only the current exact writer
can mutate after fence, fresh authorization and publication. Retained old endpoints
do not bypass the writer fence. Historical IPC attribution must be combined with
current liveness, finite session, scoped grant, revision, policy/route epoch and
monotonic deadline checks. A reply failure cannot undo an earlier commit.

`ProcessGeneration`, `AuthoritySession`, `PolicyEpoch` and `RouteEpoch` are distinct
public domains. `MonotonicNs` and `WallClockNs` have no implicit conversion; there
is no wall-clock source. Checkpoint bytes contain data, never live handles, grants,
credentials or development authority. Replacement requires fresh authorization.
See the [M4/M5 handoff](../05-delivery/phase-3/sdk-runbook.md#authority-and-later-handoff).
