# Continuum M3 bounded runtime SDK 0.1.0

Experimental local no-std package for x86_64-unknown-none. `contract.json` is an
identical packaged copy of `contracts/sdk-v1.json`; `fixtures.json` contains
canonical and selected malformed bytes checked by the independent reference.
Only the exact contract hash accepted in the P3-12 final review is frozen.

The register wrapper declares clobbers and exposes fallible typed status/value
results. Pointer-bearing raw syscalls are unsafe. All encodings are explicit
little-endian bytes; native Rust layout is never a wire format. Handle numbers
are manually owned: copying a number is not duplication. Failed receive outputs
are unusable. Blocking helpers retain one absolute monotonic deadline and commit
authority only on successful atomic enqueue/receive.

The optional allocator requires one execution thread and an explicitly supplied
self-process handle. It reserves a 64 KiB RW memory object at 0x52000000, supports
32 live allocations with 64-byte cells and alignment up to 4096, zeroes reused
allocations, and releases its backing on quiescent shutdown. The complete 64 KiB
stays charged until shutdown or process death; freeing cells does not refund
prepaid kernel lifetime credits. No IRQ/signal callback or concurrent entry may
use this heap. Failure leaves old reallocation ownership intact.

Startup parsing does not grant authority. The trusted test launcher separately
delivers bootstrap handles; its fixed roster/control pages are outside the SDK.
Services must combine historical sender metadata with current liveness, finite
session, scoped policy, revision, route epoch and monotonic deadline checks.
Checkpoints contain data and cannot restore handles or grants.

No general app SDK release, dynamic linking, threads/TLS, AVX, wall-clock source,
files, network, accounts, installer, dev transport, font/shaping/raster/display
backend, state migration or durable activation is supplied. Upstream portable
dependencies belong to the independently built example client, not this SDK.
Repository rights and handling restrictions continue to apply.
