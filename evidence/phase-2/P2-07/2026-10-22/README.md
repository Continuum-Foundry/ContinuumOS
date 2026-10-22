# P2-07 — Freestanding integration evidence accepted locally

**P2-07 is Done as a compile/final-link feasibility experiment.** This is Codex's
local implementation/self-review, not independent approval, guest execution or
selection of a toolkit. Source base is `6e53cf08e0d59873202037801487b104ef608090`,
with exact dirty compiled inputs in [compiled-inputs](compiled-inputs.json), on
`codex/p2-07-freestanding-ui`. Changes remain local and uncommitted.

The [review](review.json), [acceptance](acceptance.json),
[runbook](../../../../docs/05-delivery/phase-2/freestanding-ui-runbook.md) and
[complete hook inventory](../../../../experiments/ui-link/hooks.json) define the
accepted boundary. Full local runs are indexed in [local artifacts](local-artifacts.json).

- **Custom and Slint AOT both compile and finally link** for
  `x86_64-unknown-none` / Rust 1.99.1. Real scene validation/layout, typed text input,
  P2-04 shaping/raster, list realization, semantics and surface leases are retained.
  Slint uses the same extracted no-std AOT component, platform, layer builder and
  text Image adapter as the hosted harness. This is not an unused library import.
- Each ELF has independent Python + LLVM symbol/section/disassembly inspection:
  no unresolved symbols, remaining relocations, dynamic/TLS/host/runtime-unwinder
  linkage or WX loads. All **26 mutated ELF checks** reject. Both injected missing
  presentation symbols fail linking, and both injected `std` requests fail compilation.
- The owned **64 MiB arena** has three passing host tests covering alignment above
  base alignment, disjoint writes, exact capacity/exhaustion, zeroing and realloc
  success/failure. It intentionally never reclaims; guest runtime capacity and
  sustained use are not established. OOM/panic is terminal, never a success hook.
- The default target uses software-float helpers; the scan detects **zero emitted
  FP/SIMD instructions** in each ELF. This is not a formal ISA proof or permission
  to execute UI in the M1 kernel. Passive **28-byte `.eh_frame_hdr` + 68-byte
  `.eh_frame`** allocation-shim CFI is retained and identified, without personality,
  LSDA, exception tables or runtime unwinder. User FP/SIMD policy remains M3 work.
- The [input review](inputs.json) verifies **5,370 upstream source members** against
  exact registry archives. Custom has **32 active runtime/build packages**; Slint
  AOT has **59 normal runtime / 146 active build packages**. Runtime features exclude
  std/host backends; build/compiler inputs remain separate. No registry pin changed.
  Exact declared terms and notice hashes are retained; no distribution license is selected.
- **51 Rust unit tests pass**, including three arena tests; one pre-existing
  font-dependent custom test remains ignored in the default unit invocation.
  Both hosted candidate regressions pass unchanged goldens: **30 configurations,
  150 captures, 300 workflow states, 45 text cases and 63 integrated text captures
  each**, plus 28 custom / 29 Slint boundary cases. These are hosted results.
- All **ten prerequisite rechecks pass**, including fresh Phase 0 links/tests,
  signing, source paths with spaces and M1 doctor. The separately retained raw
  prerequisite manifest is hash-bound to this run. Existing M1 guest evidence is
  unchanged; this story launches no VM. The kernel runtime graph remains only
  kernel, limine and bitflags.

Final ELF identities (debug data and six embedded pinned fonts included):

| Candidate | ELF bytes | SHA-256 |
| --- | --- | --- |
| Custom | 25,476,384 | `a0c99bba76e4eccb50963baa60ce109d5cff5ed3d58ed296ae4a985219b57c67` |
| Slint AOT | 35,806,992 | `f8e04cdea967df0e55d06128f4df6ab2c58f2c8466c7e09f156f39e0a9e87e54` |

These are file sizes, not resident/idle usage or a guest performance result.
[Generated AOT LLVM IR](generated-ir.json) is hash-bound to target compilation;
full IR/disassembly/binaries stay local. [Development findings](development-findings.json)
preserve the signer/lock, linker-path, passive-CFI and collection-command failures.
The owner explicitly approved refreshing existing compiler signer lock enrollment
after automatic review required approval; no certificate or Windows policy changed.

`slint-interpreter = 1.18.1` with defaults disabled is separately **unavailable
offline**. It receives no compile/link credit. The tested Slint path is compiled
AOT; no native behavior replacement or interpreter preview is demonstrated.
Slint's native text/effect/clip/opacity/blend failures, adapter-derived semantics,
full redraw/copies, motion gaps and OS/API distribution questions remain unchanged.

M3/M4 must supply loader/mappings/heap/quota/failure policy, one-thread ownership,
user context policy, actual clocks/wait/input/resources and brokered presentation.
The probe supplies only deterministic platform values and RAM surfaces. No guest
UI boot/render, isolation, reload, efficiency or release guarantee follows.
P2-08 is next; all ADRs remain Proposed and the 256 MiB cold-idle target is unchanged.
