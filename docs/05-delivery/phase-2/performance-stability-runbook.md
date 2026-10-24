# P2-12 — Selected hosted damage, idle and reload stability

Status: [local acceptance passes](../../../evidence/phase-2/P2-12/2026-10-24/README.md). No guest measurements.

Run the complete optimized acceptance with the existing enrolled developer account:

```powershell
pwsh -NoProfile -File tools/Test-P2-12.ps1
```

The sandbox has a different certificate view. Reuse the existing signer in its
enrolled context; do not export the key, change enrollment or broaden host policy.
The command reruns the ten offline Phase 0/signing/tool prerequisites, Rust tests,
portable checking, the unchanged common/selected/reload regressions, native reload
and native idle, damage oracle, stability and ten sequential measurement processes.
Full logs, captures, binaries and raw JSON stay under ignored `build/` and `work/`.
The final input-hash guard rejects source changes during acceptance.

## Fixed experiment

[The protocol](../../../config/p2-12-performance.json) is declared before execution.
Version 3 preserves P2-01's optimized profile, six geometry/scale configurations, five
fresh-process repetitions, paired tracing, 5 s warm-up, 30 s active/idle windows
and nominal 100 ms process sampling on an independent observer thread. Actual
observation timestamps, skipped slots and coverage are retained. Each active block interleaves six workloads in 300 absolute
slots. Late slots are retained; no 60-Hz display cadence is implied. Repetition
zero is separate; host file caches are not flushed. The active interval may overrun when all 300 requests cannot fit in 30 s;
idle timers skip elapsed periods instead of replaying overdue observations.
The native adapter is a
separate single-window observation, with its worker and presenter charged separately.

Tracing adds exact executor/dispatch and group-composition clocks, allocation
event counters and framebuffer hashing. Both modes retain outer phase timers,
raw record storage and Rust live/peak allocation tracking. Paired differences
measure this incremental instrumentation, not the cost of all observation.
The layout interval measures selected dirty/layout bookkeeping; fixed authored
rectangles are arranged on setup/publication, not a general layout solve per frame.
Raster includes text/cache access, damage planning and snapshot maintenance;
composition clocks charge isolated group copies, masks and blending separately.
Backdrop filtering remains within raster. RAM submission copies only damage into
the measurement RGB consumer. Native BMP/WinForms copies are separately reported.

Idle contains 10 s static, 10 s scheduled 500 ms caret/1000 ms clock invalidations (no wall-clock text widget),
5 s hidden and 5 s minimized. Observer wakes are counted; OS scheduler wake counts
are unavailable. Accessibility has no periodic timer. Native preview separately
measures 10 s static, 10 s caret, 5 s hidden and 5 s minimized; it has no clock
widget, a 1 s watchdog and the explicitly recorded observer cost. Development
resource polling, when enabled, adds a 100 ms file poll. It is disabled in this
native idle baseline. Hidden/minimized native surfaces stop their animation timer.

## Damage and caches

The selected native preview and measurement harness share `retained::Frame`.
It compares owned previous values, computes old/new subtree bounds including
primitive overscan, merges intersecting physical rectangles and redraws from the
background in painter order. Translucent and clipped ancestors participate.
More than 32 contributions or over 60% surface coverage falls back to full redraw.
Any changed scene with a rich backdrop also falls back fully: this safely includes
filter support and changed backgrounds. No retained blur/shadow cache is claimed.
There is no independent shadow primitive. Group copies still use full-size scratch.

Text layout caching is selected-path-only and keyed by exact UTF-8 plus every
Config field, scoped to one immutable pinned-font engine. Its LRU has 128 entries
and 2 MiB including owned key/layout capacities and metadata. Glyph bitmaps retain
their 128-entry/1 MiB bound. The retained framebuffer is at most 16 MiB and holds
one previous tree/model/token snapshot. Colors and reload generation changes do
not create unbounded text keys. Eviction drops owned values; returned clones remain
valid. The uncached full renderer remains the pixel oracle, including shaping.
There are 300 cache key/eviction probes and 108 image comparisons with zero byte
tolerance at both resolutions and all scales. Existing authored quality/text
tolerances remain unchanged.

