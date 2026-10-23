# P2-08 measured host results

All rows pool the five fixed repetitions with their declared unequal cell weights. The JSON distributions also separate repetition 0 from repetitions 1–4. These are RAM submission costs on the named Windows laptop, not guest or physical display latency.

| Candidate / trace | Frames | Frame p50 / p95 / p99 (ms) | Missed 16.667-ms budget |
| --- | --- | --- | --- |
| custom / off | 1500 | 68.030 / 159.358 / 203.213 | 1500 |
| custom / on | 1500 | 55.056 / 125.112 / 156.887 | 1491 |
| slint / off | 1500 | 18.873 / 41.029 / 56.694 | 876 |
| slint / on | 1500 | 18.631 / 32.578 / 42.159 | 875 |

Trace-on frames being faster in this aggregate does **not** imply negative overhead. Sequential laptop power/load/cache variation dominates those between-run wall-clock deltas. The [paired deltas](paired-overhead.json) retain negative values without subtraction. Within-frame trace intervals explicitly include scan/hash/counter work and the common previous-buffer copy. All phase, input-to-RAM, raw process and per-cell samples are retained; no mean-only ranking is used.

| Candidate | Observed peak working set (MiB) | Observed peak private commit (MiB) | Active private-commit growth (MiB) | 30-s idle CPU counter delta (ms) |
| --- | --- | --- | --- | --- |
| custom | 169.215–169.938 | 166.496–167.105 | 18.254–19.574 | 0.000–0.000 |
| slint | 1133.191–1134.363 | 1132.676–1134.441 | 871.039–873.109 | 0.000–171.875 |

Custom settles near 159 MiB of private commit, with 18–20 MiB of observed active growth and unchanged idle commit. That growth was not attributed to individual subsystems, and this finite run does not prove long-run leak freedom. Slint settles near 1,127–1,128 MiB after the same 350 frames, having added about 871–873 MiB during active sampling. Its shown-component lifetime error is documented in the [comparison report](comparison-report.md). Idle commit is unchanged for custom and increases about 0.26 MiB for Slint observer samples. Both request zero renders during all eventless idle windows. There are 300 observer wakes per block; OS scheduler wake counts are unavailable. Zero reported CPU increase is a quantized counter outcome, not proof of zero instructions.

Every block submits 100% full-surface damage. The trace-on changed-pixel totals are about 12.35% of submitted pixels for custom and 11.87% for Slint; these do not establish a valid smaller damage region or an equivalent rich effect. Initial frames compare against the zero-initialized audit buffer.

| Named retained buffers, six views | Bytes |
| --- | --- |
| Common RGBA surfaces | 35,942,400 |
| RGB RAM consumer copies | 26,956,800 |
| Previous-frame audit copies | 35,942,400 |

The 94.26 MiB of named buffer capacity is a subset of process commit. Fonts, models, caches, allocator/runtime and retained toolkit data remain in process totals. Component counters are not a reconstructed live-heap partition.

| Trace-on traffic over 300 frames, identical in each repetition | Custom | Slint |
| --- | --- | --- |
| Successful Rust allocation/reallocation calls | 736,163 | 1,255,235 |
| Requested bytes (traffic, not live bytes) | 1,174,924,404 | 3,254,590,840 |
| Maximum shared glyph-cache payload | 28,156 bytes | 28,156 bytes |
| Largest per-frame reported scratch counter | 9,786,368 bytes | 10,961,920 bytes |

Allocation deltas exclude scripted actions and sample-buffer allocation outside the frame recorder; foreign/OS allocations are not counted by the Rust wrapper. Process counters include their committed effects. `touched` means custom paint coverage versus Slint output-copy coverage and cannot compare native overdraw.

Warm-up used exactly 50 frames in every block; actual warm-up duration was 5.000–5.297 seconds. Parent command durations also include startup/font/setup and post-measurement artifact work, which can be delayed independently of timed frames. The largest active-window overrun was 4.408 seconds. All 300 scheduled samples were retained; no frame was dropped and no 120-second process deadline was extended. The slowest-block result is retained, rather than hidden by a longer timeout.

| Direct resource-publication experiment | Custom pause p50 / p95 / p99 (ms) | Slint pause p50 / p95 / p99 (ms) |
| --- | --- | --- |
| 100 alternating token/layout cycles per candidate | 28.126 / 30.709 / 31.883 | 12.669 / 19.332 / 21.023 |

These pauses include direct validation/publication, rendering, tracing and continuity assertions. They are not file-watcher or guest reload latency. Each candidate has 50 token-only and 50 layout-only changes, with exact unchanged owner checkpoint bytes and increasing owner monotonic samples. The helper JSON separately records binary identities, restart/start/restore durations, exact restored state and expected failure cleanup. Combined owner/helper/compiler memory peaks and a live behavior route are unsupported.

Observed render costs fail the desired frame budget frequently for both paths. Even reduced-effects comparisons remain expensive: at scale 1.0, pooling workloads and repetitions, HD reduced-effects p50 is 32.90 ms custom / 10.00 ms Slint; full-HD is 40.65 ms / 14.22 ms. Standard rich-request p50 is 42.79 / 12.70 ms at HD and 53.03 / 18.45 ms at full-HD, but Slint paints its unsupported opaque fallback in those cases. This is a cost/fidelity tradeoff, not equal visual work.

See [all 720 candidate/mode/cell distributions](cells.csv), [run distributions and peaks](runs.json), [accounting](accounting.json) and the two candidate distribution JSON files for exact values and sample counts. No confidence interval, guest projection, photon latency or completed R09 claim follows.
