# Language evaluation

Status: research synthesis and proposed selection. “Highest-level language” is
not one objective ranking. Abstraction, memory safety, runtime dependence, dynamic
features, compiler maturity and hardware control are separate axes.

## Decision criteria

The foundation must emit freestanding code, expose CPU/memory operations, control
allocation and failure, support our chosen calling convention, and permit a small
auditable runtime. Ergonomics matters, but a language that requires an existing
OS can move work into a runtime port instead of removing it. Fast hardware does
not remove interrupt, DMA, pointer lifetime, or synchronization requirements.

| Candidate | Advantages | Foundation costs | Recommendation |
| --- | --- | --- | --- |
| Rust | Ownership, types, traits, native code, broad systems references | Learning curve; unsafe hardware edges; no stable Rust plugin ABI; compile/link discipline | Proposed kernel and native services |
| Embedded Swift | High-level value types/generics, readable UI potential, bare-metal direction | Language subset; ARC/allocation/platform hooks; rapidly evolving toolchain interfaces; x86 target experiment needed | Credible alternative, not default |
| Zig | Explicit allocation, freestanding control, compile-time facilities | Less automatic lifetime protection; exact compiler/version stability must be checked | Viable alternative for low-level preference |
| C++ | Mature native systems ecosystem and abstractions | Runtime subset/exception/RTTI policy; memory-safety burden; not aligned with desire to move beyond C-family infrastructure | Not selected |
| C# | Expressive managed ecosystem and OS precedents | AOT/runtime, GC, exceptions, platform plugs and metadata work; normal .NET apps do not just run on a new kernel | Research branch if managed OS is the central goal |
| Go / TinyGo | Accessible syntax, concurrency concepts, embedded runtime work | Allocation, GC, scheduler and goroutines must coexist with kernel rules; standard Go target does not solve the port | Not kernel default |
| Python / JavaScript | Excellent iteration and high-level app logic | Interpreter/VM, heap, runtime APIs, performance isolation and OS bindings | Optional user-space scripting later |
| Wasm languages | Language-neutral modules, defined imports, bounded execution possible | Runtime, host ABI, feature subset, state migration and sandbox policy | App/plugin experiment after native core |

Evidence: [Rust target](sources.md#s01), [Swift vision and PAL](sources.md#s06),
[Zig](sources.md#s10), [TinyGo runtime](sources.md#s11),
[Cosmos](sources.md#s12), [Singularity](sources.md#s13).

## Proposed Rust subset

- Prefer stable toolchain facilities. Pin the exact tested release in M0, rather
  than adding nightly by habit. Isolate any required unstable target/interrupt code.
- Kernel uses `#![no_std]`, `core`, and `alloc` only after a correct allocator.
  No host filesystem, sockets, threads or `std::time` are presumed.
- Kernel panic initially aborts into controlled diagnostics. User processes can
  terminate independently; unwinding across FFI or privilege boundaries is forbidden.
- Wrap architecture operations in a small module with `SAFETY` rationale, alignment,
  aliasing, ordering, ownership and interrupt assumptions.
- Restrict floating point/SIMD until context save/restore is implemented and tested.
  Hosted UI floating-point code cannot simply be moved into the kernel target.
- Normal Rust modules inside one address space provide software organization, not
  independent native process isolation.
- Use generated wire DTOs/explicit encoding for replaceable services. No `String`,
  `Vec`, trait object, reference or pointer crosses a native binary boundary by layout.

The proposed userland target is separate from the kernel target. M0/M3 must choose
a target specification, linker script, entry point, allocator, panic behavior and
syscall shim. Do not label `x86_64-unknown-none` plus a kernel code model as a complete
userland ABI. A partial `std` port can be considered after our SDK exists.

## Assembly policy

Use Rust implementations for ordinary algorithms. Permit tiny reviewed wrappers
for entry, exception stubs, context switch, syscall return, control registers,
interrupt flags, port I/O and idle instructions as required. Intrinsics and
`asm!` still emit architecture instructions; “no `.asm` files” does not mean
the architecture-specific obligation disappeared. CPU manual correctness wins
over aesthetic line-count goals. [S04](sources.md#s04), [S30](sources.md#s30)

## UI language strategy

The [toolkit comparison](ui-toolkit-comparison.md) is the selection gate; do not assume a custom UI is already chosen.

Compare Slint with a minimal typed Rust builder using reloadable token/layout data. Once
requirements are understood, add a small declarative schema or proc-macro. Do not
invent a general language, CSS implementation or JS runtime before a window can
be drawn, focused and updated. A schema should describe identity, layout,
semantics, events, resources and animation; behavior belongs in a versioned module.

Wasm may later admit Rust and other languages, but ordinary JavaScript/TypeScript
requires a JS engine or a specifically supported compilation subset. Browser DOM,
Node modules, .NET libraries and WASI APIs are not automatically available.

## Reconsideration gate

Run a bounded Rust kernel/ABI/UI experiment first. Reconsider Swift if it produces
a materially better workflow with proven x86-64 freestanding support and manageable
platform dependencies. Reconsider Wasm as the default app format only after
instruction/fuel limits, host imports, text/UI responsiveness, memory ceilings and
reload state tests succeed. No recommendation here claims benchmark superiority.
