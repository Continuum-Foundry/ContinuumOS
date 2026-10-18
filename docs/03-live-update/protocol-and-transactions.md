# Update protocol and transaction state machine

Status: proposed design, not implemented. Wire details: [update contract](../06-contracts/update-wire-format.md).

## Actors

Host builder/CLI, bounded authenticated guest endpoint, package verifier/store,
coordinator, route/resource broker, old service, candidate service and health monitor.
The coordinator serializes conflicting transactions. MVP permits one active update
transaction globally; concurrency can be added only with dependency/ownership proofs.

## State transitions

| State | Work / durable record | Failure behavior |
| --- | --- | --- |
| RECEIVING | Bounded temporary content; no execution | Discard incomplete stage |
| VALIDATED | Verify hashes, manifest, authority, ABI/schema, dependencies, resource headroom | Reject; old generation unchanged |
| STAGED | Install immutable blobs; flush if persistent | Orphan blobs are GC candidates; no activation implied |
| QUIESCING | Pause new mutations; drain/cancel work; fence old writer leases | Timeout resumes old only after proving no candidate effects |
| CHECKPOINTED | Export/validate checkpoint and handle inventory; persist if durable | Retain old ownership; abort can resume it |
| PREPARED | Start candidate with restricted/read-only/test capabilities; import state; preflight; append/flush PREPARED | Kill candidate; resume old; record ABORT |
| DECIDED | Append/flush ACTIVATE_DECISION for target generation and epoch | Recovery must honor decision or explicitly transition to a new recovery epoch |
| ACTIVE | Broker publishes one route-table root for transaction/epoch; grants fenced write authority | Query/reconcile outcome; never run both writers |
| HEALTHY | Real liveness/semantic checks under bounded observation | Failure uses class-specific recovery, not automatic blind reversion |
| FINALIZED | Append/flush health/finalization record; ACK durable/ephemeral status | Lost ACK handled by transaction query |
| RETIRED | Stop old instance; release refs/leases; deferred cleanup | Failure quarantines resources and reports leak; never reuse unsafe memory |

Some L0 updates use a smaller in-memory transaction, but must still validate,
publish one resource generation, preserve old references until readers finish and
report failure accurately. Durable L0 updates use the same persistence principles.

## Activation and crash consistency

The durable decision precedes in-memory publication. A crash between them recovers
the decided generation, reconstructs candidates/resources and establishes its route
table before accepting clients. A crash before a valid decision recovers the old
generation. PREPARED alone never implies activation.

The broker's route-table-root publication is the single logical activation point
for a coordinated in-memory service set. It is not a distributed atomic transaction
over remote servers/devices. Writer authorization checks transaction epoch at the
broker/resource owner; a cached old endpoint must not bypass that fence. Transactions
involving resources without an enforceable fence are unsupported until a different
safe handover is specified.

The old service is quiesced before taking its authoritative checkpoint. New mutating
requests are buffered at the stable proxy under strict count/byte/time limits.
Reads either use a declared immutable snapshot or wait. No request can mutate old
state after the checkpoint while being omitted from candidate state.

## Requests and side effects

Each operation declares finish, cancel or replay semantics. Capture request ID,
result/deduplication status and relevant journal sequence. At-most-once processing
within a durable owner requires persisting that history consistently with the effect.
An external API without an idempotency/query protocol leaves uncertainty after loss.
Do not use an “exactly once” label for arbitrary external actions.

Candidates cannot issue external effects during preflight. After activation, new
user input may create irreversible effects even before health finalization.
Therefore the safest first live services are pure/read-mostly or broker-journaled
settings/app behavior. The health window does not magically make every change reversible.

## Multiple builds and disconnects

Every plan has transaction ID, source revision, base generation and target generation.
Reject an incorrect base. Supersede only queued or safely abortable pre-decision plans.
Once decided, finish/recover it before applying the next. Host disconnect does not
abort a decided transaction. `QueryStatus(txid)` returns the authoritative outcome
after reconnect. Duplicate activation messages return the stored result, not a
second activation.

## Timeouts and resource headroom

Use monotonic deadlines per phase and a total transaction limit. A deadline can
abort preparation, but cannot justify freeing code/resources still executing.
Budget old+new code, state copies, buffers and queued work before staging. Fail with
insufficient headroom rather than destabilizing the old instance.

## Required evidence

Fault inject before/after every journal flush, route publication, writer-fence
change and acknowledgement. Show one writer, coherent routes, preserved accepted
state, predictable recovery and unchanged boot identity for live paths. The recovery
spec enumerates post-activation behavior.
