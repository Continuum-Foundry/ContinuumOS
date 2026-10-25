# Hosted performance remediation — local acceptance

The owner-authorized bounded rendering/publication pass is implemented and passes
local regression. This is Codex self-review, not independent approval or guest
performance acceptance. The original [P2-13 baseline](../../P2-13/2026-10-24/README.md)
is unchanged. Use the [plan and reproduction command](../../../../docs/05-delivery/phase-2/performance-remediation-plan.md).

Final run: `build/m2/performance-remediation/acceptance-1792886282462867700`. Source base and the exact dirty
source hashes are in [acceptance](acceptance.json), [tested inputs](tested-inputs.json)
and [LF provenance](source-provenance.json). The clean signed hosted SHA-256 is
`ddad7af445a94777e492acdc427db193160400f88818eb8f4d1487e7193fc330`; the clean selected ELF SHA-256 is
`87b3bc723d2ea6a9e8f3f5785cdcc90919f2cdc4fbda33b94e63aef3133fdabe`. That ELF is linked/audited, never
executed. The host artifact is used by all final conformance, native and profile runs.

Opaque interiors use exact spans; fractional/curved edges retain scalar coverage.
Group snapshots/masks use conservative widget/subtree bounds and damage/clip,
with correct stride and physical origins. A per-paint pool caps retained pixel
capacities at 4 MiB/32 buffers inside the existing conservative 64 MiB buffer
admission estimate; it is dropped after paint. Global Rust/process counters charge
pool/vector metadata as well. No persistent filtered-backdrop cache was added.
Rich damage touching the multi-pass source support falls back fully; disjoint
damage and exact shaped caret rectangles can stay partial.

Successful resource publication transfers the owned validated retained frame.
The next full submission reuses it only when complete scene/model/preedit/caret/
motion/token/scale/dimension state agrees; intervening changes redraw and malformed
dimensions reject. Validation/rejection and reader/headroom policies remain.
Native presentation uses owned RGB bytes in memory; PPM export is explicit.
BMP writing and WinForms cloning remain measured adapter work.
Two separate final-artifact host sessions verify normal presentation creates no
PPM and explicit `export_capture: true` writes one matching the capture hash.

The final [focused review](../../performance-profile/2026-10-24/profile-review.json)
validates 144 paired cells at both resolutions/all scales, identical final pixel
and checkpoint hashes, and 1,152 trace-on per-frame pixel pairs. Small diagnostic
quantiles remain small-sample observations. The independent primary tail sample
contains five fresh processes × 200 frames per cell: 1,000 samples for each of
24 workload/effect/tracing cells (24,000 total). All richer/scale cells and trace
overhead remain visible in the [profile bundle](../../performance-profile/2026-10-24/README.md).

| 1080p scale 1, trace off | Workload | p50 ms | p95 ms | p99 ms | >16.667 ms | Input-to-RAM proxy p95 ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Rich | focus | 11.603 | 13.725 | 14.916 | 2/1000 | 13.741 |
| Rich | complex-script-type-search | 12.325 | 18.642 | 21.848 | 85/1000 | 18.689 |
| Rich | 10000-row-scroll | 3.158 | 5.053 | 5.545 | 0/1000 | 5.068 |
| Rich | move-resize-overlap | 0.562 | 1.096 | 1.225 | 0/1000 | 1.139 |
| Rich | effects-and-motion | 12.609 | 17.424 | 20.505 | 64/1000 | 17.437 |
| Rich | token-layout-publication | 0.529 | 0.801 | 0.906 | 0/1000 | 0.830 |
| Reduced | focus | 8.655 | 11.417 | 14.061 | 0/1000 | 11.436 |
| Reduced | complex-script-type-search | 8.744 | 10.625 | 13.441 | 1/1000 | 10.689 |
| Reduced | 10000-row-scroll | 3.147 | 3.876 | 4.500 | 0/1000 | 3.883 |
| Reduced | move-resize-overlap | 0.542 | 0.789 | 0.967 | 0/1000 | 0.813 |
| Reduced | effects-and-motion | 8.789 | 11.910 | 16.000 | 8/1000 | 11.928 |
| Reduced | token-layout-publication | 0.549 | 0.843 | 1.059 | 0/1000 | 0.863 |

Rich typing/search p95 (18.642 ms) and effects/motion p95 (17.424 ms) remain
above the frame period. The other ten primary trace-off cells are below it.
The bounded fixes pass regression; all performance targets are not yet met.

These are elapsed host times under the fixed scripts, including scheduling;
input is a controller-to-RAM proxy, not physical input or native display latency.
CPU remains batched process accounting. No renderer-thread CPU percentile or guest
target is accepted. All 256 MiB whole-guest, 8/16 ms CPU and 25 ms guest input
requirements remain unchanged.

| Actual file reloads | Measured cycles | Completed-build-to-RAM p95 | Staging requested-Rust peak |
| --- | ---: | ---: | ---: |
| 1280x720 | 100 | 129.527 ms | 55,189,028 B |
| 1920x1080 | 100 | 128.116 ms | 67,861,032 B |

The preserved original P2-13 reload p95 was 930.549 ms at 720p / 875.284 ms at
1080p. Final visibility includes debounce, validation, staging and RAM submission,
excludes building, and does not establish native display/guest visibility.
All 200 measured first frames reused validated pixels; complete checkpoints,
retired roots, growth/cache/headroom limits and continued editing pass.

The complete M1 regression passes 33 rows and 42 bounded offline Q35/TCG launches
because shared xtask source changed. [M1 result](m1-result.json), clean comparison,
inventory, launch arguments and selected serial records back the boot/fault outcomes;
full logs/ISOs remain local. M2 re-runs all 16 P2-13 rows, ten fresh prerequisites,
locked corpus/text/action/selected/native regressions, 108 exact damage images,
clean retained/cache linking and its negative probes. Frozen Slint feature checking
and selected dependency/terms review pass without a new dependency or rights claim.
One preexisting ignored unit pressure case remains disclosed; the required real
reload pressure/rejection cases execute. No required acceptance row is skipped.

The original paced/interleaved six-view experiment remains separate: 3,000 frames,
5,985 process observations and 338/3,000 elapsed frame-budget exceedances.
Its complete per-cell records are retained here. Do not equate it to the single-view
diagnostic or derive a universal speedup by pooling unlike cells. Host scheduling,
power and file-cache effects remain uncontrolled.

[Development findings](development-findings.json) retain the fractional-clamp and
caret-key failures, the intentional interrupted validation, baseline allocation
instrumentation limitation and noisy exploratory context. Six mutated raw-profile
records independently reject. Source/artifact guards pass and all final owned
helpers are reaped. Relevant docs/ADRs remain Proposed pending guest confirmation.
Full generated raw outputs, binaries and source copies stay ignored and local.
The retained CSVs preserve distributions and diagnostic work counters; raw file
hashes bind the larger local records. No remote transfer or PR was made.
