# P4-05 copied surfaces and user-mode composition

Status: [passed local self-review](../../../evidence/phase-4/P4-05/final-1794685886700026500/README.md), 2026-11-14, for the exact copied surface/composition subset. M4 remains open.

The original Rust kernel still mediates the supervisor-only UC scanout through
P4-04 operations 0x4100–0x4104. Experimental `surface-runtime` adds a bounded
copied endpoint transport, while all surface policy, storage and software
composition run in the compositor process's private 48 MiB heap. The ordinary
build includes this service; scripted clients exist only in `surface-test`.

## Boundary and ownership

Internal operations 0x4200–0x4204 submit a client request, receive a compositor
request, send a compositor response, consume a client response and query trusted
peer liveness. These are unfrozen desktop-only operations, independent of the
frozen SDK and existing general channels. Only trusted shell/editor launch roles
have client endpoints; only the compositor role can receive/reply/query. The
kernel derives process/incarnation/principal/session from the actual current
launcher record. A writable launch word or a request body cannot choose identity.

Each client has one 4096-byte request/reply slot. A second submit denies until
consumption; receive is round-robin. Kernel copying finishes before any reader
receives bytes. Failed source/destination copies consume nothing. Owner death
clears that owner's slot. Compositor death cancels incomplete exchanges; already
copied complete replies survive until consumption or client death. Stale tickets
never address a later exchange. No attachments, writable shared mappings, raw
physical authority or retained user pointers are admitted.

The 48-byte receive prefix contains six individually encoded LE u64 values:
ticket, process, incarnation, principal, session and payload length. Response bytes
are three LE u64 values: status, value and request ID. None is a Rust layout.
Request/reply storage is fixed kernel metadata outside private heaps and is
counted separately from the existing general object queues.

## Surface encoding

Requests use the 64-byte CDU0 envelope from
[desktop-v0](../../06-contracts/desktop-v0.md), major/minor 0, no flags/features,
nonzero monotonic sequence and request ID, resource/output generation 1. Total
size is 128–4096 bytes. Every body is 64 bytes; only write appends pixel bytes.
Kinds and body fields below are experimental and endpoint-local:

| Kind | Operation | Body fields (offsets relative to byte 64) |
| --- | --- | --- |
| 1 | Create | width/height/stride/format u32 at 16/20/24/28, backing bytes u64 at 32; other bytes zero; envelope object zero |
| 2 | Acquire | All zero; returns a unique lease ID |
| 3 | Write copied chunk | lease u64 at 0, exact next byte offset u64 at 8, remaining body zero, then nonempty bytes |
| 4 | Commit | The complete 64-byte surface commit body in desktop-v0, including scene revision, extent, damage and buffer generation |
| 5 | Geometry | signed x/y/z i32 at 16/20/24, visible u32 at 28, local clip x/y/w/h u32 at 40/44/48/52; other bytes zero |
| 6 | Release/cancel | lease u64 at 0, remaining bytes zero |
| 7 | Remove | All zero |
| 8 | Request frame | All-zero body/object; synchronous copy opportunity, separate from lease release |
| 9 | Test completion | Only the compiled surface-test clients/service use this terminal barrier; ordinary service rejects it |

Statuses: 0 success, 1 malformed, 2 authority, 3 stale, 4 busy, 5 capacity.
Transport failures use the existing checked syscall AccessDenied result. Create
returns surface ID; acquire returns lease ID. Buffer generations start at 1 and
advance only on successful acquire for that surface; commit revisions start at 1
and advance exactly one. IDs never wrap/reuse. The persistent kernel display-grant incarnation seeds a disjoint 32-bit ID counter range on every compositor start; exhaustion denies. Sequence is per trusted client incarnation.

RGBA8 uses encoded-sRGB premultiplied alpha (format 1), dimensions <=1920×1080,
stride a multiple of four in [width×4,8192], backing exactly stride×height and
<=8847360 bytes. Damage and clip sums must stay within the surface. Pixel RGB
must be <= alpha. Padding is initialized, copied, and never sampled as pixels.

