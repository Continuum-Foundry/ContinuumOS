# Experimental M4 boundary and lifetime plan

Status: draft, revision 0, unfrozen. P4-01 groundwork for R14/R18; no guest
consumer or new permanent operation/protocol ID. Only the exact
[SDK subset](sdk-v1.md) remains frozen. Hosted Rust Scene/Frame/Controller layouts
are private values, never a byte protocol. See the
[P4-01 runbook](../05-delivery/phase-4/prerequisites-runbook.md).

P4-01 now implements host-only field encoders/decoders in
`tools/m4_codecs.py`, a separately authored offset/width consumer in
`tools/m4_reference.py`, and [literal byte fixtures](../../fixtures/m4/desktop-v0/fixtures.json).
`tools/Test-P4-01.py` compares both consumers on every truncation, explicit header/
body/context negatives, field swaps and deterministic mutations. Temporary kind
1 identifies the single draft body in each separately bound endpoint family;
input body kinds 1/2/3 cover key/pointer/reset only. Text has no admitted message
yet. State/action masks and role tags are experimental host fixtures, not a
guest service or new public SDK family. Full semantic graph validation, live
leases/focus/authority, physical input and authenticated sessions remain the
owning later stories' tests. The dev decoder's Boolean authentication result is
a trusted session-layer model for host tests, never a wire credential or crypto.

## P4-02 private trusted launch record

P4-02's runtime experiment adds a private read-only page at `0x70000000`, with
six little-endian 64-bit fields: role index at byte 0, admitted heap bytes at 8,
self-process handle at 16, self-thread handle at 24, timer handle at 32 and trusted
workload mode at 40. The rest is zeroed. Heap backing starts at `0x40000000` in
that owner's private address space. Fields are written individually; no Rust
struct layout is transmitted. The kernel assigns the actual identity separately.
Changing a private role word cannot change current-caller provenance or authority.

This record has no public operation ID, negotiation or ABI freeze. The build binds
its exact static payload hashes and profile; record and consumer must change
together and pass P4-02 review. Ordinary mode is zero; acceptance modes are selected
only by registered test policy in the explicitly built desktop test flavor.
The surface/input/semantic/output families below still require their later guest
consumers. See the [runtime runbook](../05-delivery/phase-4/runtime-admission-runbook.md).

## Common desktop message envelope

Propose a 64-byte little-endian envelope for the surface, input, semantic and
output families. Each family is selected by a trusted endpoint binding, not a
caller-chosen global service address. Temporary family-local kinds belong only
to independent draft fixtures. Reject unknown required features, major version,
flags, nonzero reserved bytes, lengths, arithmetic overflow and trailing bytes
before allocation. Minor-version compatibility requires a tested explicit subset;
an unknown message is not silently treated as a known prefix.

| Offset | Bytes | Field |
| --- | ---: | --- |
| 0 | 4 | ASCII `CDU0` draft magic |
| 4 | 2 | Major, initially 0 |
| 6 | 2 | Minor, initially 0 |
| 8 | 4 | Total bytes, including header; at most 4096 |
| 12 | 2 | Family-local draft kind |
| 14 | 2 | Flags, initially zero |
| 16 | 8 | Monotonic endpoint sequence, nonzero, never wraps |
| 24 | 8 | Request/correlation ID, nonzero |
| 32 | 8 | Resource generation |
| 40 | 8 | Output generation |
| 48 | 8 | Logical object ID |
| 56 | 4 | Body bytes, exactly total minus 64 |
| 60 | 4 | Reserved, zero |

Handle attachments use the accepted SDK's atomic move/duplicate semantics,
attenuated rights and trusted sender provenance. Body integers are not handles
or proof of authority. Current session/policy/route/focus/lease checks occur at
use time, including queued delivery and publication. Denied messages close any
accepted moved attachments exactly once. Unsupported versions never leak them.

## Surface proposal: P4-05

