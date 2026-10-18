# Live update model and guarantees

Status: proposed contract. R04–R07, R12–R14, R17.

## Supported classes

| Class | Example | Mechanism | Initial continuity promise |
| --- | --- | --- | --- |
| L0: assets/tokens | Palette, icons, spacing | Validate and atomically switch resource generation | Existing scene/documents remain; incompatible layout rejected |
| L1: layout/behavior data | Panel structure, key bindings | Reconcile stable logical IDs at an event/frame boundary | Restore compatible focus/selection; cancel unsupported active gestures visibly |
| L2: app logic | Editor behavior or widget module | Replace isolated process/module instance and migrate logical state | Preserve explicitly supported document/UI fields |
| L3: user-space service | Settings or file API implementation | Quiesce, checkpoint, prepare, switch stable route/epoch | Controlled pause, no stale writer; operation-specific continuity |
| L4: driver/display runtime | Input, network, compositor | Drain/reset/reconnect/rebuild resources | Declared disruption; separate per-device experiments |
| L5: kernel/boot foundation | Paging, scheduler, boot ABI | Stage and boot a new kernel | Reboot required initially |

Class is declared by the update planner after checking changed dependencies, not
guessed from a filename. A layout change can require logic/state migration; a shared
protocol change can require a coordinated set. “Hot reload supported” never means
every arbitrary edit is legal live.

## Invariants

1. Only one authoritative writer per logical owner/epoch.
2. No activation of unverified or incompatible content.
3. No resource authority created from serialized raw handles or pointers.
4. No old code retired while any thread, callback, timer or import can execute it.
5. No mixed required dependency versions visible during a coordinated route switch.
6. Pre-decision update failure preserves the old generation and its intact state,
   with mutations resumed safely; unrelated old-process crashes follow checkpoint recovery.
7. Post-decision failure follows a documented recovery path; no generic data rollback claim.
8. Durable success survives crash/reboot through persistent records and available content.
9. Ephemeral success is explicitly labeled and does not imply persistence after reboot.
10. Host and guest agree on transaction outcome by queryable ID, not an unreliable final packet.

## Continuity levels

- **Machine continuity:** kernel boot ID and monotonic uptime remain unchanged; VM reset
  is neither required nor hidden. Prove this independently of the desktop process.
- **Process continuity:** not required for native process replacement. New code can run
  in a new process while the machine stays up.
- **Interaction continuity:** windows may remain visible while events queue briefly;
  bounded queue overflow has defined cancellation/retry behavior.
- **State continuity:** versioned fields migrate. Unsupported fields/schemas are rejected
  or reset only under an explicit policy communicated to the developer.
- **External continuity:** files, sockets, device streams and remote side effects each
  have their own transfer/reconnect rules. They are not guaranteed by an in-memory swap.

## Why this is feasible

Existing systems demonstrate module/service replacement and state-aware evolution.
Theseus motivates disciplined cross-component state; MINIX motivates quiescence;
Erlang motivates code-generation lifetime management. Our process/broker design is
a proposed synthesis requiring its own evidence.
[S14](../01-research/sources.md#s14), [S15](../01-research/sources.md#s15),
[S16](../01-research/sources.md#s16)

## User/developer experience

Host save triggers debounce/build. Build errors stay on host and do not disturb the
guest. A successful build produces immutable content plus an update plan. The guest
reports received, validated, prepared, activating, healthy, finalized or rejected,
with a precise reason. Include source revision, component/build IDs, migration pair,
latency and persistence mode.

Unsupported live changes say `reboot_required`, `service_restart_required`,
`incompatible_schema`, `permission_change`, `insufficient_headroom` or another
specific error. Never silently reboot to make the experience look successful.

The ISO is generated separately from a selected coherent package graph. Replacing
its file on the host or reattaching a virtual CD does not replace running code.
Guest update content travels through the declared development protocol.
