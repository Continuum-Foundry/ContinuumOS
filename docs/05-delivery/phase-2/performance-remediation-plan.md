# Hosted UI performance remediation plan

Date: 2026-10-24. Status: Bounded fixes implemented; regression passes, remaining hosted/guest target misses recorded.
Scope: the owner's request to assess and plan fixes after P2-13. Existing
P2-13 evidence remains immutable. This plan does not create later P-level stories.

## Recommendation and baseline

Run a bounded software-renderer improvement pass before guest UI integration or
broad widget investment. M3's independent isolation/runtime foundations need not
wait for complete guest performance qualification. P2-13 accepted the declared
hosted correctness, continuity and measurement gate; efficiency targets remain
open. The current costs are large enough to justify work now.

The [accepted baseline](../../../evidence/phase-2/P2-13/2026-10-24/README.md)
contains 2,962 elapsed-budget exceedances in 3,000 frames. At 1080p, scale 1,
trace-off p95 frame times range from 453.156 to 685.983 ms across the six named
workloads. These interleaved cells contain only 40–45 observations each and mix
rich/reduced effects; their p99 values do not establish reliable tails.
Completed-build-to-RAM p95 is 930.549 ms at 720p and 875.284 ms at 1080p.
Cross-resolution differences do not establish causality on this noisy host.

Correlated sums from the 250 trace-on 1080p/scale-1 CSV rows put approximately
98.6% of measured frame elapsed time inside raster plus composition, versus
1.4% in RAM submission. These are elapsed phase shares, including scheduling
effects, and do not identify individual CPU hotspots. Input/state/layout
bookkeeping is small in this bounded authored-scene workload; it does not measure
the cost of a general desktop layout engine.

Code inspection identifies concrete optimization candidates:

- [Painting](../../../crates/custom-ui/src/paint.rs) copies the full framebuffer
  for isolated opacity/rounded-clip groups, allocates full-surface masks and scans
  the full framebuffer to composite each group. Coverage and blending still run
  per pixel through many opaque interiors.
- [Retained drawing](../../../crates/custom-ui/src/retained.rs) conservatively
  falls back to full redraw whenever a changed scene contains the rich backdrop.
  Its fallback is correct; a smaller repaint needs a proven dependency region.
- [Reload staging](../../../tools/xtask/src/reload.rs) renders validation pixels,
  hashes them, then drops them. The selected host/measurement path draws again
  for first presentation. Staging also uses the uncached layout path.
- [Native presentation](../../../tools/xtask/src/custom.rs) writes a PPM, reads
  it back, converts/writes a BMP and lets WinForms clone it. This is a separate
  adapter cost; removing it cannot explain away the RAM benchmark's raster costs.

These source findings motivate experiments. No speedup, target pass or guest
memory headroom is established by them.

## Ordered implementation slices

| Order | Work | Concrete result and acceptance |
| --- | --- | --- |
| 0 | Establish a useful focused profile | Pin a new diagnostic protocol before execution: exact binary/source, single active view, fixed scene/input/effects, both resolutions and all scales. Preserve the original six-view benchmark separately. Measure fill/coverage, text/cache, group copy/mask/blend, backdrop filtering, snapshot maintenance, staging and submission separately; count copied/visited pixels and allocations. Validate instrumentation overhead and CPU-clock precision. |
| 1 | PERF-12-01: opaque spans and bounded groups | Add exact fast paths for fully opaque interiors and fully covered spans. Restrict group snapshots, masks and blending to conservative physical bounds intersected with damage/clip, with correct stride and origin. Reuse bounded scratch storage. Preserve the existing coverage/blend rounding and painter order. Confirm the predicted reduction in visited/copied bytes and measured cost. |
| 2 | PERF-12-02: present the validated staged frame | Carry an owned validated frame and its render-state identity through successful publication; adopt it for first presentation instead of discarding/repainting it. Enable the same bounded text-layout cache during staging. Reuse only when generation, scene/model/composition/caret/motion state, dimensions and scale still match; otherwise invalidate and redraw. Admit all required frame/resources before activation and charge old-plus-new peaks. |
| 3 | PERF-12-03: timer damage and backdrop dependencies | Identify explicit old/new caret and changed-node regions. Derive backdrop source/filter influence from the actual blur passes, clamping and scale. Reuse a bounded filtered backdrop only while its complete lower-layer dependencies match. Keep full redraw for uncertain dependencies. Verify typing, scrolling, moved/removed occluders, transitions and background changes beneath the filter. |
| 4 | Native adapter follow-up, after core cost is understood | Remove the PPM write/read round trip and unnecessary conversion/clone work from normal native presentation. Keep capture export as an explicit diagnostic option and retain consumer ownership until release. Measure worker and presenter separately. Do this only after profiling demonstrates remaining adapter cost warrants the change. |
| 5 | Re-run the final gates | Compare paired before/after measurements on the same host/input set; run complete conformance, native continuity, reload/growth, clean selected linking and aggregate acceptance against the final source. Retain a distinct compact evidence bundle, including misses and failed attempts. |

