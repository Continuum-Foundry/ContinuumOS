# P2-03 — Hosted document/action model acceptance passes

**P2-03 is Done.** This is Codex's local implementation and self-review on
`codex/p2-03-document-state-actions`, based on `dacf54504f5250d92fc6dca7c4b8a86ec572a4eb`
plus recorded dirty input hashes. It is not independent approval. Changes are local
and uncommitted; no push, PR or remote execution occurred.

The [review](review.json), [acceptance](acceptance.json), [replay](replay-result.json)
and [reproduction runbook](../../../../docs/05-delivery/phase-2/document-actions-runbook.md)
record exact checks and limits. The [contract](../../../../docs/06-contracts/hosted-editor-model.md)
defines independent action/state versions and the explicit checkpoint encoding.

- A portable Rust `no_std`+`alloc` document owner preserves unsaved UTF-8 text,
  directional selection, bounded undo/redo, dirty state, search and scroll.
  Preferences and logical session/window state are separate; views only borrow.
- All **21 authored operations** run through each of four headless entries:
  button, shortcut, semantic and automation. Outcomes and final checkpoint bytes
  match. Views are destroyed/recreated and checkpoints roundtripped after every
  operation. Two independent commands produce identical structured result bytes.
- All **56 negative entry/case pairs** reject with unchanged before/after state
  hashes: stale revisions/epoch, rights/caller, version/action, fabricated/expired/
  wrong-scope contexts, invalid selection, quota, cancellation and deadline.
- All **36 Rust tests** pass: 17 editor-model and 19 xtask. Additional tests cover
  UTF-8/grapheme interiors, joining clusters, dirty baseline and undo branching,
  retry identity/eviction, context audience/revocation, authority/history/text
  bounds, revision exhaustion, every checkpoint truncation (also with adjusted
  length headers) and bounded byte mutations. No guest tests are run here.
- Independent PowerShell checks pass **28 action schema cases** and decode all
  **eight final checkpoints**, including historical text. An independently
  authored initial hex fixture checks Rust encoding AND decoding. The locked
  P2-02 corpus still passes. Library compilation for `x86_64-unknown-none` passes;
  this is not guest linking or execution.
- All ten offline prerequisite commands passed before extension: existing signing/
  compiler preflights, Phase 0 tooling/compile/link/path/dependency checks and M1
  doctor. After adding only cached `unicode-segmentation` 1.13.3, defaults off,
  the lock/terms were reviewed and the existing compiler-signing lock enrollment
  explicitly refreshed. No certificate or host policy changed. Notices and
  dependency inventory are updated. Formatting and repository audits pass.

Final logical checkpoint SHA-256:
`7e008385932d265f32791f33f31778623cbfac7334d57433808a0ba52ee399f2`.
It identifies synthetic logical bytes, not a guest build, boot or process identity.
The acceptance records the signed host executable, compiler, lock, harness and
fixture hashes. [Compiled inputs](compiled-inputs.json) distinguish tested code
from the later [reviewed input set](review-inputs.json). Full logs/checkpoints/
patches remain in the local build directories recorded in the acceptance.

[Development findings](development-findings.json) preserve the expected lock-signing
stop, a host-harness compile correction, and the failed undo-count fixture assertion.
The failed replay's result and selected verbatim output are retained. No passing
assertion was obtained by deleting history or changing implementation to lose state.

This accepts **R19's M2 interface/model slice**, logical preparation for R04/R15
and early R07/R14/R18 discipline. Hosted IDs/rights are not secure kernel handles.
Snapshots preserve logical state, while grants/high-water/dedup records remain
with the same live provider. There is no provider replacement, secure IPC, durable
file/save operation, actual toolkit input/semantics, rendered UI, shaping/bidi/font
conformance, live guest reload or measured performance. B09/B37 remain partial;
ADRs stay Proposed and the 256 MiB guest cold-idle target remains unmeasured.
**P2-04 fonts/text is next.**
