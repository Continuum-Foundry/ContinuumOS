# System layers and ownership

Status: proposed architecture. Requirement links: R01, R05, R06, R10, R14.

## Layer and authority map

| Layer | Owns | May depend on | Must not own |
| --- | --- | --- | --- |
| Firmware/bootloader | Boot discovery and handoff | Firmware/device boot interfaces | Runtime desktop state |
| Architecture HAL | CPU tables, exception entry, MMU primitives | CPU definitions | App or update policy |
| Kernel | Address spaces, threads, object tables, IRQ routing, clocks | HAL and bounded allocators | Widgets, file formats, network policy |
| Initial supervisor | Startup graph, process lifecycle, quotas | Kernel API, immutable boot packages | User document contents |
| Resource/service brokers | Stable routes and selected long-lived resources | Kernel handles, narrow service protocols | Arbitrary serialized app pointers |
| Device services | Declared device access and queues | Mediated MMIO/IRQ/DMA resources | Unrestricted physical memory |
| Data/network services | Files, sockets, settings ownership | Drivers and object protocols | Global UI focus |
| Compositor/display core | Surfaces, presentation, input hit testing | Graphics device interface, kernel events | App document semantics |
| Shell/session policy | Workspaces, launcher, window placement | Compositor, app lifecycle | Arbitrary hardware access |
| UI framework | Layout, semantics, event dispatch, paint description | App SDK and scene protocol | Kernel privilege |
| Apps | Domain behavior and logical state | Granted service handles | Other app memory and ambient devices |
| Host developer tools | Build, packaging, update initiation, test orchestration | Local tools and authorized guest endpoint | Implicit authority from guest-supplied commands |

## Initial stable core

Keep the kernel, loader, supervisor, route/resource broker and minimum recovery
display path stable during the first live-update milestones. The shell policy and
apps can evolve around them. An always-up resource broker is itself a dependency
whose bugs can block recovery; it needs narrow functionality and persistent state
only where essential. It is not a magic place to move all mutable state.

Separate display ownership from replaceable shell behavior. A stable compositor
can retain existing client surfaces while shell controls update. Replacing the
compositor itself is later work with reconnect/reconstruction semantics, not an
initial zero-flicker promise.

## Dependency graph rules

Boot ordering is explicit and acyclic where possible: kernel → supervisor →
minimal drivers → storage/package access → brokers → compositor/input → shell → apps.
The update channel must not depend on a service currently being replaced unless a
separate recovery path remains available. Serial diagnostics stay available when
the renderer, storage or network service fails.

All IPC waits need deadlines/cancellation or a justified permanent wait. Avoid
synchronous cycles such as compositor → app → settings → compositor. Bulk data
uses brokered shared memory; control messages remain bounded and copied initially.
Provide per-service queues and quotas so an app cannot exhaust the supervisor.

## Ownership principles

Each logical item has exactly one authoritative owner and an explicit lifetime.
Window geometry belongs to session policy; display buffers belong to a surface
owner/broker; document edits belong to the document service or app, never both.
Choose app-owned documents initially with checkpoint export. Introduce a separate
document service only if recovery/multi-view requirements justify its complexity.

Resources crossing a replacement boundary are either stable broker handles,
reconstructed from validated descriptions, or declared nontransferable. A handle
identifies authority and lifetime, not just a number persisted to disk. On reboot,
new handle tables are created from authorized logical references.

## Review gate

For each new subsystem, document owner, imports, granted authority, quotas, startup
dependencies, crash behavior, update class, persistent schema and tests. If its
failure can stop all recovery, include it in the relevant trusted/recovery core.
Do not call a service isolated until separate privilege/address-space tests pass.
