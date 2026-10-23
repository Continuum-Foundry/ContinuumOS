# P2-08 — Comparison and recommendation for P2-09

This report compares the existing bounded candidates. It does not select a
production toolkit, accept the M2 milestone, or certify guest performance.
Review: Codex local implementation/self-review, 2026-10-22.

The [measurement summary](measurement-summary.md) reports all frame distributions,
deadline misses, process peaks/growth, allocation traffic, damage accounting and
resource-publication pauses. [Raw run summaries](runs.json), [cell distributions](cells.csv)
and all 20 frame/process CSV pairs preserve the full comparison samples.

## Common experiment and interpretation

The exact [protocol](../../../../config/p2-08-comparison.json) uses Rust 1.99.1's
optimized `m2-comparison` profile, the same locked fonts/scenes/actions, 50 paced
warm-up frames over five seconds, 300 active samples over at least 30 seconds,
and a 30-second eventless idle window. Five repetitions and paired incremental
instrumentation modes cover both candidates. All six resolution/scale profiles,
six workload families and five requested variants occur in every block. Order
reverses on alternate repetitions. The [runbook](../../../../docs/05-delivery/phase-2/comparable-ui-runbook.md)
defines timing boundaries, process counters, unsupported workloads, memory
attribution, helper failure policy and reproducible commands.

Results describe an Intel i7-1165G7 Windows host and a RAM RGB consumer. This
laptop's power/frequency state and competing host activity are uncontrolled;
small differences and outlying samples are not causal toolkit benchmarks.
The same absolute schedule and all raw samples are retained. Six live views,
RGB consumer/audit copies, shared fonts and runtime retention all count in process
memory. Cold process repetition 0 is separate from repetitions 1–4; OS file
caches are not cleared. The 5–10 samples in an individual cell do not estimate
a stable p99. Full damage, per-frame construction and known missing features
remain visible. None of these figures establishes the 256 MiB guest idle target.

## Every comparison axis

| Axis | Custom Rust | Slint 1.18.1 AOT + shared text adapter | Evidence / conclusion |
| --- | --- | --- | --- |
| Integration | P2-07 actual scene/text/raster final link, 32 active runtime/build packages, bounded probe arena | P2-07 actual AOT scene/text/raster final link, 59 normal runtime / 146 build packages; interpreter unavailable offline | [P2-07 review](../../P2-07/2026-10-22/review.json), [hooks](../../../../experiments/ui-link/hooks.json). Both remain dependent on M3/M4 runtime services, user context policy and sustained allocation. No new target or guest execution claim |
| Visual quality | Rich blur/tint, isolated opacity and rounded ancestor clip work in the fixed corpus; hard edge coverage and unfinished motion/layout remain | Faster primitive raster in some cases; rich blur, rounded ancestor clip, isolated launcher opacity and exact fractional blend remain unsupported/failing | [Visual review](quality-review.md), [all image differences](pixel-differences.csv), [P2-06 feature outcomes](../../P2-06/2026-10-21/feature-outcomes.json). An opaque fallback does not satisfy rich-mode fidelity |
| Text/accessibility | P2-04 shaping/raster, shared keyboard/action model and semantic tree; emoji/native IME/native consumer limits | Same shared text Image adapter, controller and semantic model; native Slint text setup/export unavailable | [Fresh functional/text observations](quality.json). 300 matching workflow states and 150 matching semantic captures are independent from raster similarity; 12 missing-cluster observations remain explicit |
| Reload | 100 token/layout publications keep exact owner checkpoint; compiled adapter variants restore it in different helpers | Same experiment through the AOT renderer; private components are reconstructed, not retained native widget identities | Per-candidate reload CSV/JSON. Direct scene publication is narrower than P2-11. Helpers are restarts and proposals, not L0/L1 continuity, an active behavior transaction, native code unloading or guest L2 |
| Efficiency | Full redraw/copies; substantial CPU raster and effect cost; per-frame Rust allocations and process costs retained | Full redraw/component/text Image construction plus RGB/RGBA copies; shown-component lifetime retention is a serious current adapter defect | Per-candidate distributions, every cell and raw frame/process CSVs. Neither a mean FPS nor the absence of guest code proves efficiency. Rich/fallback results must not be equated |
| Authoring | 1,906 lines in custom-ui include shared scene/input/surface/text-paint facilities also used by Slint | Adds 489 portable AOT/platform/bridge lines, plus its host-specific probes; depends on the shared custom facilities rather than replacing all of them | [Authoring inputs](authoring-inputs.json). 2,092 model/text lines and common orchestration remain shared. Code volume is not engineering-hours or widget breadth. Exact two-variant warm incremental build times are separate; per-toolkit clean-build/tooling productivity comparison is unavailable |
| Evolution | CPU fallback under our control; damage, persistent assets, actual layout, animation and composited effects need bounded completion | Upstream CPU primitives provide a possible optimization direction; current bridge must address component lifetime and unsupported fidelity | P2-07 hook inventory and current raster source. GPU paths/API stability, guest scheduling and sustained state replacement are unmeasured for both. Export logical state through versioned model schemas, not toolkit/native layouts |
| Distribution | Original first-party rights plus exact upstream font/text/raster/Unicode notices remain; no blanket distribution clearance | Exact GPL-3.0-only / Slint royalty-free / Slint software alternatives are inventoried; OS-internal and potential public SDK/API exposure remain unresolved | [Input terms/notice identities](../../P2-07/2026-10-22/inputs.json), [dependency inventory](../../../../legal/dependency-inventory.json), [notices](../../../../THIRD_PARTY_NOTICES.md). No license selected or vendor contacted; unresolved Slint suitability blocks its commitment at P2-09 |

