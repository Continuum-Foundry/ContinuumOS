# Independent SDK and nonrendering runtime experiment

P3-12 is Done under [local self-review](../../../evidence/phase-3/P3-12/2026-11-01/README.md). Only the exact reviewed contract hash is frozen. No display backend,
guest UI confirmation, state migration or M3 exit is accepted.

## Inputs and separation

`sdk/continuum-rt` is a standalone no-std Rust package, version 0.1.0, with no
dependencies. `contracts/sdk-v1.json` selects the tested public IDs from the
experimental runtime source. `tools/sdk_contract.py` checks generated bindings
against both sources and independently implements little-endian encodings and
structural validation. Its host production consumer calls SDK code; its Python
reference does not. Byte mutation, truncation, reserved fields, enum values,
feature bits, rights bits, minor versions and major incompatibility are exercised.
Pointer validity, actual object rights and current authority are guest checks.

`tools/Build-P3-12Sdk.ps1` copies allowlisted public sources into three fresh
roots and builds locked/offline in each root's independent Cargo output tree.
The client depends only on the SDK, selected `editor-model`, pinned
unicode-segmentation 1.13.3 and libm 0.2.16. Services depend only on the SDK and
packaged example policy/route implementations. No kernel manifest, module or
private native layout is available in any root. The client package contains no
provider implementation. Service header encoding/decoding is separately written
and does not call the production header codec.

The client is built once. Its SHA-256 is checked after each service build. The
old service uses protocol minor 0 with no optional feature; the rebuilt service
uses minor 1 and feature bit 0. Both use declared SDK 0.1.0; changing a service
protocol minor is not represented as an invented SDK release. Each retains its
own source/lock/compiler arguments, dependency graph and artifact identity.

## Commands

Dot-source `tools/Enter-Dev.ps1`. Run signing, xorriso and guest commands in the
already approved execution context. Keep the host awake; do not alter deadlines,
power settings, signing enrollment or security policy. Acceptance is serial.

```powershell
python tools/sdk_contract.py --output build/m3/P3-12/codecs
cargo build --locked --offline --manifest-path sdk/continuum-rt/Cargo.toml --bin conformance
python tools/sdk_contract.py --output build/m3/P3-12/codecs --production build/cargo/signed/debug/conformance.exe
cargo xtask test --suite m1 --case sdk
cargo xtask test --suite m1 --case sdk-runtime
pwsh -NoProfile -File tools/Test-P3-12.ps1
python tools/Review-P3-12.py --acceptance <printed-acceptance-json> --destination evidence/phase-3/P3-12/<run-id>
```

The final-source acceptance/review and compact retained bundle are linked above.
New executions are development probes until reviewed; no interrupted probe counts as a pass.
The `sdk` case builds, packages and executes; it is not compile-only. Separate
metadata flavor checks supplement actual ordinary/boundary/scheduler/capability
and later runtime regression guests.

## Guest scenario and runtime

The trusted adapter supplies an immutable roster and explicit process-local
handles. These fixed fixture pages are test bootstrap, not a public discovery
namespace. The SDK has no knowledge of their addresses. The 64-byte public
startup parser checks magic/revision/length/page/reserved fields; this bounded
launcher still supplies zero startup feature/bootstrap/clock fields. Real elapsed
time is read through the HPET clock operations. An SDK entry adapter handles a
normal Rust return by invoking rights-checked exit with its retained self handle;
the legacy staged UD2 and old integer-only probes remain separate.

The unchanged client exchanges versioned 48-byte headers and 16-byte payloads
with both service artifacts. Compatible old/new minors work, incompatible major,
unknown mandatory feature, header/payload reserved bits and mismatched payload
sequence reject before provider mutation. The compatible-minor case also executes
the echo operation with the original client artifact. An immutable zero-filled object moves atomically with attenuated read
rights; the old sender handle becomes invalid and receiver write escalation
fails. Trusted receive metadata and current liveness are checked separately from
payload identity. Existing ephemeral route, policy, fencing, duplicate/reply,
expiry and failure-cut scenarios then run across private old/new service images.

A separate `sdk-runtime` ISO uses its own runtime-feature client ELF and kernel
identity. It executes six processes in one runtime incarnation, independently of
the nine route incarnations. Its allocator reserves one 64 KiB RW memory object,
closes the source handle after mapping, and reclaims 64-byte cells and metadata
for up to 32 live allocations with alignment at most 4096. The entire backing
stays charged until quiescent explicit unmap or process death. This preserves
prepaid kernel lifetime allocation credits instead of consuming a fresh object
on every heap allocation. No kernel ceiling or guest deadline is raised.

The nonrendering smoke checks full-heap/slot exhaustion and reuse, zero-fill,
reallocation and failure atomicity, grapheme boundaries, selected model creation
and destruction, libm results and distinct live XMM markers across preemption.
Normal return, fallible Rust allocation failure, panic containment, healthy
continuation and explicit heap shutdown are observed. Kernel and user instruction
reviews distinguish admitted SSE2 from the historical integer-only fixtures.
No fonts, shaping, rasterization, scene or display pipeline runs in this guest.
Exact final object/frame/table/mapping/stack/pin/wait settlement is mandatory for
both ISO guests. Their two boot identities do not demonstrate live UI continuity.

## Authority and later handoff

Process generation, authority session, policy epoch and route epoch are separate
domains. `MonotonicNs` and `WallClockNs` have no implicit conversion; no wall-clock
source is implemented. The supported SDK clock and deadlines are monotonic HPET
elapsed nanoseconds, distinct from delivered-tick uptime. A wake is level readiness,
not an ownership reservation. Blocking send/receive composition retries the same
absolute deadline; only successful enqueue/receive commits authority. A failed
receive can alter output bytes but installs no handle and consumes no message.

M4 must bind its authenticated development session to current trusted caller
provenance; no dev transport, credentials or host command execution exists here.
It must supply real input translation/order/reset/focus, immutable font/resource
lifetimes, semantic export, bounded callbacks, frame opportunities, surface/damage
leases and actual compositor/display ownership. The selected full portable text
and raster paths retain M4 parser/feature/instruction and guest confirmation gates.

M5 must check provider scope, revision, session/policy/route epoch and monotonic
deadline at use, freshly authorize replacements, fence the old writer and retain
old code until no execution can reach it. Checkpoints contain data only: no live
handles, credentials, grants or developer authority. IDs and serialized contexts
cannot restore authority. Closing a route does not universally revoke duplicates
or mappings. File/control/diagnostic service seeds remain interfaces; supervision
27–33 stays experimental. No account/installer/network/presentation family is
frozen by this bounded SDK.
