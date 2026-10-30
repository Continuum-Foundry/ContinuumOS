# ADR-0025: Prepaid subtrees and quiescent process reaping

Status: Proposed. Date: 2026-10-30. Experiment: P3-09.

## Decision

Admit a deliberately small single-CPU supervision profile. A trusted static Rust
supervisor executes at CPL3; its children use the same checked syscall, loader,
capability, wait and eager-FP mechanisms as ordinary runtime clients. The kernel
owns admission and destruction, never a suspended Rust continuation. One owner
thread and priority zero remain mandatory. Equal round-robin preemption and a
maximum of eight owners bound fan-out's scheduling share; no hard CPU-time or
real-time guarantee follows.

Use prepaid **lifetime** subtree envelopes for this first profile. Bootstrap has
eight units including itself. A child's requested units include that child and
all descendants it may subsequently admit. Granting units deducts them from its
parent, and delegated units cannot be spent twice. Credits are not returned by
exit, close, free, receive or join during that epoch. This deliberately sacrifices
long-lived service-manager usability to keep retained memory, queued authority,
dead-parent lineage and restart churn charged without unsafe refund attribution.
Only trusted bootstrap after complete settlement can reset the epoch.

Each unit reserves at most 1 MiB lifetime allocated user backing (including
image/stack), 24 lifetime allocation records, one thread, 12 live local handles,
12 lifetime objects (two self objects plus ten creations), 16 live mappings,
one lifetime channel pair, four lifetime messages/8192 inline bytes, one active
wait and four lifetime timer creations. An existing mapping can be replaced after
unmap, within that limit. Public memory-object calls remain at most 64 KiB;
shared mapped pages do not create new backing. At most 1 MiB of such additional
mapped aliases fits sixteen mappings; this is separate from allocated backing.
Per-owner diagnostics are limited to four records/second, hence at most 32 across
an epoch. The experimental trace also has a 2048 validated-call lifetime ceiling
per task. The control owner receives its own complete unit, and global tables
retain unused owner/object/allocation capacity. Emergency handling uses existing
precharged kernel/IST storage, independently of these grants.

Caller identity comes from the current generation-bound task. Child claims must
match its principal/session, narrow grants, select service=false, and use the
fixed admitted memory ceiling. Only named immutable ISO modules are executable.
The initial child grants are its own process/thread handles; inherited handles
are unsupported and any nonempty inheritance request is rejected. There is no
ambient supervisor authority in an application, no elevated child priority and
no arbitrary ELF installation. P3-10 still owns policy/session validity.

## Lifecycle and recovery

Create loads and checks the image, allocates the context, address space and
object table, then publishes a Prepared instance. Start separately permits entry.
Tokens increase monotonically across bootstrap epochs and never wrap/reuse;
these are lookup IDs, not secret capabilities. Only the direct parent or trusted
supervisor may start, terminate or join an instance. A join only consumes a Reaped
exit record after all its descendants have been joined. Bounded child enumeration
lets the supervisor discover dead-parent descendants. No instance can be joined
while old execution or a kernel wait can still reach it.

Death first closes that subtree's admission and marks its descendants Stopping.
At the scheduler's quiescent kernel root, remove waits/completion state, release
registry handles/mappings/queued references, destroy the owned address space and
stack, clear the task slot, and only then publish Reaped with its exact code and
fault vector. Retained transferred objects keep their existing independent
reference lifetime. Dead-parent lineage remains until descendants settle.
Orphans are terminated, not adopted into a fresh identity or session.

The trusted user supervisor requests at most two retry permits, separated by
10/20 ms backoff; further retry requests are quarantined. Fresh creates still
require complete context validation and spend units. Even relabelled/manual
launches cannot exceed seven child admissions per bootstrap epoch. A supervisor
fault closes all privileged admission and kills its subtree. The experiment's
trusted bootstrap can restore a completely settled tree with a fresh session;
production bootstrap recovery and persistence are not implemented. Restarts
preserve no application work and never claim live state transfer.

## Alternatives and gate

Refunding resources on process exit is unsafe while another owner or queue still
retains an object. A future reusable quota ledger must explicitly attribute every
retained backing/reference to live ancestor accounts and test account retirement.
Larger thread counts, priorities, dynamic images, elastic quotas, persistent
restart policy and SMP require separate evidence. This conservative profile is
not the full planned M3 capacities or the 256 MiB whole-guest cold-idle target.

Acceptance requires actual CPL3 create/start/join/kill, healthy progress beside a
CPU hog, per-owner and delegated-budget exhaustion, identity/priority/inheritance
denial, parked-child and parent/supervisor death, retained-transfer teardown,
bounded retries and exact post-epoch inventories. Run all narrower feature
flavors and full M1 before declaring the story complete. See the
[supervision runbook](../05-delivery/phase-3/supervision-runbook.md).
