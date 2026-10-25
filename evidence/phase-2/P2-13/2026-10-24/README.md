# P2-13 — Final Phase 2 hosted acceptance passes locally

**P2-13 is Done; M2's declared hosted subset passes local acceptance.** This is
Codex implementation/self-review, not independent approval, guest acceptance or
a usable desktop OS. Frame/resource-latency misses remain explicit; the 256 MiB
guest cold-idle and guest responsiveness targets remain unchanged and unaccepted.

Source base: `82ef69d7f5546e2d54d1b2ca2cdae82ae9feeb8a` plus the exact [tested inputs](tested-inputs.json)
and [LF/source provenance](source-provenance.json). Final run:
`build/m2/P2-13/acceptance-1792868580682524600`. All sixteen required rows pass.
Use the [aggregate runbook](../../../../docs/05-delivery/phase-2/acceptance-runbook.md).

| Artifact | Exact SHA-256 | Outcome |
| --- | --- | --- |
| Signed optimized hosted executable | `2e37e09059db0473b1d87ca6f4067ad4123331e880016179c3b563da59327a5e` | Executed for final fixtures, native replay and measurements |
| Selected freestanding ELF | `1439de70bd36c6d5ddc8673b5917460ab27d84e79d36beb87df3b5db0acab214` | Clean final link and independent audit; never executed |

Ten fresh prerequisites include Phase 0 compile/link, clean builds/source paths
with spaces, signer/compiler/dependency checks and M1 tool doctor. The clean host
uses `m2-comparison`, Rust 1.99.1/LLVM 23.1.1 and locked offline inputs. Native
replay records the same signed executable hash. The separate clean ELF retains
actual `Frame::draw` and `Engine::layout_cached`, distinct RX/R/RW loads, zero
unresolved symbols/relocations and no host runtime. Its thirteen malformed-image
negatives, real missing-hook link failure and accidental-std compilation failure
pass. The 64 MiB non-reclaiming arena is a probe, never a guest heap.

The complete common/selected renderer, keyboard/pointer, semantic/readability,
typed-action/context and text subsets pass without reduced corpus/tolerances.
Final shared/text/action suites run twice with byte-identical result hashes;
native HarfBuzz and checkpoint decoding remain separate consumers. The existing
ignored unit paint-pressure case is disclosed; required real paint-pressure
preservation is exercised by the reload suite. No required matrix row is skipped.

Native token and structural layout publication changes generation 7 to 8 in the
same worker process and window, preserving synthetic unsaved text, complete
history/checkpoint, selection, scroll, search, focus and geometry, with continued
editing. Failure, input reconciliation, delayed readers and headroom refusal pass.
The before/after images support visible change; lifecycle/state traces prove
continuity. These are host identities and monotonic observations, not kernel
boot ID or guest uptime.

| Final real file reloads | Measured cycles | Completed-build-to-RAM p95 | Staging requested-Rust peak |
| --- | ---: | ---: | ---: |
| 1280x720 | 100 | 930.549 ms | 56,636,513 B |
| 1920x1080 | 100 | 875.284 ms | 75,068,517 B |

There are **3,000 frame / 8,444 process samples** across five
fresh-process paired trace repetitions. **2,962/3,000 frames exceed the
16.667 ms elapsed frame budget.** Full distributions, phase/input/submission
clocks, paired instrumentation variation, static/caret/clock/hidden/minimized
costs and memory/old-plus-new peaks are in [metrics](metrics.json) and the raw
CSV files. CPU process counters are separate from elapsed per-frame clocks;
RAM submission is not scanout or photon latency. Resource visibility includes
the fixed debounce, validation, staging and submission, excluding building.

