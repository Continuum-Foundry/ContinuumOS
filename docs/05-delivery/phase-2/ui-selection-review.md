# P2-09 — Provisional custom Rust UI decision

Review date: 2026-10-23. Reviewer: Codex local implementation/self-review,
not independent approval or an owner product-design decision. Source baseline:
`1e84686` (main after pull). ADR-0008/0011 remain **Proposed**.

## Decision and mandatory criteria

Select **custom-ui 0.1.0 + text-core 0.1.0 + editor-model 0.1.0**, authored in
Rust with validated data assets and the existing CPU renderer, provisionally for
P2-10 onward. No new UI language, interpreter, GPU, guest compiler or kernel UI
dependency is selected. This authorizes bounded selected-core completion, not a
claim that M2, guest integration, efficiency or publication has passed.

The exact configuration, active target packages/features, license choices,
notice hashes and frozen Slint inputs are in
[the selection record](../../../legal/ui-selection.json). It pins Rust 1.99.1,
the two Cargo locks, both fixture locks and six unmodified Noto font identities.
Custom defaults have no feature flags; `behavior-v2` is a separate compiled
experiment, not the production default. Host measurements used `m2-comparison`;
the target link used `experiments/ui-link` release settings on
`x86_64-unknown-none`. Those profiles are not timing-equivalent.
Frozen comparison text pins explicitly normalize CRLF to LF so Git checkout
conversion cannot alter their identity. Font and upstream notice pins remain
exact raw-byte hashes.

| Mandatory decision criterion | Review and disposition |
| --- | --- |
| Original OS and portable implementation | Pass for feasibility: no_std + alloc libraries, no host backend in the portable core; M1 kernel/ISO remain separate |
| Real chosen final link | Pass: [P2-07 custom ELF audit](../../../evidence/phase-2/P2-07/2026-10-22/custom-elf-audit.json) records entry 0x400000, separate RX/R/RW, zero unresolved symbols/relocations and no dynamic loader; bounded arena is a probe, never a guest heap |
| Required scripts, effects, semantics | Credible bounded completion: common Arabic/Hebrew/Indic/CJK text corpus, rich blur/tint, isolated opacity and rounded ancestor clips have custom evidence; keyboard/semantic workflows match independently. Required remaining input/layout/motion cases below stay in M2 |
| Resources, layout and eventual behavior replacement | Credible bounded completion: 100 direct publications preserve exact checkpoints and distinct compiled helpers restore logical state. P2-11 must implement real same-process resource loading; M5 must implement active behavior routing and retirement |
| Intended toolkit distribution terms | Resolved for this provisional custom configuration through explicit permissive license alternatives, OFL fonts and Unicode notices below; no Slint code/API is adopted. Actual artifact packaging and project release authorization remain release gates |
| Honest efficiency and completion scope | Pass as a decision, not a performance result: large measured misses are retained, with bounded P2-10/P2-12 work and a stop/reconsideration rule. No numeric guest target is relaxed |

An unmet mandatory criterion blocks adoption. The latter three rows establish a
completion route, not passing implementation tests. P2-10/P2-11/P2-12/P2-13 cannot
be marked Done by this review. No broad widget or app catalogue precedes them.

## Review of all comparison axes

The accepted [full comparison](../../../evidence/phase-2/P2-08/2026-10-22/comparison-report.md),
[quality review](../../../evidence/phase-2/P2-08/2026-10-22/quality-review.md),
[measurements](../../../evidence/phase-2/P2-08/2026-10-22/measurement-summary.md)
and [target review](../../../evidence/phase-2/P2-07/2026-10-22/README.md)
remain the measured record; this decision does not rerun or rewrite them.