Draft commit body is 64 bytes: `lease_id:u64`, `scene_revision:u64`,
`width:u32`, `height:u32`, `stride:u32`, `format:u32`, `buffer_bytes:u64`,
`damage_x:u32`, `damage_y:u32`, `damage_width:u32`, `damage_height:u32`,
`buffer_generation:u64`. Encoding is by field, with independently authored
offset/byte fixtures before integration. Initial format is encoded-sRGB RGBA8
premultiplied alpha, draft format tag 1. Dimensions are nonzero and at most
1920 by 1080; stride is at least width times four, a multiple of four, and at
most 8192. Backing bytes equal stride times height, bounded at 8847360 bytes.
Checked damage sums stay within dimensions. Padding is initialized and never
displayed as pixels. Buffer ID/generation does not grant a mapping.

Acquire returns broker-owned write authority. Commit revokes or copies from the
writer before accepting immutable readers; private RW aliases cannot survive as
a loophole. At most three live leases/surface, eight surfaces globally and two
retained resource generations. In-use buffers cannot be written or reacquired;
release follows the last compositor/presentation reader. Cancellation, client
death and restart have explicit release/quarantine outcomes. P4-05 proves actual
mapping enforcement and exact refs; a Python bounds check proves neither.

Negative fixtures: overflow/pitch mismatch, unsupported format, out-of-bounds
damage, extra/missing attachment, writable retained alias, stale lease/generation,
wrong owner, double commit/release, reader timeout and death during commit.

## Input proposal: P4-06/P4-10

Draft body is 48 bytes: `timestamp_ns:u64`, `scene_revision:u64`,
`focus_epoch:u64`, `device_id:u32`, `event_kind:u32`, `x_16_16:i32`,
`y_16_16:i32`, `code:u32`, `modifiers:u32`. Family-local event kinds cover key,
pointer, text and reset, with their exact text payload added through a separately
versioned bounded UTF-8 message. No arbitrary native event pointer is permitted.
Timestamp and sequence are service-assigned; endpoint identity identifies the
trusted physical input service. An app-authored body cannot impersonate it.
Use signed fixed-point logical coordinates tied to explicit scene/output
generations, checked transforms and hit-test reconciliation. Flags/modifiers
have explicit supported masks. Reset cancels held keys/buttons/capture/composition
and flushes the bounded queue with counted drops. Queue limit is 256 events.

Protected focus has a separate authority route; automation/dev clients cannot
observe or inject synthetic secrets. Diagnostic reports contain bounded reason
codes and sequence IDs, never key text or document/credential contents. M7 owns
real login/credentials. Negative fixtures cover stale sequence/transform/focus,
unknown device/kind, invalid text, overflow, missing key release, owner death,
cross-owner injection and protected-capture attempts.

## Semantic proposal: P4-09

Draft node body is 56 bytes: `node_id:u64`, `parent_id:u64`, `scene_revision:u64`,
`x_16_16:i32`, `y_16_16:i32`, `width_16_16:u32`, `height_16_16:u32`,
`role:u32`, `state_bits:u32`, `action_bits:u32`, `label_bytes:u32`, followed
by strict UTF-8 label bytes, at most 256. No embedded pointer, callback or
credential value. At most 4096 exported nodes, depth at most 32, chunked within
4096-byte messages. Snapshot publication is atomic after validating unique IDs,
parent existence/acyclicity and current scene/resource/output identity. Virtual
rows keep stable logical identity and position/count through a later explicit
extension rather than overloading native pointers.

Action invocation reuses the checked typed-action executor with trusted caller,
scope, document/model revision and expiry. Visual text is not an authority token.
Independent logical assertions own semantic quality; pixel comparisons cannot
accept it. Negatives include duplicate/cyclic nodes, bad role/mask/UTF-8/length,
secret disclosure, stale snapshot, unsupported action and cross-owner invocation.

## Output proposal: P4-11

Draft body is 40 bytes: `output_id:u64`, `topology_revision:u64`,
`pixel_width:u32`, `pixel_height:u32`, `scale_numerator:u32`,
`scale_denominator:u32`, `logical_origin_x:i32`, `logical_origin_y:i32`.
One output only, nonzero stable ID/generation, topology revision 1, origin zero,
720p or 1080p, rational scales 1/1, 3/2 or 2/1. Freeze rounding only after
independent paint/input/semantic fixtures agree at fractional edges. A mode,
scale or replacement output advances generation and invalidates stale commits.
M4 does not admit live topology/mode changes; M22 owns that extension, 4K,
multi-output and hot-plug. Negative fixtures reject zero denominator/IDs,
unsupported dimensions/scales/topology/origin and stale output generations.