Start with one optimization at a time. Use an identical deterministic replay
before/after each slice, then the relevant regression suite. Run the full expensive
acceptance after the combined candidate stabilizes or a failure justifies it.
Keep rich/reduced effects separate in diagnostic reports, with cold/warm and
individual workload results. Do not aggregate unlike cells into a speedup claim.

For stage 0, host CPU timing must first demonstrate sufficient precision for
sub-frame claims. Existing process CPU counters and per-frame QPC elapsed times
cannot supply renderer-thread CPU percentiles by themselves. If the available
CPU clock is too coarse, retain batched CPU and elapsed distributions separately
and leave CPU percentile compliance open. Do not convert variable-frequency
cycle counts to milliseconds using an assumed clock rate.

Any new cache must have a byte/entry cap, exact invalidation key, eviction and
staging-overlap policy pinned before its experiment. Charge retained capacities,
masks, scratch, glyph/layout caches, old/new frames and native copies to measured
totals. Reuse the existing admission budgets where possible. Overflow uses a
correct bounded fallback or explicit admission failure; it never silently drops
effects or consumes unbounded memory. Cache expansion requires fresh memory review.

## Correctness and measurement gates

Each optimization must preserve the locked 30-configuration/150-capture corpus,
300 workflow states, 45 text cases and 63 integrated text captures. Retain exact
incremental/full pixel parity for damage cases and unchanged authored edge/text
tolerances. Add adversarial cases for the changed mechanism, especially offscreen
group origins, nested translucent/rounded groups, stride padding, fractional
scales, filter boundaries, scratch exhaustion and resize/reload invalidation.

Staged-frame reuse additionally needs stale-state and paint-failure tests proving
the old generation remains editable, frame/semantics stay coherent and reader
leases remain valid. Timer/backdrop work must preserve static/hidden/minimized
zero-frame behavior, explicit timer costs and required animation opportunities.
At least 100 real file reloads at each resolution must retain complete checkpoints,
retire old roots and pass the existing fixed growth/cache/headroom limits.

