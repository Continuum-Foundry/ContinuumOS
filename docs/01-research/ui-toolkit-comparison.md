# UI toolkit comparison: Slint and a small custom core

Status: bounded comparison reviewed; P2-09 provisional custom decision. Revision 2.

P2-08's [local comparison report](../../evidence/phase-2/P2-08/2026-10-22/comparison-report.md)
now fills every axis with measured evidence or precise unsupported/failing outcomes.
Its [host measurements](../../evidence/phase-2/P2-08/2026-10-22/measurement-summary.md)
retain all raw samples and separate resource continuity from compiled-helper
restart. Custom is recommended conditionally for P2-09 review, with substantial
performance completion work. Slint's current bridge retains shown components;
native fidelity and OS/API terms remain unresolved. P2-09 now
[provisionally selects custom Rust](../05-delivery/phase-2/ui-selection-review.md)
with exact license choices, frozen Slint comparison inputs and mandatory M2
completion/reconsideration gates. These host results do not pass a guest
performance or reload gate; ADR-0011 remains Proposed.

## Purpose

Challenge the assumption that we should implement every widget/layout facility
ourselves. Compare Slint with a minimal Rust UI core before expanding either. The
kernel, compositor, app permissions and live-update contracts stay our own.

Slint publishes custom platform/window-adapter interfaces. Its software path
supports partial redraw and no-OS configurations, but documents restrictions on
item transforms, effects and script coverage. Its live preview uses an interpreter;
native API changes can require application recompilation/restart. A hosted preview
does not establish that interpreter support exists in our freestanding setup.
[S76](sources.md#s76), [S77](sources.md#s77), [S78](sources.md#s78)

## Prototype scope

Build the same editor-with-sidebar scene and virtualized file list, a launcher/panel,
focus/navigation, theme change and repeatable input sequence. Use the same fonts,
resolution, scale, document corpus and memory-accounting rules. Include at least one
required complex-script case and a requested rich visual effect; document limitations
instead of silently choosing an easy demo that hides them.

For Slint: pin an exact version and disabled/default feature graph, identify allocation,
clock/input/font/clipboard hooks, implement a small platform adapter, and compile the
actual guest configuration. Test its app surface submission without assuming winit,
LinuxKMS, a browser, `std`, or a guest GPU API. Hosted interpreter support and guest
AOT support are separate configurations with separate evidence.

For the custom candidate: implement only the primitives/layout needed by the scenes,
not an entire general toolkit. Measure its font/shaping, widget authoring and testing
costs honestly. Reusing a rasterizer does not solve text semantics or accessibility.

## Comparison matrix

| Axis | Evidence required |
| --- | --- |
| Integration | Exact freestanding compile/link; identify every platform hook and unsupported dependency |
| Visual quality | Reference scene review, clipping, typography, scaling, required effects and fallback |
| Text/accessibility | Script corpus, selection/IME implications, semantic tree/actions and custom OS adapter effort |
| Reload | Token/data change plus compiled native behavior change; checkpoint continuity and incompatible edit rejection |
| Efficiency | Optimized frame-time distributions, idle wakeups, damaged area, resident/peak bytes and allocations |
| Authoring | Code/adapter complexity, diagnostics, iteration latency and future maintenance |
| Evolution | CPU fallback, GPU migration path, backend API stability and state export boundaries |
| Distribution | Exact licenses, notices and intended OS/toolkit/API distribution model |

Licensing must be resolved for the intended distribution, including whether exposing
toolkit functionality in the platform changes applicable terms. Do not infer that
running without Linux automatically makes a product an embedded system, or that a
desktop-app license automatically covers an OS platform. Record the selected license
and obtain clarification where necessary; this document makes no legal determination.
[S79](sources.md#s79)

## Hot reload strategy

Test three cases independently: reloadable tokens/assets, layout identity/focus,
and new compiled behavior in a replacement process. If guest interpreter support is
impractical, the common replacement/checkpoint mechanism can still reload compiled
UI code while the OS remains running. Report the actual component pause and rebuild
cost. Toolkit preview alone is not the final live-update feature.

## Decision gate

M2 chooses a provisional path using hosted data plus guest compile feasibility. M4
confirms it in the running guest before broad app/toolkit investment. Either candidate
must preserve R04/R05/R15 or supply a concrete bounded completion path; otherwise
revise the integration, not the requirements silently. A demonstrated low-cost toolkit
path can supersede a custom-core default through ADR-0011. Do not maintain two full
production UI frameworks indefinitely merely because two prototypes were built.

Artifacts: exact dependency/features manifest, two comparable scene implementations,
measurement report, missing-feature inventory, license record and recorded decision.
The provisional custom direction is scoped by P2-09; final guest confirmation remains M4.
