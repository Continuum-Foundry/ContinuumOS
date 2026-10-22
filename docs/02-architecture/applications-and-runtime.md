# Applications, SDK and execution runtimes

Status: proposed. R05, R07, R10, R14. Decision: [ADR-0006](../04-decisions/0006-app-runtime.md).
The SDK also exposes [typed actions and scoped context](actions-and-optional-ai.md)
under R19. Optional inference is a separate service after M5, not a required
runtime or boot dependency.

## Native application baseline

Start with statically linked Rust process images built against our versioned SDK.
The kernel's ELF loader and userland entry point must actually exist before the
first native app. The SDK wraps handles, events, memory, file access, surfaces,
logging and lifecycle methods. It does not pretend to be POSIX or full Rust `std`.

Each app package declares identity, entry point, supported protocol/state versions,
required/optional capabilities, memory/CPU quotas and update policy. Launching an
app grants a narrowed set of handles, not unrestricted access to all services.
Permission changes during an update are reviewed by policy and not automatically
accepted because the previous version was trusted.

## First applications

| App | Primary purpose | Reloadable state |
| --- | --- | --- |
| Diagnostics console | Logs, status and controlled commands | View filters and scrollback; privileged commands require separate authority |
| Text editor | Demonstrate meaningful state continuity | Document text/revision, selection, undo metadata, viewport, unsaved flag |
| File browser | Exercise filesystem, virtual lists and async operations | Directory capability reference, selection, sort mode, viewport |
| Settings | Exercise token reload and persistence | Declarative settings with explicit schema versions |
| Inspector | Show scene damage, resource use and update epochs | Diagnostic view state, no authority hidden in normal app state |
| Installer/first-run setup (M7) | Mandatory graphical keyboard-accessible M8 installation and account setup | Non-secret view/progress state only; no credentials or privileged disk handles; active writes remain service-owned |

Do not call the console a POSIX terminal until process I/O, a shell and terminal
semantics actually exist. M10/B48 formalizes native install/launch/update/uninstall,
permission increases and app-data retention using the same SDK/package graph.
See [desktop service responsibilities](../06-contracts/desktop-services.md).
M16's [Windows study](../01-research/windows-compatibility-feasibility.md) and M17's
[browser study](../01-research/browser-feasibility.md) are research/prototype gates,
not installed app/browser promises or M8/M14 prerequisites. Keep native ELF first;
no foreign app is assumed to export M5-compatible state or typed actions.

## Application lifecycle

States: installed → starting → ready → running → quiescing → checkpointed → retiring
→ stopped, with failed/recovering branches. Ready means the app can serve declared
interfaces, not merely that its process exists. A hung app is constrained by deadlines
and resource limits. Crash loops trigger backoff and a recoverable disabled state.

Apps export logical checkpoints at safe boundaries. An editor should preserve
unsaved text, not only window geometry. File/network handles are brokered or
reconstructed from authorized logical references. An unsaved buffer containing
sensitive data follows the checkpoint storage/privacy policy.

## Wasm experiment

This independently gated work belongs to M20/B27; it does not require M15 or gate
M8/M14. Its declared B12/B18/B20 prerequisites determine execution readiness.

Use a no-std-capable interpreter such as Wasmi for a bounded prototype. Its published
features include fuel metering; we must still supply allocation and secure host
imports. [S65](../01-research/sources.md#s65)

Define an MVP import ABI for events, logging, timers, UI updates and granted
resources. Pin supported core Wasm features. Fuel bounds execution work, not all
wall-clock time: expensive or blocking host functions need separate deadlines and
quotas. Limit linear memory, module size, recursion/stack, compilation/translation
work and outstanding host calls. Yield cooperatively at known boundaries; terminate
an unresponsive module without taking down the renderer.

Replace a whole module instance with state export/import. Do not copy raw Wasm
linear memory as the universal migration strategy; internal layouts change and
host handles may not survive. Feature-rich WASI/Component Model support is optional
later integration. WIT can inspire contracts without requiring a full component
runtime at boot. [S69](../01-research/sources.md#s69)

Native process isolation still matters if Wasm hosts expose powerful resources or
contain runtime bugs. A later AOT approach requires code provenance, target/CPU
compatibility, executable-memory policy and platform functions.
[S66](../01-research/sources.md#s66), [S67](../01-research/sources.md#s67)

## Compatibility policy

Version the SDK and wire interfaces independently from app release numbers. Publish
the support window only after tests exist. Old clients may continue through a
compatible adapter; unknown mandatory features fail clearly. A package with the
same source API can still be binary-incompatible. No binary compatibility promise
is made before the ABI gate.