## Reload accounting

Each resolution at scale 1 runs five warm-up and 100 measured consecutive actual
file-builder/validator/publication cycles. A complete encoded checkpoint, including
both history stacks, is compared after every commit; provider identity stays fixed.
Every cycle holds an old generation reader through first RAM submission, then
releases it and waits 100 ms before counting resources. Two retained readers also
force a fourth-root headroom rejection; release permits retry.

Before the run, settled growth tolerances are fixed at 64 KiB requested live Rust
allocation, 4 MiB process private commit and 8 MiB working set relative to warm-up
cycle five. No old resource root may survive settlement. Glyph/layout cache limits
apply on every cycle. The independent review additionally requires an exact live
Rust allocation plateau after subtracting the four build-ID copies' decimal-width
change; a one-time baseline counter Value is reported separately. Observation rows stream to disk; the bounded diagnostic
trace is flushed, so evidence accumulation cannot be mistaken for UI retention.
Full failed development runs remain local and are not overwritten by acceptance.

Builder time is separate. Completed-build-to-RAM visibility includes real 100 ms
debounce, validation, staging and repaint/submission; it excludes compilation and
does not measure native scanout. Rust peak is reset before staging, including old
and new roots/pixels/model copies. Process peak counters include the full process
lifetime, including builders and font initialization. Generation reservations are
policy headroom, not physically allocated bytes. Known allocation categories
overlap total Rust/process counters and must not be added as disjoint quantities.

The Python reviewer rechecks raw counts, phases, hashes, checkpoint identity,
growth and cache limits and exports compact CSV samples. Its report must preserve
frame/input/reload target misses and bounded follow-ups. Six retained views,
shared fonts/text engine, logs, runtime, mapped code, stacks and allocator/OS
retention belong to the hosted experiment. Kernel, scheduler, isolation, drivers,
guest services and guest compositor costs are not measured. The 256 MiB guest
cold-idle target remains unchanged and unaccepted.


## Workload scope and follow-up gate

The fixed active scripts prepare typed actions and measure the real controller and
executor. They do not measure physical input-device latency. Movement/resize uses
window-model actions plus authored settings-node movement/width changes at each
fixed surface size; native resize is separately a selected-core conformance replay.
The sixth active family is direct coherent scene publication; the separate 100-cycle
runs measure the actual file builder, debounce, staging and resource publication.
The RAM consumer copies only validated regions; native BMP/WinForms still copies
full images. Small per-cell p99 samples are maxima, not reliable tail estimates.
All always-on allocation/sampling costs belong to the experiment process; trace
allocation events include observer work when it overlaps a frame.

A local pre-fix probe demonstrated a translucent root being published before
native opacity rejection. The final parser rejects it before generation creation;
unit cases cover alpha 0/128/254, and builder/API tests at both resolutions compare
completed files, active generation, scene revision, full checkpoints and frame hashes.
The preserved earlier run is superseded, never used as final measurement evidence.

P2-13 must recheck the final selected retained/cache path in its clean freestanding
link fixture. P2-07's original comparison closure and this story's portable Rust
check do not demonstrate a guest heap, runtime, framebuffer or presentation backend.


Phase clocks measure elapsed time, including scheduling delays; per-frame thread
CPU is not measured. Aggregate process CPU remains a separate 100-ns counter.
Evidence serialization after the measurement windows is excluded from UI timings.
The accepted run records host load/power/display context without isolating DVFS or
other host activity; noisy positive/negative tracing deltas are retained rather
than converted into a constant correction. Full distributions and bounded target
follow-ups are in the acceptance review; the guest targets remain open.
