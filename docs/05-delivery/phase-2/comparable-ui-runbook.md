# P2-08 — Comparable hosted quality, resource and reload experiments

This experiment compares the bounded P2-05/P2-06 implementations after P2-07's
accepted final links. It does not select a toolkit or execute a guest UI.

## Reproduce

Use the existing Windows development signer and workspace-local Rust 1.99.1.
No lockfile, enrollment, certificate or host policy change is required. Cargo
stays offline. Run from the repository root in the established local execution
context; the sandbox cannot access the private signing key or execute xorriso.

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Test-P2-08.ps1
# The script prints its absolute acceptance directory. Use that path below.
python tools/Review-P2Comparison.py <acceptance-directory> evidence/phase-2/P2-08/2026-10-22
```

An already completed, freshly checked ten-command prerequisite manifest can be
passed with `-PrerequisiteEvidence <path-to-prerequisites.json>`. The script checks
all ten outcomes and records its digest. `-SkipMeasurements` is a development
option and cannot satisfy acceptance; the independent reviewer rejects it.

The script builds `m2-comparison` (opt-level 3, debug 1, one codegen unit, LTO off)
twice, with `slint-candidate` then `slint-candidate,behavior-v2`. It signs and copies
the exact binaries into distinct ignored `build/cargo/signed/p2-08/v1|v2/` paths.
They share the same final source and exact Cargo lock. Compilers, AOT generation
and signing are outside frame/idle measurements; their elapsed build times and
file sizes remain separate in `acceptance.json`. These are warm-cache incremental
iteration costs, not clean-build estimates. The application runtime is included
in the measured process. No interpreter is available offline.
Both candidates run from the same combined harness binary. Unused code can remain
mapped, so its file size and process base are not minimal per-toolkit production
footprints. P2-07 retains separate target ELF identities; no candidate-specific
clean compiler-memory or build-time comparison is claimed here.

## Measurement protocol and limits

[config/p2-08-comparison.json](../../../config/p2-08-comparison.json) declares the
comparison schedule before execution. It implements P2-01's five repetitions,
5-second / 50-frame paced warm-up, 30-second active and 30-second idle windows, 100-ms process
sampling and paired instrumentation modes. Each window interleaves all six
geometries, six workload families and five requested visual variants. It does
not claim a separate 30 seconds for each cell, continuous 60-Hz presentation, or
an implemented clock/caret/native-drag workload. Those missing features are
explicit unsupported outcomes for both candidates. The P2-01 profile stays
unchanged. P2-12 owns sustained selected-core performance and stability acceptance.

There are 20 sequential fresh-process blocks, each with 300 active frames and
600 process samples. Geometry is `n % 6`, workload is `(n / 6) % 6`, and variant
is `(n / 36) % 5`; all 180 combinations occur in every block. No scheduled frame
is dropped. Absolute 100-ms pacing permits lateness/overrun, which are recorded;
the parent kills and reaps a block after 120 seconds without extending its timeout.
Candidate and trace order reverse on alternate repetitions. The first repetition
is reported separately from repetitions 1–4; every process receives warm-up and
the Windows file cache is not flushed. Small per-cell samples make p99 essentially
the largest observation, not a reliable tail estimate.

The common layout interval measures current dirty/layout bookkeeping; this is a
fixed-rectangle fixture, not an implemented general constraint solver. Raster
includes P2-04 shaping, bitmap cache, effects or declared fallback, and all actual
candidate renderer work. Slint includes per-frame component/image construction,
native RGB raster and its RGBA bridge. Submission commits a full surface lease and
copies RGBA to an RGB RAM consumer. This excludes file writes and native window/
display presentation. Input latency starts at the scripted action receipt and
ends at this RAM copy, using one process's `Instant` clock. It is neither guest
latency nor photon latency. `frame_ns > 16,666,667` is a budget miss observation,
not an assertion that frames were scheduled at 60 Hz.

Every frame submits full-surface damage. Trace-on also scans changed pixels
against the previous output of the same view, hashes the RGBA bytes and counts
successful Rust allocation/reallocation calls and requested bytes. These are
cumulative traffic, not live bytes; a realloc counts its new requested size.
The deltas cover the frame function, including its trace work; scripted action
dispatch and row/process-observer allocations outside that function are excluded.
Foreign/OS allocations are covered only by process counters. Both modes retain
phase clocks, raw row buffering, a previous-frame buffer and its copy; trace-off
retains the allocator's disabled branch. The paired delta measures incremental
scan/hash/counter overhead, not all possible observation overhead. No overhead
is silently subtracted. JSON serialization, phase samples, previous-buffer copying
and trace work are in `total_ns`, outside `frame_ns` where documented.
`touched` is custom paint coverage for the custom path and common output-copy
coverage for Slint. Slint-native overdraw is unavailable; these columns must not
be compared as equivalent renderer overdraw counters.
Reload/helper diagnostics use the same frame recorder with allocation counting
disabled: their raw zero counter deltas are not allocation measurements. Only
the trace-on measurement CSVs provide enabled allocation traffic.

Six views stay alive to avoid resolution allocation churn, sharing one text engine.
The process therefore includes all six common surfaces, six RGB consumer buffers,
six previous-frame audit buffers, model/font/tree data, bitmap caches, Slint state,
allocator/runtime retention and raw sample vectors. Per-frame component counters
separately identify common surface/RGB/audit/cache/scratch bytes; do not sum peaks
at different instants or subtract shared pages to invent a guest estimate.

`GetProcessMemoryInfo(PROCESS_MEMORY_COUNTERS_EX)` supplies working set, private
commit and process-lifetime peaks. `GetProcessTimes` supplies cumulative kernel +
user CPU in 100-ns units. The narrow unsafe Windows FFI uses the current-process
pseudo-handle and an initialized `repr(C)` buffer; it adds no portable/kernel ABI.
See Microsoft's [memory fields](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex)
and [CPU definitions](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes).
An idle block has no injected app events. Its 300 observer timer wakes and CPU
are included; actual OS scheduler wake counts are unavailable. Zero requested
renders is separate from zero CPU. Neither process memory nor executable/ELF size
certifies the 256 MiB guest cold-idle target. Host power, competing activity and
physical display completion are uncontrolled, limiting small between-run differences.
The 100-ns CPU units and nanosecond duration fields are reporting units, not
measurement-precision claims. An unchanged idle CPU counter can reflect counter
quantization; it does not prove the observer or process executed zero instructions.
Slint's existing platform clock advances deterministic milliseconds when creating
windows; it is distinct from the QPC duration clock. It does not supply real
clock/caret/animation timer behavior during the idle experiment. That behavior
remains an unsupported workload, rather than receiving zero-wakeup credit.

## Resource and compiled behavior cases

`ui-reload` keeps one controller/provider alive for 100 alternating token/layout
publications. Each validates before publication, renders changed pixels, and
checks exact unchanged checkpoint bytes: text/history/revisions, directed
selection, focus, search, scroll, preferences and window geometry. Every record
includes the owner PID and increasing host monotonic sample. Token-only and
layout-only edits are independent. This is a direct scene-publication experiment,
not P2-11's file watcher, parser/transaction or native-window continuity gate.

Invalid token/asset publication rejects. A separate composition/capture fixture
calls the actual helper launch gate, which rejects before starting a child. Removing
its focused editor cancels transient input and moves focus to search. It does not
restore a stale checkpoint to conceal that intentional state change.

For compiled behavior, two differently built bounded helper processes restore
the same owner snapshot, render it, run a text-entry event and render the changed
output. Version 1 inserts `Continuum`; version 2 inserts `CONTINUUM`. The parent
retains authoritative state and validates identities/restored checkpoint hashes.
Helper results remain proposals; publishing returned edits/routing ongoing input
to a selected helper is unsupported here. A child restart is recorded as such and
receives no L0/L1 or guest L2 credit. No native code is unloaded.

The compiled-helper experiment records elapsed build/start/restore time and exact
binary/checkpoint/output identities. Combined simultaneous owner+helper+compiler
CPU/commit peaks are not measured. The separate active measurement's reload
workload covers resource-generation publication, not a production behavior
transaction. Interpreter overhead remains unavailable, rather than zero.

While the owner remains alive, a deliberate Rust compile failure, child startup
failure, invalid restore, and 100-ms forced timeout all preserve its exact snapshot.
Children are waited/reaped. A malformed returned checkpoint is rejected before
owner mutation. Builds have a 900-second parent limit; normal helpers have a
10-second limit. The separately bounded regression commands retain their own
300-second limit. These limits are best-effort Windows host process deadlines.

## Quality review and compact evidence

Both optimized binaries replay the full candidate matrices with unchanged goldens:
30 configurations, 150 captures, 300 workflow states, 45 text rasters and 63
integrated text cases each. Boundary checks remain candidate-specific. Run
`Review-P2Quality.py <custom-run> <slint-run> <bundle>` using the paths printed in
the two regression stdout logs. It independently checks state/semantic equivalence,
computes image differences without a new tolerance, and creates a contact sheet.
Full-resolution images must be inspected separately where detail matters.

`Review-P2Comparison.py` validates schedule/counters/identities, exports all raw
frame/process samples as compact CSV, preserves first-process and warmed
distributions, all 180 cells per candidate/mode, reload observations and artifact
hashes. Full pretty JSON, binaries, logs and failed development runs remain local.
Freeze only the reviewed compact bundle; include exact dirty compiled-input
hashes, source base and a checksum manifest. Run docs, governance, contracts and
index-based repository hygiene before completion. Recommendation belongs to
P2-08; provisional commitment and distribution suitability remain P2-09.
