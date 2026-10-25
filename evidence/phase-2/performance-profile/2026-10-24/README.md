# Fixed single-view before/after and primary tail records

See the [implementation and acceptance review](../../performance-remediation/2026-10-24/README.md).
`profile-review.json` reaggregates every exact cell and pairs final pixels/state.
`runs.json` binds the signed baseline/candidate and local raw directories;
`baseline-compiled-inputs.json` records the pre-optimization diagnostic source hashes.
The preserved baseline executable/source copies are local archival prerequisites,
not artifacts shipped in this repository. Final source is clean-built by the runner.

Diagnostic CSVs retain elapsed raster/composition/submission, copied/visited bytes,
phase work and scratch/event counts. Baseline allocation-event zeros are unavailable
and excluded. Baseline has no new outer input clock. Each tail CSV retains all
frame/input-to-RAM/scratch/allocation/hash observations; richer work fields remain
in local raw JSONL and their means in the review. Empty values are unavailable,
not zero. Repetition identity is in the filename; geometry/effects/tracing are in
the filename and workload indices map to the pinned protocol. No samples are dropped.

Scratch-peak bookkeeping changed from recursive estimates to live plus cached
pixel/mask capacities. Its before/after delta is not an exact like-for-like memory
reduction. Structural copy/visit counts, bounded admission and the complete
requested-Rust/process peak measurements remain separate accounting records.

Both resolutions × three scales × two effects × two trace modes × six workloads
give 144 diagnostic cells, with eight warm-up and 16 measured frames each. Primary
1080p/scale-1 tails use five fresh processes with eight warm-up and 200 measured
frames per cell, totaling 1,000 per cell/24,000 measured frames. Baseline/final
diagnostics and all tails run sequentially after the complete regression/build work.
Process cold does not flush the OS file cache; the host is not CPU-isolated.
Warm-up frames execute but are excluded from retained latency distributions;
cold first-frame latency percentiles are not qualified by this pass.

Tracing adds operation clocks, event counts and framebuffer hashing. Frame clocks
exclude post-frame audit hashing/serialization; batch CPU/elapsed include them.
Paired trace deltas remain signed and are not subtracted as corrections. Text/fill
clocks overlap and must not be summed. CPU-probe deltas demonstrate empirical
process-clock granularity, not renderer-thread CPU precision. No guest/whole-system
or native display-completion acceptance follows from these host RAM measurements.