| Axis | Chosen custom path and tradeoff | Slint alternative and disposition |
| --- | --- | --- |
| Integration | Actual shared scene/text/raster final link, 32 active packages including four first-party packages; explicit runtime hooks | Slint 1.18.1 AOT also links; 59 normal runtime / 146 active build packages; interpreter unavailable offline. Link success alone does not choose either |
| Visual quality | Required rich mode, clips and isolated opacity work in fixed scenes. Hard rounded-edge coverage, fixed rectangles and immediate transitions need completion | Current bridge fails rich blur, rounded ancestor clip, launcher opacity and fractional blend. Its opaque fallback cannot replace required rich mode |
| Text/accessibility | Pinned shared shaping/raster, logical state and semantic actions; P2-10 must complete visual navigation and adapter policies | Uses the same custom text Image/controller/semantics adapters; no native Slint text/semantic advantage demonstrated. Neither has native IME or screen-reader acceptance |
| Reload | Exact owner checkpoints survive direct token/layout publication; compiled variants restore snapshots. Data reconciliation avoids requiring a guest interpreter | Similar bounded helper/publication feasibility; private AOT components rebuild. Neither demonstrates native-window/file-driven reload or active behavior cutover |
| Efficiency | Trace-off frame p50/p95/p99 68.030/159.358/203.213 ms; 1500/1500 misses at 16.667 ms. Peak private commit 166.496–167.105 MiB across runs, six views plus copies; no guest estimate | 18.873/41.029/56.694 ms and 876/1500 misses; often faster, sometimes different visual work. Peak 1132.676–1134.441 MiB reflects a shown-component retention defect, not intrinsic toolkit footprint |
| Authoring | 1906 custom lines include facilities shared by both candidates; 2092 model/text lines shared. More first-party layout/control/motion maintenance is accepted for the bounded core | Adds 489 portable bridge/AOT/platform lines, but still needs common facilities and feature repair. No measured clean-build or engineering-hours productivity ranking |
| Evolution | CPU renderer and logical schema under our control; damage/cache/pacing and future surface backend can evolve independently | Upstream primitive speed remains a valid future alternative if fidelity/lifetime/terms close. No GPU migration or API-stability benefit demonstrated for this guest |
| Distribution | Explicit MIT/Apache-2.0/Zlib alternatives plus OFL/Unicode notices for this target. Original Rust APIs can be part of a future SDK subject to first-party rights review | GPL-3.0-only / royalty-free 2.0 / commercial alternatives recorded, none selected. OS/platform/API suitability unresolved, so Slint adoption is blocked |

Both paths submit full-surface damage today. Only about 12.35% of custom's
trace-on submitted pixels changed; this is motivation to investigate damage,
not proof of valid smaller regions or a promised speedup. Even reduced effects
miss targets: custom scale-1 HD/full-HD p50 is 32.90/40.65 ms. Slint's faster
rich-request samples paint a different opaque fallback. The decision prioritizes
required fidelity and an owned completion route without claiming custom is faster.
Laptop power/load, mixed cell weights, small per-cell tails, common observation
costs and quantized idle CPU limit the comparison. Keep the 256 MiB guest cold-idle
target and all frame/input/reload targets unchanged.

## License choice and intended use

Intended eventual use is an original proprietary OS UI library, OS shell/apps,
and first-party SDK interfaces, with unchanged bundled fonts. P2-09 approves
only local development. It grants no new first-party rights or publication permission.
The selected upstream license texts permit these kinds of use subject to their
conditions; this is a scoped engineering suitability review, not legal certification.

The selection record covers all **28 third-party target packages** individually,
with exact versions, checksums, active features and retained notice hashes.
Select MIT wherever supplied (including packages with historical `MIT/Apache-2.0`
metadata, backed by their actual MIT texts); select Apache-2.0 for
unicode-linebreak 0.1.5 and Zlib for foldhash 0.1.5. Preserve all existing notice
texts, including supplemental libm terms and Unicode copyright files, even where
an alternative is selected. No upstream source is modified by P2-09.

| Material | Selected terms and required delivery treatment |
| --- | --- |
| MIT packages | Include each actual copyright and permission text with substantial copies; no assumption that root LICENSE covers dependencies |
| unicode-linebreak | Apache-2.0 copy, source attribution and any applicable NOTICE; modified files must be marked if later changed; patent/termination and trademark limits remain |
| foldhash | Zlib notice retained in source distributions, origin truthful, altered source marked; retain the notice in binary notice bundle too |
| Six Noto fonts | OFL-1.1, unmodified byte identities; readable copyright/license with each copy, no font-only sale, fonts retain OFL. Any subset/format/name change requires renewed modified-font/reserved-name review |
| Unicode corpus and embedded data notices | Retain package COPYRIGHT and official Unicode License V3 for test data with copies or accompanying documentation; no endorsement claim |
| Original code / future SDK | Root all-rights-reserved policy remains; no public SDK grant is created here. Re-audit exact shipped artifacts and compiler/runtime notices before distribution |

