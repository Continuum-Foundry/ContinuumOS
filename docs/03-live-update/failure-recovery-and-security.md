# Failure recovery and update security

Status: proposed. R10–R13, R17. Companion: [transaction state machine](protocol-and-transactions.md).

## Failure table

| Failure | Required response |
| --- | --- |
| Host compilation fails | Keep guest untouched; show compiler diagnostics |
| Transfer truncates/hash fails | Reject temporary content; no candidate execution |
| Unsupported ABI/schema/feature | Reject before quiescing where possible; report exact mismatch |
| Candidate fails to start/import | Stop candidate, reclaim safe resources, resume old before decision |
| Old process crashes before a valid checkpoint | Recover its latest valid checkpoint/journal; report any uncheckpointed work loss or uncertain requests; do not claim lossless update |
| Quiescence times out | Abort only if old ownership is intact; never force-free active code |
| Host disconnects before decision | Apply configured staging timeout; keep old working |
| Host disconnects after decision | Guest completes/recover transaction; host queries status later |
| Coordinator crashes before decision | Supervisor reconstructs old generation; discard candidate authority |
| Coordinator crashes after decision | Recover decided epoch/route graph from durable records |
| Candidate fails after activation, reversible class | Fence candidate, recover compatible latest state, publish recovery epoch |
| Candidate fails after irreversible effects | Preserve journal/data, use forward fix or explicit recovery UI |
| Old instance will not retire | Revoke/fence authority, retain unsafe-to-free resources, report bounded quarantine |
| Disk full / flush failure | No durable-success ACK; retain recoverable old/decided records and signal storage error |
| Display/shell fails | Stable diagnostics and rescue path; clients can resubmit surfaces |
| Kernel fails | Panic diagnostics/rescue boot; not counted as successful no-reboot reload |

## Recovery ownership

The supervisor restarts a failed coordinator from immutable trusted content. A
minimum recovery endpoint must not share the updated process. The broker reconstructs
routes only after validating activation records and required blobs. Missing content
does not cause silent mixed-version boot; select a validated last good generation or
enter rescue mode with an explicit reason.

Persistent mode and ephemeral mode use the same logical states, but only the former
survives power loss. The UI and protocol say which mode succeeded. Durable package
availability and migration checkpoint references are checked before ACK.

## Post-activation recovery

A recovery action is a new epoch/transaction, not erasure of history. Retain current
accepted mutations and result/deduplication records where supported. If reverse
migration is unavailable, keep the data readable by a compatible recovery tool and
block unsafe writes. Do not hide loss behind a “rolled back successfully” message.

For the first demo, use reversible theme changes and an editor logic change with
identical checkpoint schema. Then test a deliberately incompatible schema and show
clean rejection. Irreversible service migrations are later tests, not required for
the first compelling live experience.

## Update authority

Only explicitly enabled development sessions can initiate dev updates. Authenticate
and bind transaction messages to boot/session identity, sequence and content digest.
Reject replay, downgrade of required protocol/security fields and authority escalation.
The host chooses artifacts; guest requests cannot trigger arbitrary host compilation
commands or filesystem access.

Candidates/migrations run with restricted authority. They cannot publish routes,
change their own permission grants, mutate authoritative old state, or perform real
external side effects during validation. Health tests include semantic correctness
and resource limits, not just a successful heartbeat.

## Distribution updates

Do not reuse dev credentials or transport trust as a public update system. Use a
separate root/repository design with reviewed signatures, metadata expiry,
anti-rollback state and key rotation/recovery. A legitimate local rollback needs an
explicit authorization policy distinct from accepting an attacker's older package.
TUF is the reference, not an implemented dependency today.
[S60](../01-research/sources.md#s60)

## Fault-injection evidence set

For every phase record boot ID, uptime, transaction ID, old/new epochs, route root,
accepted request IDs, document revision, journal sequence, package references and
resource counts. Inject app/service/coordinator crash, transport loss, deadline,
allocation failure and disk fault at deterministic hooks. Recovery passes only if
both continuity claims and data invariants match the declared update class.
