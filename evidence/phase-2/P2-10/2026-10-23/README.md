# P2-10 — Selected hosted core passes local acceptance

**P2-10 is Done.** This is Codex local implementation/self-review, not independent
approval, a completed M2 gate, guest UI, release or performance acceptance.
Source base: `d110b7407c87d35f63e1893f8f15d00e7969b57b`, plus the recorded dirty
input hashes. No dependency, kernel/ISO source or Windows policy changed.

The final guarded run is `build/m2/P2-10/acceptance-1792795086303`.
All **ten prerequisite rows and eleven acceptance commands pass**. Tested source,
scripts, fixtures and configurations are hashed before and after the run;
the guard rejects changes. See [inputs](inputs.json),
[tested pins](tested-inputs.json), [commands](acceptance.json) and
[criterion review](review.json). The existing local signer was reused.

Optimized signed host artifact SHA-256:
`5cca9ae03aea55bc0084e26a08bf99414b6b619007c9337ce70da3b158a8c86f`.
It was built with Rust 1.99.1 / LLVM 23.1.1, `m2-comparison`, on the recorded
Windows/i7-1165G7 host. This is artifact identity, not a speed or guest-memory claim.

## Accepted behavior and evidence

- Bounded fixed/fill/content row/column/overlay layout, scroll transforms,
  virtualization, finite/depth/cycle rejection and atomic arrangement. Paint-only
  publication skips layout and cached semantic work.
- Revision-checked input, idempotent releases, moved/removed capture cancellation,
  focus fallback, modal retirement, queue overflow/reset and composition policies.
  Visual navigation and drag selection retain bidi affinity and grapheme boundaries.
- Semantic hierarchy, roles/names/values/actions, focus/checked/selected state,
  virtual positions/counts and change notifications. Keyboard and semantic workflows
  produce identical complete model checkpoints in all **30 configurations**.
- Interruptible 120 ms opacity transitions, reduced motion, bounded deadline hooks
  and native caret/transition frame scheduling. Full idle/performance is P2-12.
- All **150 unchanged reference captures / 300 workflow states**, **45 text cases**
  and **63 integrated text captures** pass. Each selected run separately paints all
  45/63 text cases in selected mode. Two selected runs match their result, matrix,
  workflow and visual-text hashes. All **21 authored text/scale navigation cases**
  pass, with explicit emoji/tofu coverage limits.
- Independent hierarchy, geometry and text assertions pass. Six dense **64×64
  sampling** edge oracles report maximum channel errors **5, 7, 5** at 100/150/200
  percent for both standalone and coincident ancestor curves, within the unchanged
  tolerance of **16**. Flat/exterior pixels remain exact.
- Native Windows replay submits **15 frames** in one process, including resize,
  fractional-scale pointer input, drag selection, modal keys and opacity commands.
  Accepted `Draft: notes\n` survives; final pressed keys/buttons/capture are empty.

The Rust checks pass **22 custom-ui + 17 editor-model + 3 text-core + 22 xtask**
tests, plus the explicitly invoked ignored paint-failure case and a separate
22-test behavior-feature run. Portable `x86_64-unknown-none` checking and the full
P2-04 Unicode/native-HarfBuzz/negative fixture gate pass. The latter is text
conformance evidence, not native IME or assistive integration.

## Retention and boundaries

[Development findings](development-findings.json) preserve the blocked signer view,
grapheme-stop failure and two edge-oracle failures. The final source and tolerances
pass; none of those failures was converted to a waiver. Full logs, PPMs, patch,
source inventory and generated binaries remain under ignored `build/` and `work/`.
This compact bundle retains actual manifests, checkpoint/semantic traces, selected
pixels, diagnostics and hashes; its checksum manifest covers only retained files.

The selected painter still redraws fully and copies isolated groups. It rejects
conservative temporary demand over 64 MiB before clearing the unpublished buffer;
this is not allocator-wide OOM recovery or process/guest accounting. There is no
guest execution, native screen reader, automatic native DPI qualification or file
reload. Original ISO behavior is untouched. The 256 MiB cold-idle target and frame
targets remain unaccepted. ADRs stay Proposed.

Next: [P2-11](../../../../docs/05-delivery/phase-2/11-hosted-resource-and-layout-reload.md),
using the [selected-core handoff](../../../../docs/05-delivery/phase-2/selected-ui-runbook.md).
No push, PR, remote build or publication occurred.