A lease is Writing until every sequential byte has arrived. Commit validates all
bytes and metadata before changing the active snapshot to InUse. Further writes,
duplicate commits and release of that active snapshot deny. Replacing it marks
the old lease Released; release frees it once. Cancel may free an uncommitted
lease. Removal/owner death discard all owned leases outside a reader interval. Ordinary service immediately schedules a full exposure repair when dead-client cleanup changes its inventory; the owner-death fixture uses that same helper. After the terminal test frame, fixture teardown preserves that capture while finishing the inherited runtime workload.
At most eight surfaces, three leases/surface and 24 MiB of surface backing are
admitted. Every allocation is fallible. The full output scratch allocation is
reserved before the first surface commit; its maximum is 8294400 bytes. Together
these fit the already admitted 48 MiB compositor heap. No budget is enlarged.
Only resource generation 1 is admitted; multi-generation resource publication is
still P4-15 work, within the earlier at-most-two ceiling.

## Presentation and failures

Composition clears every output pixel to RGB (12,18,24), sorts by signed z then
surface ID, clips signed placement and local integer rectangles, and blends
premultiplied source-over with rounded division by 255. The kernel converts RGB
channels to the actual scanout format. Full redraw repairs movement, hide,
removal and death exposure. Damage is validated but remains a hint; optimization
belongs to P4-17. There is no toolkit, input routing or SceneCommit requirement.

The single owning compositor thread excludes mutations throughout each frame
reader interval. Source row copy completes before row scratch reuse. Composition yields after each 16 rows (and the final row), without changing the copied row count or reader interval. An accepted
frame opportunity is distinct from a lease becoming writable/released; the active
snapshot remains InUse. Rejected create/acquire/write/commit/geometry operations
leave the last displayed frame unchanged. The mediated synchronous scanout makes
no atomic-frame, VSync, tear-free, display acknowledgement or latency claim.
A fault during row presentation may leave a partial frame. Kernel process reaping
revokes display authority and clears incomplete mailbox exchanges; serial/control
and other roles remain scheduled. No crash continuity or safe code replacement
is claimed. Later clients need an explicit fresh service binding after failure.

## Validation and evidence

Preview `tools/Test-P4-05.ps1 -Plan` before guests. The selected plan is schema 2,
22 VM launches, 20 separate metadata builds and no full milestone suites. The
one-guest development probe uses `-Probe -Plan` then `-Probe`; it omits the repeated prerequisite aggregate while the final story gate runs all ten checks fresh. The development probe is never story acceptance. Run in the existing
signer/xorriso context, serially, with the host awake. Do not change host settings.

The plan selects independent optical 720p/1080p overlap and move/hide/remove/
malformed/death/compositor-panic/stalled-presentation cases, existing display, ordinary runtime and
focused memory/boundary/scheduler/capability/channel/SDK-runtime regressions.
Both clients are separately linked images. Each optical case retains the existing
nine-owner/900-heap-cycle workload, restart counts, 30000-ms guest deadline and
320-ms guest-local scheduling bounds. A registered test hook assigns session 2002
to the editor's second incarnation to exercise real wrong-session denial; other
roles retain session 1001. A separate compositor panic and CPU-stalled presentation test mailbox cancellation
and actual kernel teardown. These are synthetic sessions, not user accounts.

The independent Python reviewer reconstructs expected pixels without importing
producer/renderer code; reads exact raw serial/QMP and request/reply identities;
checks negative transitions and settlements; compares actual QEMU PPM captures
at every pixel; and rejects removed criterion records. Host literal-byte and
adversarial broker/transport tests supplement actual guest proof. They do not
alone establish story acceptance. All generated runs/failures remain local under
`build/` and `work/`, with a compact reviewed bundle/capsule required at closeout.

Unsafe review: private frame allocation becomes a slice only after successful
bounded allocation and is freed only after synchronous readers finish. The global
heap has one owning user thread. Kernel mailbox mutation follows the existing
single-CPU non-suspending syscall rule; no borrow or pointer crosses user entry.
Every user copy uses the existing checked boundary and releases its pin before
return. Original scanout unsafe/MMIO rules remain P4-04's responsibility.
