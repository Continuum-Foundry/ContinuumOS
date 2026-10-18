# ADR-0006: Native apps first; bounded Wasm as an experiment

Status: Proposed. Date: 2026-10-17.

## Context and decision

The OS needs native processes regardless of app scripting choices. Build a minimal
Rust SDK and static native apps first. Spike a Wasmi-based sandbox for UI/plugin
logic after the same lifecycle/import contracts work natively. Do not make a full
browser, .NET runtime, WASI or Component Model prerequisite for boot.

## Alternatives

All-native maximizes control but narrows app languages. All-managed/Wasm shifts
much foundation work into runtime porting. Wasmtime/AOT may later improve execution
speed but requires more platform and executable-code infrastructure.

## Consequences

Two formats create testing/SDK costs. An interpreter is not automatically efficient;
fuel does not bound an expensive host import. Heap, stack, module translation and
authority all need limits. Native process isolation remains valuable around a runtime.

## Gate and reconsideration

Demonstrate event handling and stateful replacement with strict memory/fuel/time
limits. Compare app responsiveness and footprint. Make Wasm the app default only
with measured benefits and a maintained feature/import compatibility policy.

Evidence: [S65](../01-research/sources.md#s65), [S66](../01-research/sources.md#s66),
[app spec](../02-architecture/applications-and-runtime.md).
