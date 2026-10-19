# Primary source catalogue

Access/review date for every entry: **2026-10-17**. O = direct page opened; I =
indexed primary-source excerpt inspected. Findings below are short paraphrases.
Source documentation can change; a source URL is not a package or toolchain pin.
Project proposals in other documents are our synthesis, not endorsements by these sources.

## Languages, runtimes and precedents

### S01
[Rust: x86_64-unknown-none](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html) — O.
Freestanding ELF target, without `std` or a default allocator. Default codegen avoids vector/floating-point registers; explicit OS support is needed before enabling them. Use: compiler feasibility.

### S02
[Rust type layout](https://doc.rust-lang.org/reference/type-layout.html) — I.
Only documented layout guarantees can be relied upon across compilations. `repr(C)` does not recursively stabilize embedded Rust-layout types. Use: reject raw object-layout persistence.

### S03
[Rust external blocks](https://doc.rust-lang.org/reference/items/external-blocks.html) — I.
Language and foreign calling conventions are explicit ABI concerns. Use: narrow syscall/FFI edges; a C ABI does not require C implementation source.

### S04
[Rust inline assembly](https://doc.rust-lang.org/reference/inline-assembly.html) — I.
Compiler-supported architecture-specific assembly has explicit operand, register and correctness rules. Use: audited CPU boundary rather than a claim of zero machine-specific code.

### S05
[Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) — O.
Optimization, debug data, panic strategy and other profile properties are configurable. Use: distinguish fast iteration builds from measured release builds.

### S06
[Embedded Swift vision](https://github.com/swiftlang/swift-evolution/blob/main/visions/embedded-swift.md) — I.
Describes a compilation model for constrained, bare-metal and low-level software. Use: viable language alternative, subject to actual toolchain/target tests.

### S07
[Swift embedded getting started](https://www.swift.org/get-started/embedded/) — I.
Official embedded software entry point and hardware examples. Use: practical ecosystem evidence, not proof of our x86 kernel support.

### S08
[Embedded Swift PAL proposal by maintainers](https://forums.swift.org/t/introducing-the-embedded-swift-platform-abstraction-layer/89248) — I.
Explains platform dependencies and a deliberate interface replacing ad hoc hooks. Use: assess runtime porting and toolchain change risk.

### S09
[Embedded Swift documentation](https://docs.swift.org/latest/documentation/embeddedswift/) — I.
Documents language subset and external dependencies; inspected support wording includes an experimental note. Use: avoid treating full Swift and Embedded Swift as interchangeable.

### S10
[Zig language reference](https://ziglang.org/documentation/master/) — O.
Language facilities include freestanding targets, explicit allocation and assembly interfaces. Use: systems-language alternative; exact development-version syntax is not pinned.

### S11
[TinyGo runtime/package organization](https://tinygo.org/docs/guides/contributing/organization/) — I.
TinyGo substitutes its own runtime for facilities such as allocation, scheduling and channels. Use: Go-like language feasibility does not eliminate runtime engineering.

### S12
[Cosmos upstream](https://github.com/CosmosOS/Cosmos) — O.
Managed-language OS construction framework with its own tooling. Use: C# precedent; adopting the framework wholesale would change our from-scratch boundary.

### S13
[Singularity: Rethinking the Software Stack](https://www.microsoft.com/en-us/research/publication/singularity-rethinking-the-software-stack/) — I.
Original research on software-isolated processes, communication contracts and manifests. Use: high-level OS precedent, not a performance claim for this project.

### S14
[Theseus, OSDI 2020](https://www.usenix.org/conference/osdi20/presentation/boos) — I.
Original Rust OS research linking modularity and state management to live evolution and recovery. Use: important precedent; do not claim first-ever live OS updating.

### S15
[MINIX live update developer guide](https://wiki.minix3.org/doku.php?id=developersguide%3Aliveupdate) — I.
Describes service quiescence and update/state-transfer mechanisms. Use: service safe-point design and limitations.

### S16
[Erlang code loading](https://www.erlang.org/doc/system/code_loading.html) — I.
Documents current and old code versions and how execution transitions between them. Use: code lifetime and retirement are explicit lifecycle problems.

### S17
[Linux livepatch consistency](https://docs.kernel.org/6.14/livepatch/livepatch.html) — I.
Explains consistency when redirecting functions and safe switching between implementations. Use: kernel live patching is narrower than arbitrary kernel restructuring.

### S18
[Linux livepatch system state](https://docs.kernel.org/6.9/livepatch/system-state.html) — I.
State-changing patches can make reversal unsafe; state/version bookkeeping is needed. Use: rollback cannot be inferred from keeping an old binary.

## Isolation, boot and platform

### S19
[seL4 capabilities tutorial](https://docs.sel4.systems/Tutorials/capabilities.html) — I.
Capabilities express authority over kernel objects. Use: capability model precedent; this project does not inherit seL4 proofs.

### S20
[seL4 proof assumptions](https://sel4.systems/Verification/assumptions.html) — I.
Documents trust boundaries including hardware, boot/assembly and DMA behavior. Use: language or kernel proofs do not automatically cover malicious DMA.

### S21
[Zircon handles](https://fuchsia.dev/fuchsia-src/concepts/kernel/handles) — O.
Handles are process-local object references with rights, transfer and lifetime semantics. Use: kernel-enforced handles and restricted delegation.

### S22
[Zircon kernel concepts](https://fuchsia.dev/fuchsia-src/concepts/kernel/concepts) — I.
Channels and sockets are distinct IPC objects; channels transport messages and handles. Use: separate control messages from bulk streams.

### S23
[Fuchsia component capabilities](https://fuchsia.dev/fuchsia-src/concepts/components/v2/capabilities) — I.
Component authority is deliberately routed through declared relationships. Use: service access should not arise from ambient global naming alone.

### S24
[Genode components overview](https://genode.org/documentation/components) — I.
Describes components, drivers and device-independent service interfaces. Use: driver/service decomposition precedent.

### S25
[Limine bootloader upstream](https://github.com/limine-bootloader/limine) — I.
Multi-protocol bootloader, with BSD-2-Clause shown upstream. Use: bootstrap reuse candidate; verify exact release and notices before distribution.

### S26
[Limine protocol upstream](https://github.com/limine-bootloader/limine-protocol) — O.
Primary protocol documentation repository. Use: pin boot requests/responses rather than copying a tutorial's arbitrary memory layout.

### S27
[UEFI boot services](https://uefi.org/specs/UEFI/2.10/07_Services_Boot_Services.html) — I.
The loader must use the current memory-map key at `ExitBootServices`; boot services end after successful handoff. Use: own-loader alternative and memory ownership.

### S28
[UEFI console and Graphics Output Protocol](https://uefi.org/specs/UEFI/2.10/12_Protocols_Console_Support.html) — I.
Defines preboot graphics facilities. Use: discover framebuffer metadata; do not call firmware graphics functions after boot-services exit.

### S29
[ACPI specification](https://uefi.org/specs/ACPI/6.6/) — I.
Platform description and power/configuration interfaces. Use: discover interrupt/timer/platform tables; full AML and laptop power management remain separate work.

### S30
[Intel architecture manuals](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html) — I.
Authoritative CPU architecture source for paging, protection, interrupts and multiprocessor support. Use: first-party low-level implementation rules.
P1-04 review, 2026-10-19: [Volume 3A revision 090](https://cdrdv2-public.intel.com/874249/253668-090-sdm-vol-3a.pdf),
sections 7.13–7.14, 10.2 and 10.7, supplies the fatal-entry frame, IDT/IST and TSS contract.
Implementation/measurement boundaries are in the [exception runbook](../05-delivery/phase-1/exception-diagnostics-runbook.md).

### S31
[ELF program headers](https://refspecs.linuxfoundation.org/elf/gabi4%2B/ch5.pheader.html) — I.
Defines loadable segments and their metadata. Use: validate loader ranges, permissions, alignment and sizes.

### S32
[VirtIO 1.2 specification](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html) — I.
Normative virtual-device interfaces including queues and device-specific operations. Use: initial negotiated feature subset; newer draft features must not be assumed.

### S33
[virtio-drivers crate](https://docs.rs/virtio-drivers/latest/virtio_drivers/) — O.
Bare-metal guest driver library requiring a HAL for DMA allocation/address translation. Use: candidate reusable mechanism; HAL safety stays our responsibility.

### S34
[QEMU WHPX](https://www.qemu.org/docs/master/system/whpx.html) — I.
Documents Windows Hypervisor Platform acceleration. Use: Windows host feasibility experiment; availability depends on host setup.

### S35
[QEMU GDB support](https://www.qemu.org/docs/master/system/gdb.html) — O.
Supports guest debugging through the gdbstub. Use: boot/exception diagnostics; `-s -S` is a debugging mode, not a production launch configuration.

### S36
[QEMU QMP reference](https://www.qemu.org/docs/master/interop/qemu-qmp-ref.html) — O.
Machine control protocol for host tooling. Use: automate observations and test lifecycle; it does not implement guest state migration for us.

### S37
[QEMU VirtIO GPU](https://www.qemu.org/docs/master/system/devices/virtio-gpu.html) — I; a later direct open failed.
Documents 2D and accelerated backend families. Use: separate display transport, host backend and guest graphics stack requirements.

### S38
[QEMU security model](https://www.qemu.org/docs/master/system/security.html) — O.
Security guarantees depend on supported use cases/configurations; TCG is not offered as a security isolation guarantee. Use: trusted development workloads and cautious host exposure.

## Graphics, interaction and text

### S39
[Qt Quick scene graph](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph.html) — I.
Retained scene representation enables batching and state-change reduction. Use: architecture principle; no dependency on Qt is selected.

### S40
[Qt renderer details](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph-renderer.html) — I.
Discusses retention of GPU geometry and compatible batching. Use: make rendering proportional to actual changes; do not copy toolkit benchmarks as ours.

### S41
[Wayland architecture](https://wayland.freedesktop.org/architecture.html) — I.
Compositor-centered input and surface-damage model. Use: eliminate unnecessary display intermediaries; our initial protocol is not Wayland-compatible.

### S42
[Wayland protocol specification](https://wayland.freedesktop.org/docs/html/apa.html) — I.
Damage, buffer lifetime and visibility-sensitive frame callbacks are explicit. Use: frame pacing and shared surface ownership.

### S43
[Flutter rendering performance](https://docs.flutter.dev/perf/ui-performance) — I.
Explains expensive offscreen layers and the memory/work tradeoffs of caching. Use: selective effects and measured caches, not blanket offscreen rendering.

### S44
[Android SurfaceFlinger and WindowManager](https://source.android.com/docs/core/graphics/surfaceflinger-windowmanager) — I.
Separates app buffers, surface control and composition decisions. Use: app execution need not redraw every window transformation.

### S45
[HarfBuzz shaping concepts](https://harfbuzz.github.io/shaping-concepts.html) — I.
Shaping maps Unicode text to correctly arranged glyphs using script/font rules. Use: text is more than per-codepoint glyph lookup.

### S46
[Unicode bidi algorithm](https://www.unicode.org/reports/tr9/) — I.
Defines bidirectional ordering. Use: correct mixed-direction layout and cursor/selection behavior.

### S47
[Unicode text segmentation](https://www.unicode.org/reports/tr29/) — O.
Defines grapheme, word and sentence boundaries. Use: editing must not split user-perceived characters by byte or codepoint.

### S48
[Unicode line breaking](https://www.unicode.org/reports/tr14/index.html) — I.
Defines line-break properties and rules. Use: wrapping and language-appropriate line boundaries.

### S49
[WCAG 2.2](https://www.w3.org/TR/WCAG22/) — O.
Web accessibility requirements provide useful contrast, focus and interaction references. Use: guidance, not a claim that an OS is WCAG certified.

### S50
[WAI keyboard interaction guidance](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/) — I.
Describes predictable focus and keyboard conventions. Use: native desktop interaction design by analogy, not a web-only UI dependency.

### S51
[AccessKit](https://accesskit.dev/) — O.
Accessibility infrastructure for toolkits with platform adapters. Use: semantic tree candidate; our custom OS requires its own adapter and assistive services.

### S52
[rustybuzz upstream](https://github.com/harfbuzz/rustybuzz) and [manifest](https://github.com/harfbuzz/rustybuzz/blob/main/Cargo.toml) — O.
Rust shaping implementation candidate. Use: evaluate exact no-OS build/features, fonts, allocations and license; portability is not yet demonstrated.

### S53
[tiny-skia upstream](https://github.com/linebender/tiny-skia) — O.
Portable Rust rasterizer candidate. Use: software-rendering comparison spike, not an assumed guest port.

### S54
[Vello upstream](https://github.com/linebender/vello) — O.
Rust rendering project with renderer variants and distinct hardware requirements. Use: evaluate a specific renderer/version; do not treat the project name as one stable dependency.

## Storage, security, delivery and testing

### S55
[SQLite atomic commit](https://www.sqlite.org/atomiccommit.html) — I via official mirrors.
Explains ordering, flush assumptions and crash-recovery implications. Use: durable updates need a real storage contract; adding SQLite alone cannot repair a broken flush path.

### S56
[littlefs design](https://github.com/littlefs-project/littlefs/blob/master/DESIGN.md) — I.
Power-loss-resilient filesystem design for constrained systems. Use: storage tradeoff/reference; upstream C and flash-oriented assumptions need explicit evaluation.

### S57
[smoltcp upstream](https://github.com/smoltcp-rs/smoltcp) and [manifest](https://github.com/smoltcp-rs/smoltcp/blob/main/Cargo.toml) — O.
Rust TCP/IP stack with configurable features. Use: candidate network service library; device, timer, allocation and socket integration remain ours.

### S58
[TLS 1.3, RFC 8446](https://www.rfc-editor.org/info/rfc8446/) — I.
Protocol reference. Use: reuse a maintained TLS implementation rather than inventing cryptography.

### S59
[rustls upstream](https://github.com/rustls/rustls) and [CryptoProvider](https://docs.rs/rustls/latest/rustls/crypto/struct.CryptoProvider.html) — O/I.
TLS implementation whose crypto provider includes secure randomness and platform requirements. Use: no-OS porting audit; do not assume default crypto dependencies are pure Rust.

### S60
[The Update Framework specification](https://theupdateframework.github.io/specification/latest/) — O.
Separates signing roles and addresses rollback/freeze/mix-and-match attacks. Use: later distribution trust model, distinct from local development authentication.

### S61
[SOURCE_DATE_EPOCH](https://reproducible-builds.org/docs/source-date-epoch/) — O.
Defines a standardized build timestamp input. Use: one piece of reproducibility, alongside input ordering, tool pins and complete artifact comparison.

### S62
[GNU xorriso](https://www.gnu.org/software/xorriso/) and [manual](https://www.gnu.org/software/xorriso/man_1_xorriso.html) — I.
ISO tooling and El Torito support. Use: package a bootable image with the chosen loader's pinned recipe; ISO is not automatically a USB disk image.

### S63
[SPDX overview](https://spdx.dev/learn/overview/) — O.
Software bill-of-materials and licensing interchange standard. Use: dependency/license inventory; legal conclusions remain outside this architecture research.

### S64
[SLSA provenance v1.2](https://slsa.dev/spec/v1.2/provenance) — O.
Build provenance specification entry. Use: record build inputs and producer context; an attestation alone does not confer a security level.

### S65
[Wasmi upstream](https://github.com/wasmi-labs/wasmi) — O.
Interpreter with no-std support and fuel metering. Use: bounded app/plugin experiment; imports and OS integration still determine authority and resource limits.

### S66
[Wasmtime platform support](https://docs.wasmtime.dev/stability-platform-support.html) and [minimal embedding](https://docs.wasmtime.dev/examples-minimal.html) — I.
Custom platforms require explicit embedder functions, allocation and configuration. Use: later AOT/runtime evaluation rather than immediate dependency.

### S67
[Wasmtime security](https://docs.wasmtime.dev/security.html) — O.
Runtime threat model and sandbox considerations. Use: Wasm isolation is not an absolute guarantee against engine bugs or harmful host imports.

### S68
[WebAssembly core specification](https://webassembly.github.io/spec/core/) — O.
Execution and validation reference. Use: pin a supported feature subset for the chosen runtime; a moving spec is not a compatibility promise.

### S69
[WIT reference](https://component-model.bytecodealliance.org/design/wit.html) — I.
Interface descriptions and resource concepts. Use: possible future binding generator; full Component Model/WASI support is not selected for MVP.

### S70
[Loom](https://docs.rs/loom/latest/loom/) — O.
Concurrency exploration tooling. Use: host-side models of carefully scoped algorithms, not proof of hardware/interrupt code.

### S71
[Miri](https://github.com/rust-lang/miri/) — I.
Interpreter checking classes of Rust undefined behavior with target/operation limits. Use: host-testable parsers and unsafe abstractions where supported.

### S72
[USB HID specifications](https://www.usb.org/hid) — I.
Input-device class and usage specifications. Use: preserve physical key/usage distinctions; a full USB host stack is deferred.

### S73
[JSON Schema 2020-12 core](https://json-schema.org/draft/2020-12/json-schema-core) and [validation](https://json-schema.org/draft/2020-12/json-schema-validation) — O.
Structural schema and validation vocabularies. Use: draft manifest shapes; syntax checks do not prove payload integrity, authority or semantic compatibility.

### S74
[Ajv JSON Schema support](https://ajv.js.org/json-schema.html) — O.
Validator documentation including draft-specific support. Use: evaluated validator alternative; the scratch installation failed and the actual audit used PowerShell Test-Json instead.

### S75
[PowerShell Test-Json](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/test-json?view=powershell-7.5) — I.
Official schema-validation command documentation; describes the newer JsonSchema.NET implementation. Use: built-in validator for this documentation's synthetic manifest fixtures; the audit records actual PowerShell version.

## Reconciliation research (revision 2)

### S76
[Slint backends and renderers](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backends_and_renderers/) — O.
Documents custom backends, software rendering and current limitations. Use: compare a concrete no-OS UI configuration against our effects/text requirements.

### S77
[Slint platform interfaces](https://docs.slint.dev/latest/docs/rust/slint/platform/) — O.
Defines platform/window adapters, event delivery and renderer integration. Use: toolkit-to-OS prototype; it does not supply our compositor or native services.

### S78
[Slint live preview](https://docs.slint.dev/latest/docs/slint/guide/tooling/live-preview/) — O.
Uses runtime interpretation for development reload; native API changes can require application restart. Use: distinguish hosted preview from guest interpreter support and compiled behavior replacement.

### S79
[Slint license terms](https://slint.dev/terms-and-conditions) — O.
Primary terms for available license routes and distribution restrictions. Use: record the intended distribution and selected terms; no license applicability conclusion is asserted here.

### S80
[Burn no-standard-library inference](https://burn.dev/books/burn/advanced/no-std.html) — O.
Official embedded CPU inference example. Use: optional small-model experiment; operators, backend, quantization, memory and CPU-state handling still need exact-target validation.

### S81
[Rust OSDev bootloader image workflow](https://github.com/rust-osdev/bootloader/blob/main/docs/create-disk-image.md) — O.
Describes BIOS/UEFI disk images and a sample build arrangement. Use: candidate loader alternative; separate ISO packaging and toolchain requirements need tests.

### S82
[Kani Rust feature support](https://model-checking.github.io/kani/rust-feature-support.html) — O.
Documents supported language features and concurrency limits. Use: focused bounded logic harnesses, not a concurrent-kernel or device proof.

### S83
[x86_64 crate](https://docs.rs/x86_64/latest/x86_64/) — O.
CPU register/instruction/paging abstractions. Use: candidate architecture mechanism; our mapping, ownership, interrupt and safety obligations remain explicit.

## Access limitations

Redox book direct access returned 403. A TinyGo runtime URL and a Linux DMA guide
URL failed direct retrieval. Initial seL4/Fuchsia URLs without the expected suffix
were corrected through indexed upstream pages. QEMU GPU and UEFI newer-version GOP
direct opens were intermittently unavailable; the older official GOP chapter and
indexed QEMU device documentation support only the limited findings above.

No inaccessible source is used as the sole basis for a critical accepted decision.
Before selecting a library, use its exact manifest, license, release source and
local build experiment, rather than this catalogue as a substitute for integration.