Public license texts were checked 2026-10-23 without sending project details:
[MIT](https://opensource.org/license/mit),
[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0),
[Zlib](https://opensource.org/license/zlib),
[OFL-1.1](https://openfontlicense.org/open-font-license-official-text/),
[Unicode](https://www.unicode.org/license.txt).
Their grants/conditions support the scoped interpretation above. Exact vendored
notices and source hashes govern the pinned inputs rather than moving web pages.

Slint's pinned [royalty-free 2.0 terms](https://raw.githubusercontent.com/slint-ui/slint/v1.18.1/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md)
restrict standalone/API exposure and embedded use and require attribution. No
assumed OS exemption, GPL relicensing or commercial agreement resolves that here.
Slint remains unselected, so its unresolved applicability does not enter the
chosen production closure. Existing combined comparison binaries still contain
Slint and are **not** approved delivery artifacts. No vendor was contacted.
The handling hold, ownership/title review, actual artifact SBOM/notice packaging
and qualified release review remain under [LEGAL.md](../../../LEGAL.md) and
[RELEASE.md](../../../RELEASE.md); `distribution_cleared` stays false.

## Source ownership and guest integration

| Owner boundary | Selected work / integration obligation |
| --- | --- |
| crates/editor-model | Authoritative document/history/revisions and one typed executor; window/focus/search/scroll logical state stays outside raster allocations |
| crates/text-core | Allowlisted font resources, shaping/bidi/raster and bounded caches; preserve the authored script corpus and explicit missing-glyph reports |
| crates/custom-ui | Scene/layout, input transforms, semantic identity, CPU paint and surface lease logic. Rust values remain internal; never serialize memory layouts |
| tools/xtask plus tools/Preview-Custom.ps1 | Host orchestration, file IO, resource builder/watcher and Windows presentation. Keep native window/session alive across L0/L1; host FFI stays outside portable crates |
| M3 kernel/runtime | Checked user ELF mappings, guarded stack/entry, owned fallible heap/quota/reclaim, user FP/SIMD policy, isolated caller/session provenance, failure/reaping and one owning UI execution context |
| M4 surface/input/semantic adapters | Versioned broker messages for acquire/write/commit/release, generation/stride/damage checks, separate compositor window authority; monotonic input/frame deadlines and wait/wake; ordered key/text/composition/reset, focus/capture and revision transforms; semantic snapshots/actions through typed executor |
| M4 resources and guest acceptance | Immutable font/image handles, bounded generation staging and release, same selected core on actual guest presentation; real device/scale/text/semantic tests and guest boot-ID/uptime proof for resource reload |
| M5 behavior replacement | Quiesce logical owner, migrate explicit versioned checkpoints, validate schema/rights/revision/epoch, atomically route one writer, drain callbacks and retire old process only after release; no native Rust pointers or arbitrary dlclose |

Use the existing [hook inventory](../../../experiments/ui-link/hooks.json) and
[surface](../../06-contracts/ui-and-scene-protocol.md) /
[action](../../06-contracts/actions-and-context.md) drafts as integration inputs.
M3/M4 must supply independently tested codecs, authority and quota enforcement;
the hosted structs and 64 MiB non-reclaiming probe arena are not a process ABI or
production allocator. Guest timing, font service, loader and UI are absent today.

Freeze `crates/slint-ui`, Slint fixture/golden data, optional `slint-candidate`
host feature, `slint-aot` probe configuration and Slint preview/review tools as
comparison/test material at this decision's input hashes. Keep their old evidence
and failure records immutable. Shared model/text/surface code remains selected
code and may change; do not duplicate it to maintain a second production toolkit.
Run frozen tests only for relevant shared regressions. A Slint-specific repair,
new feature or renewed adoption comparison needs a separately scoped experiment.

## Required completion and reconsideration

These are implementation obligations, not completed cases. Preserve the entire
30-configuration/150-capture matrix, 300 workflow states, 45 text cases and 63
integrated text captures; do not reduce the corpus to manufacture a pass.

| Gate | Concrete remaining work and required failure cases |
| --- | --- |
| P2-10 (next) | Bounded row/column/overlay/scroll/virtual-list measure/arrange; cycle/depth/finite bounds; stable semantic IDs; separate layout/paint/semantic invalidation with counters; resize/scroll/fractional hit transforms and stale-target rejection |
| P2-10 input/text | Keyboard-only editor/search/list/settings and semantic-action parity, visible non-color focus/high contrast; key/text/composition separation, visual bidi affinity and selection; capture/modifier release on moved/removed targets, modal dismissal and queue overflow. Retain emoji/.notdef and native IME/assistive limits |
| P2-10 motion/quality | Interruptible normal transitions starting at current visible state, reduced motion without unnecessary movement, bounded scheduled caret/clock/drag work; validate rich/reduced effects and rounded-edge quality against unchanged tolerances |
| P2-11 | Actual bounded file builder/parser, debounce/partial-write recovery and atomic token plus structural layout generation; same native window/process and exact unsaved text/history/selection/scroll/focus/geometry continuity; old readers retain resources; reject missing/corrupt/duplicate/cyclic/stale/version/oversize/headroom failures; typing/gesture/composition and delayed-release cases |
| P2-12 | Profile raster/effect/text costs; cache immutable shape/layout/shadow/blur inputs with explicit budgets/invalidation, conservative old/new/filter/occlusion damage and full-redraw oracle; reduce unnecessary copies where leases permit; static idle sleeps; five-repeat optimized raw metrics and 100 real reloads with fixed growth tolerances, old+new peaks and named timer costs |
| P2-13 | Clean selected host and final target links, complete M2 matrix and self-review; no skipped mandatory behavior or claimed guest target pass |

P2-10 owns layout/input/semantics/motion completion; P2-11 owns actual resource
continuity. P2-12 owns measured optimization/retention acceptance. Neither new
mandatory behavior nor real L0/L1 publication is deferred to M4/M5. Native IME,
native assistive consumer, enforced capabilities and guest L2 retain their already
declared later gates; hosted modeled interfaces are not substitutes for them.

Reopen ADR-0011 before broad widgets if the fixed custom text/effect/semantic
corpus cannot pass, supported structural reload requires restarting its owner,
or bounded layout/cache/damage work cannot produce a credible measured hosted
baseline without dropping required fidelity or accepting unexplained growth.
P2-12 must retain misses and bounded follow-ups; misses are not silently waived
and do not certify the guest budget. Reopen at M4 if target runtime hooks,
guest responsiveness/memory or resource continuity fail. New license inputs,
font modifications, public API changes or a proposed Slint adoption require a
new scoped rights review. A repair of Slint lifetime alone does not close its
other gaps. M18 acceleration is a separate gate, not the assumed rescue path.

## Reproduce this decision review

Run the required offline prerequisites with existing signing/tool enrollment:

```powershell
pwsh -NoProfile -File tools/Recheck-P2Prerequisites.ps1
python tools/Review-P2Selection.py <prerequisites.json> build/m2/P2-09/review
node audit/check-docs.cjs
node audit/check-governance.cjs
pwsh -NoProfile -File audit/validate-contracts.ps1
# After staging the reviewed compact bundle:
node audit/check-repository-hygiene.cjs
```

The selection reviewer verifies upstream evidence manifests, unchanged chosen
source/locks/features/notices, frozen prototype identities, fonts and prerequisite
results, with deliberate invalid selection mutations. It does not render, boot,
benchmark or supply legal approval. Full command logs stay local. The retained
[review bundle](../../../evidence/phase-2/P2-09/2026-10-23/README.md) records the
five story criteria and exact review inputs. No upload or Jira issue is authorized.

## Final hosted completion review — 2026-10-24

[P2-13 aggregate acceptance](../../../evidence/phase-2/P2-13/2026-10-24/README.md) now closes P2-10–P2-13's
declared M2 hosted completion list, including clean retained/cache final linking.
The original P2-09 JSON and its chosen-source pins remain a historical decision
snapshot; final source identities, exact tested artifacts and current raw samples
are recorded by P2-13. All eight comparison axes, license choices and frozen
Slint findings remain intact. Actual distribution clearance remains false.

No required hosted behavior is deferred to guest work. PERF-12-01–03 retain the
measured raster/copy, staging and timer work; guest numeric targets are unchanged
and unaccepted. M3 runtime/authority, M4 backend/resource continuity and M5 behavior
replacement remain the next integration gates. Reconsideration rules above and
the ADRs' Proposed status remain in force.
