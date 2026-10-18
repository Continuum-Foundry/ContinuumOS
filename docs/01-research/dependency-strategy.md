# Dependency and reuse strategy

The kernel and architecture remain original. Use portable libraries where they
reduce implementation risk without importing another OS as the foundation.
Dependencies are candidates until pinned, compiled and exercised in our environment.

## Candidate inventory

| Candidate | Placement / intended use | Required integration check | Research |
| --- | --- | --- | --- |
| Limine + protocol bindings | Boot path | Pin matching releases/protocol; validate response metadata and binary licenses | [S25](sources.md#s25), [S26](sources.md#s26) |
| Rust OSDev bootloader | Boot alternative | Exact toolchain/image path plus actual UEFI ISO boot; disk-image success is insufficient | [S81](sources.md#s81) |
| x86_64 crate | Architecture support | Pin CPU/interrupt/paging feature subset; audit unsafe mapping assumptions | [S83](sources.md#s83) |
| Slint | Competing UI prototype | Exact guest features/backend, text/effects, reload, semantics and distribution terms | [comparison](ui-toolkit-comparison.md) |
| OVMF | QEMU UEFI firmware | Known firmware build, writable variable-store policy, checksum/license | [S27](sources.md#s27) |
| virtio-drivers | Driver mechanism | Exact feature subset; DMA HAL, PCI transport, reset semantics | [S33](sources.md#s33) |
| Small rasterizer / tiny-skia | CPU rendering | Allocations, float/SIMD, no-OS features, pixel layout, worst-case input | [S53](sources.md#s53) |
| rustybuzz + font rasterizer | Text shaping/rendering | Guest build, font parsing limits, Unicode/version fixtures, fallback | [S52](sources.md#s52) |
| Wasmi | Optional app runtime | no-std feature graph, heap, stack/fuel limits and import authority | [S65](sources.md#s65) |
| Burn CPU backend | Optional inference after M5 | Supported model/operators, allocator/FP/SIMD, quotas, model license and actual memory | [S80](sources.md#s80) |
| Wasmtime | Later AOT/runtime path | Platform functions, memory protection, trap handling, code cache trust | [S66](sources.md#s66) |
| smoltcp | Network service | Poll/timer integration, buffers, entropy needs and exposed feature set | [S57](sources.md#s57) |
| rustls + reviewed crypto provider | TLS service | Time, randomness, provider architecture/dependencies and trust anchors | [S59](sources.md#s59) |
| AccessKit or adapted semantic model | Accessibility | Custom OS adapter and native assistive service, not just host toolkit support | [S51](sources.md#s51) |
| SQLite | Possible future settings database | Implement genuine file locking, sync, atomicity and recovery semantics | [S55](sources.md#s55) |
| littlefs | Storage comparison only | Upstream C, flash assumptions, adaptation costs and directory semantics | [S56](sources.md#s56) |
| Vello variant | Renderer research only | Select exact variant, backend, guest portability and memory/driver costs | [S54](sources.md#s54) |
| xorriso | Host ISO packaging | Tested platform availability and pinned boot recipe; no guest dependency | [S62](sources.md#s62) |

The original research supplied no lockfile. Phase 0 now supplies separate host and
dependency-experiment lockfiles plus a [compile/tooling report](../05-delivery/phase-0.md).
These do not admit the candidates to a production kernel or UI stack.
Slint, Burn and the alternative bootloader are candidates, not new mandatory guest dependencies. Inventing hashes or version
pins without a build would create false reproducibility. M0 produces the real
toolchain lock and dependency inventory.

## Admission checklist

For each guest dependency, record exact version/commit, digest, license expression,
transitive dependencies, feature flags, MSRV, build scripts, native tool requirements,
`std`/OS assumptions, allocator use, panic behavior, unsafe code boundary, supported
CPU features and parser attack surface. Cross-compile it independently before
adding it to the kernel or a privileged service.

`no_std` is necessary for many candidates but not sufficient. It can still require
allocation, floating point, atomics, thread-local storage, host imports or external
symbols. Disable default features deliberately and inspect the resulting graph.
Do not replace a missing platform function with a success-returning stub.

## Reuse versus authoring

Author the capability policy, kernel object integration, IPC contracts, service
supervision, migration rules and desktop interactions. Prefer maintained primitives
for cryptography and complex font shaping. A new kernel does not require a new
cipher or a superficially correct Unicode implementation.

Use content hashes for package identity, but keep publisher identity and permissions
separate. Signed malicious code is still malicious. Package verification must occur
before executing build plugins, migrations or post-install actions in the guest.

## Licensing and supply chain

Choose the first-party project license before public source release. Dependencies,
fonts, icons and sample media each need a record. Host tools and shipped guest
components have different distribution consequences; document both without making
unsupported legal conclusions. Preserve notices and create an SPDX-compatible
inventory. [S63](sources.md#s63)

Prefer vendored or cached exact inputs for offline builds. No opportunistic network
download in a guest build script. Host build scripts are code with host authority;
audit them and isolate untrusted inputs. Record provenance for generated assets,
code and binary firmware. Updates to toolchain/dependencies get their own review
and must rerun relevant boot, protocol, migration and graphics fixtures.