All 108 damage/full images match exactly. Each resolution's 100 measured reloads
after five warm-up cycles preserves the full checkpoint; no retired root survives
settlement. Fixed cache/growth budgets and the independent normalized live-Rust
allocation plateau pass. Native static/hidden/minimized phases submit zero frames.
Host process/cached-font/framebuffer/harness attribution is retained, with no
claim of whole-system guest budget acceptance. PERF-12-01–03 retain bounded
raster/copy, staged-frame and timer follow-ups without dropping fidelity.

The preserved comparison/selection review checks all eight axes, the 32-package
selected closure, intended license alternatives/notices and unchanged frozen
Slint inputs. P2-06–09 manifests verify 171 retained files. Original feature,
lifetime and performance failures remain visible. P2-09's chosen-source pins and
`m2_complete: false` remain historical to that decision; P2-13 records the final
source and M2 conclusion. Actual distribution clearance remains false.

| Required row | Review | Compact evidence |
| --- | --- | --- |
| prerequisites-clean-artifacts | Pass | [prerequisites.json](prerequisites.json), [selected-acceptance.json](selected-acceptance.json), [selected-elf-audit.json](selected-elf-audit.json) |
| comparison-selection | Pass | [selection-review.json](selection-review.json), [selected-inputs.json](selected-inputs.json) |
| renderer-fixtures | Pass | [corpus-result.json](corpus-result.json), [corpus-pixel-review.json](corpus-pixel-review.json) |
| keyboard-pointer | Pass | [selected-result.json](selected-result.json), [selected-review.json](selected-review.json) |
| text | Pass | [text-result.json](text-result.json) |
| semantics-readability | Pass | [selected-result.json](selected-result.json), [selected-review.json](selected-review.json) |
| typed-actions-context | Pass | [editor-model-result.json](editor-model-result.json), [commands.json](commands.json) |
| token-reload | Pass | [native-reload.json](native-reload.json), [resource-independent-review.json](resource-independent-review.json) |
| layout-reload | Pass | [reload-result.json](reload-result.json), [native-reload.json](native-reload.json) |
| failed-update | Pass | [reload-result.json](reload-result.json), [opacity-1280x720.json](opacity-1280x720.json), [opacity-1920x1080.json](opacity-1920x1080.json) |
| reader-retirement | Pass | [reload-result.json](reload-result.json), [metrics.json](metrics.json) |
| repeated-reload | Pass | [cycles-1280x720.csv](cycles-1280x720.csv), [cycles-1920x1080.csv](cycles-1920x1080.csv), [layout-cycles-1280x720.csv](layout-cycles-1280x720.csv), [layout-cycles-1920x1080.csv](layout-cycles-1920x1080.csv) |
| damage-idle | Pass | [damage.json](damage.json), [damage-review.json](damage-review.json), [native-idle.json](native-idle.json) |
| metrics | Pass | [metrics.json](metrics.json), [frames-0-off.csv](frames-0-off.csv), [process-0-off.csv](process-0-off.csv) |
| harness-cleanup | Pass | [harness-negatives.json](harness-negatives.json), [negative-processes.json](negative-processes.json), [commands.json](commands.json) |
| documentation-regressions | Pass | [commands.json](commands.json), [repository-checks.json](repository-checks.json) |

[The criterion review](review.json) records the conclusions and named next gates.
[Failed/interrupted attempts](development-findings.json) preserve the signer-path
refusal and LLVM demangling-auditor correction. No host policy or enrollment was
changed. The actual early-exit/timeout helpers and seven record mutations reject;
all owned final helpers are reaped. Full logs, binary/ELF/LLVM output, raw captures,
large traces and complete failed runs stay local under ignored `build/` and `work/`.
This bundle retains compact raw samples, reviewed state/image observations and
identities; its manifest covers only the files retained here.

Kernel/ISO/package/launch inputs and pins are unchanged; no new M1 boot result is
claimed. M3 isolation/runtime/authority, M4 guest presentation/resource reload and
M5 behavior replacement remain open. Native IME/assistive consumers and durable
storage remain later gates. ADRs stay Proposed. No P3 stories, remote issue,
push, publication or release is part of this work.
