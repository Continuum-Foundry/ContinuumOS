# P2-05 — Bounded custom Rust candidate

The candidate uses the unchanged P2-02 corpus, P2-03 owner/actions and P2-04
fonts/text pipeline. It is a hosted experiment; Slint comparison, guest link
closure, toolkit selection, live reload and performance acceptance remain later
stories. No kernel/ISO source or host App Control policy changes are needed.

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Supply-TextInputs.ps1
pwsh -NoProfile -File tools/Test-P2-05.ps1
cargo xtask test --suite custom-ui
```

The acceptance harness runs all ten offline prerequisite checks, Rust tests,
the separately compiled `custom-ui/behavior-v2` feature, freestanding library
checking, P2-04 acceptance (including the independent native HarfBuzz oracle),
two optimized candidate runs and a bounded native Windows presentation replay.
It requires the existing development certificate's documented execution path.
Only first-party path dependencies were added to Cargo.lock. The owner explicitly
approved refreshing its enrollment with the existing signer in this session;
no new certificate or App Control policy was created.

For an interactive native preview after building the comparison profile:

```powershell
cargo build --locked --offline -p xtask --profile m2-comparison
pwsh -NoProfile -File tools/Preview-Custom.ps1
pwsh -NoProfile -File tools/Preview-Custom.ps1 -Replay
```

The preview is a small host-only WinForms adapter around the signed Rust xtask
executable. Rust owns the model, scene, focus/capture, text and RGBA pixels. The
adapter supplies physical key usages for the declared subset, committed text,
logical pointer coordinates and a native event loop. It displays an owned bitmap
after explicit RGBA→RGB→bottom-up BGR24 conversion. It does not render text with
Windows fonts. Each response has a bounded wait; shutdown/timeout reaps the child.
Native presentation is exercised at 1280×720/scale 1; all six physical-size/scale
combinations are captured before presentation by the Rust harness. Native IME,
UI Automation/screen-reader integration and native multi-DPI transitions are open.
The live preview has a bounded lifetime and no file/network command interface.

`crates/custom-ui` is `no_std` + `alloc`, with unsafe code forbidden. Its public
Rust values are internal builder types, not an ABI or untrusted data decoder.
The host reuses P2-02's whole-corpus/schema/lock validator before converting JSON
to typed nodes. Publication validates all nodes, resources, topology, tokens,
generation and next revision before replacing the old scene. Geometry is the
corpus's fixed absolute layout; rows/columns express containment/preorder, without
an implicit constraint solver. Semantic IDs remain independent of paint order.

The renderer supports rounded clips, glyph/placeholder images, premultiplied
encoded-sRGB source-over, isolated group opacity, the prescribed three-pass
backdrop blur and opaque reduced-effects fallback. Font rasterization is the
pinned P2-04 pipeline. Virtual list rows use a jump-ahead form of the exact locked
LCG, checked against every sequential row; no 10,000-row widget tree is allocated.
Realization stays at 15 rows for the authored viewport, with a hard cap of 64.
An oversized viewport rejects rather than silently exceeding the cap.

The shared provider owns document state. UI, shortcut and semantic activation
use its versioned executor. Keyboard editing supports committed text/newline,
grapheme deletion, logical left/right selection, select-all, undo/redo, tab focus,
list arrows, launcher and settings activation. Pointer editor hits use the same
shaped caret map as paint. Native text appends to search; composition start/update/
cancel/commit is tested as modeled input. Complete visual-arrow navigation,
clipboard, native keymap/IME and a general editor are not implemented.

The shared model still admits 64 KiB documents. This candidate rejects an edit
that would exceed the text renderer's 4096-byte view budget before mutating the
model. Input sessions/host diagnostic history are capped at 4096 events/commands;
pressed keys and retained action history are capped at 256. Queue overflow and
capture-owner removal cancel transients; owner death is a modeled reset injection,
not a demonstrated native service/process replacement.

Layout, paint and semantics have separate invalidation flags/counters. The first
candidate conservatively clears and redraws the full buffer after a visible
change; old/new/filter damage is retained and full redraw is reported explicitly.
It does not claim an optimized dirty-region renderer. Settled content requests
no frame in the deterministic idle check. The caret is static; no clock/caret
timer or decorative animation is installed. Standard motion token durations are
reported; this candidate applies all changes immediately and has no animated
transition implementation. That visual limitation stays visible for P2-08.

Each of the 30 configurations executes ten independently asserted workflow steps
and captures the five fixed checkpoints (150 images). A sixth settled frame
drains the final cancellation before the idle decision test. Every checkpoint
asserts state and the semantic tree separately from pixels. The harness additionally
paints all 45 P2-04 text cases and all seven P2-02 complex/long strings in editor,
search and label nodes at three scales (63 integrated captures). Deliberate emoji/
coverage misses retain P2-04's policy; long labels clip/wrap in the fixed geometry.

`tools/Review-Custom.py RUN OUTPUT` checks independent flat colors and exterior
clips across all 150 captures and losslessly converts selected PPMs to PNG.
Candidate-specific RGBA/semantic hashes are frozen only after local review in
`fixtures/m2/custom-v1/goldens.json`. A mismatch rejects; the command never rewrites
the golden file. These hashes are this candidate's baseline, not Slint's oracle.
The common geometry/paint tolerances remain unchanged for P2-08; analytic vectors
are exact and glyph/rounded edges use this candidate's declared hard-coverage
geometry. No multilingual/accessibility expert approval is implied.

Raw results, all PPMs, state/semantic traces and timing samples stay under ignored
`build/m2/P2-05/`. `result.json` excludes timing so two separate runs must match
exactly. `timings.json` records layout, raster and capture hashing/RGB-copy durations
with `Instant` (Windows QPC); disk writes are excluded. These are instrumentation
samples, not the five-repetition, 30-second workload gate of P2-08/P2-12.

Surface allocation counts/bytes, pixels touched, full damage area, realized rows,
shape work, glyph-cache payload/entries and temporary paint storage are explicit.
They count selected containers, not allocator-wide calls, process RSS/commit or
guest memory. Font bytes, lazily decoded faces, model snapshots, host image copies
and temporary text allocations require separate accounting. P2-04's 1 MiB bitmap
LRU and 128-entry cap remain; allocator OOM is not generally recoverable.

The adapter/hook inventory and exact dependency/features/source hashes are retained
with acceptance. P2-07 must supply a real allocator/OOM/panic path and FP policy,
link closure, input/clock/wait/presentation hooks and runtime evidence. P2-08 must
compare visual/interaction limitations, full redraw/copy costs and maintained
first-party code with Slint. This story does not select the toolkit or certify
the 256 MiB guest cold-idle target.
