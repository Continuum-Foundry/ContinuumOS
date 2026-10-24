# P2-10 — Selected hosted layout, input, semantics and motion

The selected core is custom Rust, using the existing surface, document executor,
font pins and immutable comparison corpus. Slint remains frozen comparison material.
This work supplies hosted behavior; it does not implement a guest UI, native IME,
screen reader, file reload or the P2-12 performance gate.

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Test-P2-10.ps1
# Reuse an inspected, successful prerequisite recheck from this development session:
pwsh -NoProfile -File tools/Test-P2-10.ps1 -Prerequisites build/m2/P2-01/prerequisites-RUN
pwsh -NoProfile -File tools/Preview-Custom.ps1 -Selected
pwsh -NoProfile -File tools/Preview-Custom.ps1 -Selected -Replay
```

The acceptance command uses the existing development signer. It never installs a
certificate, edits App Control policy, changes a lockfile or fetches dependencies.
The ten Phase 0/signing/doctor prerequisite rows must all pass. Rust tests, the
separate behavior feature, portable checking, full P2-04 acceptance, the unchanged
150-capture/300-state corpus with 45 text and 63 integrated text captures, two
selected 30-configuration runs (including all 45 text cases and 63 integrated text
captures in selected paint mode) and native Windows replay are separate results.
The independent Python consumer checks hierarchy, values, focus, virtual rows,
resize geometry, authored grapheme boundaries and six dense rounded-edge oracles.
It never updates a golden or increases the corpus tolerances.

## Layout and invalidation

`layout::Plan` measures each node once, then arranges each once. Rules specify
fixed, fill or content sizes and parent-local anchored or flow placement.
Rows/columns share remaining main-axis space among fill children; overlays place
flow children at the same padded origin. Content measurement treats fill as zero
intrinsic demand, preventing circular percentage/content dependencies. There is
no iterative constraint solver. Scroll transforms move descendants while retaining
the viewport clip. Virtual lists realize visible rows plus two overscan rows.

The shell retains the authored tree separately from arranged bounds, avoiding
cumulative resize/scroll transforms. The reference plan preserves authored 640×360
geometry, stretches the editor, anchors the side panels and bounds the list height.
The native adapter supports explicit 100/150/200-percent transforms. Automatic
Windows DPI changes and OS window-management authority remain outside this gate.
Window geometry remains host shell/model state, never toolkit compositor authority.

Validation precedes publication: 1024 nodes, 64 children/node, depth 32, unique
logical/semantic IDs, finite coordinates bounded by 16384, known resources/actions,
bounded rules and at most 64 realized rows. Rejection leaves scene and document
untouched. Layout, paint and semantics have separate flags/counters. Color/opacity
publication causes paint; semantic labels/actions cause semantics plus paint;
geometry/topology/font metrics cause layout. Semantic snapshots are cached by model
revision and semantic invalidation. The raster path still redraws the full surface;
this is not P2-12 damage or layout/text-cache optimization.

## Input and text policies

- Pressed keys/buttons, capture and composition are view transients. Key usages,
  committed text and composition are distinct. Both modifier sides release safely;
  shortcut modifiers suppress committed text. Shift-Tab reverses navigation.
- Non-release events require current geometry revision. The selected dispatch
  cancels transients after rejection and permits sequence resynchronization.
  Key-up is idempotent even after target removal. Pointer-up across revisions
  cancels without activation. Moved/replaced/removed capture owners cancel capture,
  held keys/buttons and composition. A removed/nonfocusable focus owner falls back
  to editor, then the first remaining focusable control; no eligible control returns
  an explicit error without discarding document state.
- The 256-event queue cancels all transients on overflow and counts every discarded
  event. A bad queued event or explicit reset discards the remaining queue. Sequence
  and timestamp ordering remain checked after resynchronization. The diagnostic
  session retains the existing 4096-event/60-second timestamp and 256-action caps.
- One explicit modal scope traps keyboard/semantic focus. Escape clears transients,
  restores eligible prior focus, and the shell retires its subtree and semantics.
  Nested modals reject. A resize preserves retired subtrees rather than resurrecting
  dismissed controls. This is a hosted test dialog policy, not secure system focus.
- Visual left/right/up/down/home/end use shaped stops, a line/stop affinity and
  document-wide Unicode grapheme boundaries. A shaping cluster can split an Indic
  or emoji grapheme; those interior stops are excluded from navigation. Pointer and
  drag selection use the same shaped map and explicit physical-to-logical transform.
  Affinity is view state, not a native pointer or checkpoint ABI.
- Composition remains transient until an authorized commit. Escape/focus retirement
  cancels it; compatible movement with the same focused ID preserves it. Editing
  shortcut commands do not alter a live preedit. Commit/cancel and failure preserve
  accepted document edits. Native IME, clipboard and a full keymap are not supplied.
- The selected view admits at most 4096 UTF-8 bytes and 256 logical lines, rejecting
  unsupported controls/bidi overrides before insertion. Font/glyph/shape limits stay
  pinned. Emoji coverage still reports missing clusters/tofu; no font is invented.
  Extremely narrow wrapping can still reject honestly at the existing line/work
  limit while retaining the model. Allocator-wide OOM is not recoverable in general.

## Semantics and visual behavior

The hosted consumer receives stable IDs, explicit parent IDs, role/name/value,
actions, logical bounds, focus/checked/selected state, virtual position/count and
added/removed/focus/value/state/bounds notifications. Bounds are arranged logical
bounds; row bounds are intersected with their list viewport, including empty
overscan bounds. No native UI Automation or assistive-speech claim follows.
Virtual row IDs use the reserved `virtual:` namespace; authored IDs cannot collide.
Visible row activation and logical list navigation route through P2-03's executor.
Unrealized row activation rejects; parent navigation can realize an arbitrary
in-range logical row. Focus, text/search and settings actions use the same executor.

Selected painting adds geometric focus rings, selected-row bars and checked marks
as non-color cues. Rounded surfaces use 4×4 coverage and isolated ancestor clips;
flat vectors stay exact. Six 64×64 mathematical edge-oracle comparisons retain
the existing maximum channel error of 16. Legacy comparison mode and its golden
hashes remain unchanged. Clipped groups still copy buffers. Before selected paint,
a conservative 64 MiB temporary-storage bound covers nested full-surface groups,
mask storage and blur ping-pong; pressure rejects the unpublished frame. Surface,
font and process accounting remain separate and are not a 256 MiB guest result.

Opacity transitions last 120 ms, can be interrupted from their current visible
value and snap to their endpoint under reduced motion. A monotonic scheduler
retains at most 64 transitions, schedules 16.667 ms transition/drag, 500 ms caret
and optional 1 s clock deadlines, and skips elapsed intervals in bounded work.
Clock/drag deadline hooks are modeled signals, not new clock/window widgets.
The native adapter uses returned deadlines for caret/transition frames and stops
that timer when settled. Reduced motion keeps a steady caret and no decorative
movement. Full idle CPU, frame cost and retention measurements remain P2-12.

## P2-11 handoff

The logical owner, source layout, arranged scene, semantic identity and transients
are now explicit. P2-11 must supply the actual bounded file builder/parser,
coherent token plus structural layout publication, compatible focus/capture/
composition reconciliation and reader/resource lifetimes in the same native
window/process. P2-10's in-memory publication and native resize are not file reload.
Keep the exact unsaved document/history/checkpoint and shell state, retain all
rejections, and preserve the original kernel/ISO path separately.

P2-11 has now completed this hosted handoff; see the [resource reload runbook](resource-reload-runbook.md) and [local acceptance](../../../evidence/phase-2/P2-11/2026-10-23/README.md). P2-12 optimization and repeated stability remain pending.