## Dev framing and plan proposal: P4-12 through P4-16

Keep the explicit 32-byte `OSU1` envelope in the
[update wire draft](update-wire-format.md); do not tunnel native structs or
reinterpret the desktop envelope as authenticated framing. Length/kind/flags
validation precedes allocation. Use at most 65536 bytes/chunk, 1048576 bytes/frame,
64 MiB/blob, 96 MiB/transaction, 64 blobs and one in-flight transaction. Compression
and executable payloads are disabled. Limit parser depth to 16 and metadata to
64 KiB. Offset/length/coverage/digest and duplicate-overlap checks precede seal.

The draft plan binds boot, authenticated session, transaction ID, current base
and target resource generation, L0/L1 class, source digest and bounded component
graph. No kernel/helper/app code, process launch or behavior substitution. P4-13
selects a reviewed construction and validates target-compatible entropy,
provisioning, key lifecycle, peer/boot binding, replay/downgrade/tamper/expiry and
restart negatives. A plain digest, sequence or loopback socket is not a MAC or
authenticated session. No credential construction is chosen by this plan.

P4-14 validates complete bytes and semantic plan, immutable font/blob handles,
old/new/readers/queues/cache peak admission and a reserved recovery budget.
P4-15 publishes one coherent data root across paint/hit-test/semantics; it releases
old bytes only after the final reader. P4-16 tests every transition fault,
duplicate/reorder/cancel, lost ACK/status query, disconnect, validator death,
unavailable staging and retry. Pre-publication failure keeps the old generation
editable. Post-publication status reflects the authoritative current generation;
ephemeral publication is never called durable or a behavior rollback.

Dev-disabled startup exposes no dev device/endpoint/grant. Serial diagnostics
remain independent and cannot mutate resources. Test both launch shape and real
guest authority absence; an omitted host port alone is insufficient enforcement.

## Fixture, freeze and oracle ownership

P4-01 owns host-only draft length/offset and profile rejection fixtures. They must
include independently supplied literal bytes and a consumer using a different
encoding strategy, plus malformed/overflow/stale-context cases. Generator and
generated decoder agreeing with each other is insufficient. P4-03/05/06/09/11/12
own real independently built service/client exchanges, moved-handle cleanup,
versions and authority/lifetime negatives. P4-13 owns cryptographic review and
P4-14/15 own resource-plan and publication correctness. P4-17 owns clock/sample
validity; P4-18 owns whole-guest accounting; P4-19 reviews the exact exercised
subset before any new freeze. Untested kinds, feature bits and extensions remain
experimental regardless of a schema or host fixture pass.


## P4-03 implementation boundary

[P4-03's reviewed primitives](../../evidence/phase-4/P4-03/final-1794185451085334400/README.md) add an internal trusted device broker and test-flavor CPL3 entitlement/range probes. They do not freeze a public driver/message family or extend the SDK contract. Hardware mappings stay supervisor-only; user roles see checked descriptions, while the temporary kernel mediator owns queues, IRQ and DMA/reset. See the [device runbook](../05-delivery/phase-4/device-authority-runbook.md) for the supported matrix, unsafe review and extraction handoff. Physical input, presentation and serial transport remain later stories.

## P4-04 implementation boundary

[P4-04's checked output](../../evidence/phase-4/P4-04/final-1794635215585403600/README.md) adds experimental internal operations
0x4100–0x4104 for trusted display acquisition/query and bounded RGBX row copying.
They carry explicit integer arguments, signed packed placement and an exact owner
generation through the existing checked syscall boundary; no Rust structure layout
is exchanged. Scanout/capture remain unavailable to apps. Source-release completion
is synchronous CPU copy plus MFENCE, with no retained pointer or VSync promise.
These operations do not freeze a public display/message family or extend the SDK.
P4-05 must retain authority and copy lifetimes while adding surface leases and
composition; see the [runbook](../05-delivery/phase-4/framebuffer-runbook.md).
