# P2-07 — Standalone UI final-link probes

These are independent compile/link experiments. Neither ELF is bootable or
executed. They introduce no kernel dependency, heap, process loader or guest UI.
The same `custom-ui` implementation and extracted `slint-ui` no-std AOT/text-image
adapter are used by the hosted harness and these probes. This link experiment
selects no toolkit; P2-09 separately records the
[provisional custom decision](ui-selection-review.md). Neither result executes a guest UI.

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Test-P2-07.ps1
```

Use the enrolled developer account: the ordinary sandbox has a different
certificate store. The existing compiler signer lock enrollment was refreshed
with the owner's explicit approval after automatic review required confirmation.
All root registry pins/checksums are unchanged, and the isolated lock's registry
pins match them. No certificate or Windows policy changed. Keep every dependency
operation offline; `Supply-TextInputs.ps1` verifies the existing six font inputs.

`-PrerequisiteEvidence <path/to/prerequisites.json>` reuses an independently
retained successful `Recheck-P2Prerequisites.ps1` result. The runner requires all
ten checks and records its exact hash. Without it the runner performs a fresh
recheck. The prerequisite recheck includes fresh Phase 0 build/link/doctor,
source paths with spaces, dependency compilation and M1 doctor; it is not a new
VM boot. P1-07's previous acceptance remains the guest baseline.

## Exercised paths and allocation

`experiments/ui-link` has an isolated Cargo workspace/lock, with optional
`slint-aot`. The exported one-shot `ui_probe_entry(seed, milliseconds)` creates a
small editor/list scene, dispatches a real Controller text event/typed action,
validates all six font hashes, and calls actual layout/shaping/raster APIs.
The original P2-07 custom probe used `paint::render`. P2-13 updates only that
selected branch to `retained::Frame::draw`, retaining damage planning and the
bounded layout cache as well as raster output. Its clean link must preserve both
retained-draw and cached-layout symbols. Slint uses the exact shared `slint_ui::paint`, AOT
component, scene-to-layer builder, text-image adapter, platform window and software
renderer. Both create semantic observations and commit/release the common surface.
The returned checksum depends on output pixels, counters, semantics and allocation,
keeping those paths live through section garbage collection. The fixture covers
API closure; hosted regression runs cover the full locked reference corpus.

The probe owns a **64 MiB BSS arena**, aligned to 4096 bytes. It aligns absolute
addresses, checks addition and extent before reserving a disjoint range with CAS,
and returns null without moving the cursor on exhaustion. `dealloc` intentionally
does not reclaim; default `realloc` allocates/copies and keeps the old block valid
on failure. Host tests cover above-base alignment, disjoint writable ranges, exact
limit, exhaustion, zeroing and realloc success/failure. It is a real finite
one-shot allocation provision, unsuitable for repeated rendering. The arena's
reservation is not measured live heap usage or evidence of the 256 MiB idle target.
OOM/panic spins terminally; no test executes the ELF or claims process recovery.

The probe entry has no startup code. Any future caller would need a loaded,
zeroed image, SysV stack alignment/return address, DF clear, proper FP/SIMD context,
and exclusive one-thread ownership. Rust layouts here are internal implementation
types, not frozen messages or a process ABI. Fonts are local immutable input bytes,
never system discovery. No production memory, syscall or host-service stub is used.

## Audit and negative cases

Full outputs stay under ignored `build/m2/P2-07/acceptance-*/`: compiler logs,
exact commands/features, both ELFs, link maps, LLVM reports, generated AOT LLVM IR,
full disassembly, source hashes and dependency manifests. `Review-UiLinkInputs.py`
separates target normal edges from host build/proc-macro inputs and verifies each
active registry archive and expanded source member. It inventories declared terms,
MSRV, features, build scripts and notice hashes. This is not a release SBOM or
license clearance. The existing upstream notices still apply; no new registry
dependency is adopted or font/binary published.

`Review-UiLink.py` parses ELF independently of the linker and corroborates with
LLVM readobj/nm/objdump. It requires a static x86-64 ELF at 0x400000 with distinct
RX/R/RW loads, retained real API symbols, no named undefined symbols, no remaining
relocations, dynamic/TLS/host/runtime-unwinder linkage or writable executable sections.
Small passive `.eh_frame`/header records on Rust allocation shims are retained
and reported; personality/LSDA augmentation and exception landing-pad tables are
rejected. They are not discarded to force a zero-metadata result.
It scans actual emitted instruction operands/opcodes for FP/SIMD and retains
counts and excerpts. This detects presence; it is not a formal ISA/unsafe-code
proof. Font/math dependencies and compiler intrinsics still require review and
user FP/SIMD policy before execution. M1's kernel path remains unchanged.

Each candidate must fail final linking when `missing-hook` injects an actual
undefined presentation symbol, and fail compilation when `host-feature` requests
`std` for the freestanding target. The auditor must reject mutated machine/entry,
WX, dynamic/interpreter/TLS, relocation, unresolved-symbol and truncated images.
There is no ignored-unresolved option or linker-script discard of runtime evidence.

Slint's compiler-generated target LLVM IR and final AOT/runtime symbols are
audited after code generation; no claim of manually reviewing every expanded
Rust statement is made. Exact `slint-interpreter = 1.18.1`, defaults disabled,
gets a separate offline availability attempt. Its missing input is not waived
or presented as a guest compile result. Only AOT is the tested Slint guest path.

## Runtime handoff and comparison

The complete [hook inventory](../../../experiments/ui-link/hooks.json) distinguishes
probe memory/platform values, portable implementations and M3/M4 obligations.
M3 must supply user mappings/loader/stacks, allocation/quota/failure policy,
thread ownership and FP/SIMD initialization/save/restore. M4 must supply real
monotonic clocks, wait/input/pacing, immutable resources and brokered presentation.
The Slint fixture clock advances synthetically; event-loop/proxy, clipboard,
native font/IME and accessibility services are unavailable. Default no-op or
unsupported methods receive no working-service credit. Neither rendering nor
callbacks belong in the current kernel or interrupt path.

Final-link closure cannot resolve Slint's native text/effect/rounded-clip/opacity/
blend gaps, standard-motion gaps, full redraw/copy costs or OS/API distribution
terms. Those remain comparison inputs. Guest execution, rendering, scheduling,
resource continuity, behavior replacement and performance require later gates.
