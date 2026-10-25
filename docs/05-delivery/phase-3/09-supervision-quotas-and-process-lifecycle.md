# P3-09 — Supervise process trees and enforce resource ceilings

Issue type: Story. Epic: P3. Status: Not started. Owner: Unassigned.
Depends on: P3-08 Done. Suggested labels: `phase-3`, `supervisor`, `quotas`.
Backlog: B13 supervision/quotas, B11 child launch completion.
Requirements: R10/R14/R18; preparatory R06/R09/R17. Test families: T01/T03/T06/T11/T09.

## User story

As the system supervisor, I need bounded child admission, failure reporting and
complete reaping so a crashing or resource-hungry service cannot consume the guest.

## Implementation scope

1. Start a minimal trusted user-mode supervisor from the ISO package. Define its
   bootstrap grants, process tree, lifecycle states, create/start/terminate/join
   semantics and separation from kernel emergency handling. Publish a process only
   after image/context/quota admission commits; retain precise startup/exit reasons.
2. Enforce per-process and subtree ceilings for memory, threads, handles, mappings,
   channel messages/bytes, waiters/timers and diagnostic rates. Reserve capacity for
   supervisor/kernel progress. Track delegation so many children cannot bypass an
   ancestor ceiling. Use bounded CPU scheduling shares/priority ceilings, not an
   unsupported hard CPU-time or real-time guarantee.
3. Implement child launch through trusted context/delegation checks. Ordinary apps
   cannot forge another principal/session, inherit unlisted privileged handles or
   raise priority/resource/authority ceilings. Broker-grant policy follows at P3-10;
   this story enforces structural ceilings even for authorized launch requests.
4. Reap faults, exits and killed/hung children: deschedule first, drain/cancel owned
   waits, settle transfers, release tables/mappings/allocator backing, then recycle
   instance IDs. Define orphan ownership and supervisor death, avoiding reuse while
   a kernel continuation, queued reference or old thread can still reach the instance.
5. Add bounded restart budgets/backoff and crash-loop quarantine. Supervisor/broker
   failure closes privileged admission until trusted bootstrap restores it; automatic
   restart never silently adopts a stale session/grant or masquerades as preserved work.

## Acceptance criteria

- [ ] Real user-mode supervision launches, observes, joins and reaps normal, failed,
  killed and hung tasks with exact lifecycle/exit outcomes and healthy-task progress.
- [ ] Per-process/subtree quotas and priority ceilings are enforced before resource
  commit; child fan-out cannot bypass them or consume reserved control capacity.
- [ ] Unauthorized identity selection, handle inheritance and ceiling escalation are
  rejected; stale/reused process IDs cannot target a new instance.
- [ ] Every declared death/orphan/transfer/wait teardown path releases its owned
  resources without dangling execution, double completion or unbounded zombies.
- [ ] Restart storms are bounded; supervisor death yields a declared fail-closed
  recovery or controlled failure with correlated diagnostics, never false readiness.

## Tests and evidence

Run fork/launch storms within static-image support, thread/handle/memory/message
exhaustion, priority escalation, OOM/panic, CPU hog, blocked child kill, parent death,
join-after-exit and supervisor failure. Measure post-settlement inventories and
restart counts; check denial leaves old healthy processes usable. Retain child
context/grant manifests, exact quota errors and process-tree cleanup traces.

## Exclusions and handoff

No persistent service manager, credential administration, arbitrary executable
installation or live-state-preserving restart. P3-10 adds ephemeral scoped policy
on top of these enforced launch/resource ceilings; P3-11 adds stable routes.
Sources: [kernel design](../../02-architecture/kernel-and-execution.md),
[identity model](../../02-architecture/identity-and-authorization.md),
[objects contract](../../06-contracts/ipc-and-objects.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