The [requirements](../../00-overview/requirements.md#performance-targets-and-accounting)
remain unchanged: 1080p/60 render/composition CPU p95 below 8 ms and p99 below
16 ms, guest input-to-present submission p95 below 25 ms, resource visibility
below 500 ms, and the 256 MiB whole-guest cold-idle target. Hosted measurements
remain a precursor to those guest gates.

For the hosted follow-up, require a credible path to the 16.667 ms frame period
on the named ordinary workloads and completed-build-to-RAM p95 below 500 ms.
Report every workload/effect/scale cell and all misses. Obtain at least 1,000
measured observations for each primary 1080p/scale-1 ordinary workload when making
tail claims, spread over fresh-process repetitions; keep richer cases and other
scales visible. Precisely label any hosted CPU/input proxies and do not describe
their success as guest compliance. The full existing paced/interleaved protocol
also remains part of the final regression record.

## Stop and reconsider

Review after slices 1 and 2. If the measured profile still shows no credible route
to the frame/reload budgets, reopen ADR-0011's renderer/toolkit completion review
before expanding the UI. Use the remaining measured hotspot to choose the next
bounded experiment; do not assume a GPU port will rescue the design. An observed
improvement that still misses targets is recorded as an improvement with open
targets, not completion.

Guest runtime, presentation and whole-system accounting are qualified at M3/M4.
The existing Slint fidelity/lifetime/terms gaps still apply to any renewed
comparison. No new toolkit, font, license or GPU dependency is selected by this
plan. Preserve P2-13 as the original accepted baseline and link subsequent results
from the handoff instead of rewriting its frozen evidence.

## Reproduction and delivery

Use the existing enrolled local signer and offline locked inputs. Begin
implementation with fresh Phase 0 prerequisites; do not alter host policy or
enrollment implicitly. Full runs belong under ignored `build/m2/` or `work/`.
Final conformance uses the exact clean-built executable, native replay records
its hash, and the selected retained/cache path must still close its clean
freestanding link. Use `tools/Test-P2-13.ps1` for the final aggregate gate,
updating the diagnostic/acceptance protocol explicitly if new observations are
added. Retain new results separately under `evidence/phase-2/` within the existing
4 MiB / 200-file / 256 KiB-per-file policy.

## Execution record

The owner authorized implementation. The candidate now has exact opaque-span
fills, bounded group snapshots/masks and ephemeral scratch reuse, validated-frame
publication, conservative backdrop dependency damage and shaped caret damage.
The filtered-backdrop cache proposed in slice 3 was unnecessary for this pass;
uncertain or intersecting source damage still redraws fully. The native adapter
passes RGB bytes directly in memory and exports PPM only when explicitly requested;
BMP transport and the WinForms ownership clone remain charged.

The focused protocol is `config/ui-performance-remediation.json`. Six geometry/
scale configurations, rich/reduced effects, tracing on/off and six workloads
produce 144 diagnostic cells. Primary 1080p/scale-1 tails use five fresh processes
with 200 samples per cell each. No host cache flush or CPU isolation is implied.
The process-clock precision probe is empirical; batched CPU remains separate.
The pre-optimization diagnostic accidentally left allocation-event counting off;
its zero event counts are unavailable, not a zero-allocation result. Final paired
tracing overhead is reported without subtracting an instrumentation correction.

Reproduce the full bounded local regression with the preserved signed baseline:

```powershell
pwsh -NoProfile -File tools/Test-UiPerformanceRemediation.ps1 -Baseline build/cargo/perf-remediation-baseline/xtask.exe
```

The baseline executable/source hashes are archival prerequisites retained locally;
the final executable is rebuilt cleanly by the runner. M1's complete regression
runs because shared xtask inputs changed. The final aggregate hosted gate follows,
then sequential single-view before/after diagnostics and primary tails. The
separate Python consumer reaggregates every fixed cell and validates pixel/state
identity. This execution record grants no guest performance acceptance.


## Final measured outcome

[The distinct acceptance review](../../../evidence/phase-2/performance-remediation/2026-10-24/README.md) and
[focused diagnostic/tail bundle](../../../evidence/phase-2/performance-profile/2026-10-24/README.md)
retain all outcomes, failed attempts and limitations. Primary 1080p/scale-1 trace-off elapsed frame p95 ranges from 0.789 to 18.642 ms across the twelve workload/effect cells (1,000 samples each). Real file-reload completed-build-to-RAM p95 is 129.527 ms at 720p / 128.116 ms at 1080p. Rich typing/search p95 (18.642 ms) and effects/motion p95 (17.424 ms) still exceed the 16.667 ms period; ten other primary trace-off cells are below it. All misses remain recorded; elapsed/RAM results do not accept renderer-thread CPU, guest input or whole-guest memory targets.

The complete M1 gate passes 33 rows/42 offline launches; all sixteen hosted
P2-13 rows pass against a fresh clean host/selected ELF build. Paired diagnostic
pixels/checkpoints and all 108 incremental/full images match exactly; 200 measured
file reloads reuse validated first frames and preserve state/growth/reader limits.
The original P2-13 evidence remains unchanged. This pass establishes a practical
hosted improvement path; M3/M4 must qualify guest execution, presentation and
whole-system accounting. No GPU, toolkit, dependency or guest-heap decision is added.


## Remaining rich-effect target work

This pass does not close PERF-12-01–03 as universal performance compliance. The
two primary rich-effect p95 misses remain open. In the traced typing/search tail,
mean backdrop work is 3.060 ms, group copying 1.576 ms and mask/blend 1.002 ms;
these are elapsed observations, with overlapping clocks kept separate. Each
full rich frame still copies 7,076,800 group bytes and initializes 1,769,200 mask
pixels while visiting only 28,384 blend pixels. The paired rich-focus diagnostic
previously copied 33,177,600 bytes and visited 8,294,400 blend pixels.

The next bounded candidate is storing only boundary spans for provably opaque,
full-alpha clipped groups, while retaining complete snapshots for translucent
groups. Charge row/offset metadata and scratch capacities before admission;
preserve conservative bounds, ancestor masks and exact fractional/stride behavior.
Re-run the same pixel oracle and 1,000-sample rich cells before claiming a target
pass. This is a proposed follow-up, not implemented or measured by the current
result. It does not require a persistent blur cache or a toolkit/GPU switch.
Cold first-frame latency and renderer-thread CPU percentiles remain unqualified;
the current focused distributions follow eight warm-up frames.