P2-05/P2-06 retain their separate native Windows preview/replay evidence. P2-08
does not rerun those native windows or import their presentation timing into RAM
submission measurements. Shared logical keyboard completion and semantic export
are the fresh functional observations here; native input, native focus-tree and
native assistive-consumer breadth remain distinct limitations.

## Slint lifetime finding and bounded follow-up

The existing adapter creates and shows a fresh top-level component for each
paint. Its drop path does not hide it. Slint documents that showing a component
retains an additional strong reference until it is hidden; this is also visible
in the hash-verified 1.18.1 `i-slint-core/api.rs` and `window.rs` inputs. The missing
hide is a concrete adapter lifetime error consistent with the measured growing
process commit. It is not evidence that Slint intrinsically requires this memory.
[Slint component lifetime API](https://docs.slint.dev/latest/docs/rust/slint/trait.ComponentHandle.html#tymethod.show)

A bounded follow-up should either retain/reuse one window/component per view or
hide every temporary shown component with cleanup on all error paths. Test weak
handle expiration and process/counter plateaus across fixed repeated paints,
then rerun both the unchanged quality matrix and exact final links if portable
adapter code changes. Retain the current failed resource behavior as the baseline.
The 16 MiB staged text-image cap does not bound shown components or Slint internals.

## Recommendation and remaining gates

Recommend the custom Rust path for provisional P2-09 consideration because its
tested fidelity and current ownership behavior fit the required subset more
directly. This is a conditional engineering recommendation. It does not waive
its frame-budget misses or endorse a broad custom widget implementation.
Slint's primitive raster benefits deserve credit, but the current integration
cannot be adopted with lifetime retention, missing required effects/clip/opacity,
native text/semantic gaps and unresolved OS/API terms concealed by fallback.

P2-09 must review the full measured distributions and mandatory criteria before
committing. If the custom cost/completion path is not credible, record a blocked
decision and a bounded comparison follow-up rather than selecting by preference.
Do not maintain two production frameworks to avoid that decision.

Concrete completion work to assess at that gate:

1. Profile custom raster by rich/reduced mode and resolution. Cache unchanged
   shape/layout/effect inputs, bound the cached blur, avoid full redraw/copies
   where coverage is proved, and remeasure. No GPU or scalar/SIMD guest speedup
   is assumed. A host build's ISA/runtime costs differ from the freestanding probe.
2. Finish selected layout/constraint validation, stable focus/semantic identities,
   visible keyboard behavior and interruptible normal/reduced-motion behavior in
   P2-10. Preserve the font/script/emoji and native-consumer limits explicitly.
3. Implement actual hosted resource loading/publication and rejected-edit recovery
   in P2-11. Direct `Scene::publish` calls do not prove parser/watcher transactions,
   native-window lifetime continuity, resource retirement or an active helper route.
4. Remeasure sustained selected-core idle, damage, resource retention, reload peaks
   and latency in P2-12; review all M2 gates in P2-13. Do not treat this mixed RAM
   workload or stable custom process footprint as whole-desktop idle acceptance.
5. M3/M4 supply isolated guest loaders/heaps/quotas, one-thread runtime ownership,
   actual input/clocks/wait/presentation and guest assets. M5 supplies authorized
   typed behavior publication, quiescence, epoch fencing and safe old-code retirement.

The Slint royalty-free 2.0 text permits defined application integrations with
attribution and restricts standalone/API exposure and embedded use. Whether this
OS/platform and a future SDK fit an applicable license remains a review question;
running an original OS does not decide that classification. The pinned terms and
the public [royalty-free text](https://raw.githubusercontent.com/slint-ui/slint/v1.18.1/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md)
were rechecked on 2026-10-22. Qualified distribution review remains required;
no project information was sent to a vendor and no publication is authorized.
