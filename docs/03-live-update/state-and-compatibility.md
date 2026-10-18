# State, ABI and compatibility rules

Status: proposed. R05, R07, R14.

## Types of state

| Type | Example | Transfer policy |
| --- | --- | --- |
| Durable user state | Saved document | Authoritative storage record; code rollback must not silently revert it |
| Unsaved logical state | Editor buffer, selection, undo history | Versioned checkpoint plus revision; explicit size/privacy limits |
| Session state | Window geometry, focus, workspace | Stable IDs owned by session service |
| Derived state | Glyph atlas, layout cache | Rebuild; optional compatible cache hint only |
| Resource authority | Open file/socket/surface lease | Regrant/transfer through authorized broker, never trust a numeric value |
| In-flight execution | Rust future, stack, native closure | Drain/cancel/restart from a logical operation boundary |
| Device state | DMA rings, GPU objects | Device-specific quiesce and reconstruction; not ordinary serialization |
| Secret state | TLS/private key material | Secret broker or explicitly protected lifecycle; exclude from normal checkpoints |

## Checkpoint envelope

Use schema ID/version, producer component/build, source epoch, logical revision,
transaction ID, payload length/digest, resource-reference inventory and migration
history. Validate before allocating its full claimed size. Proposed MVP maximum
checkpoint is 16 MiB, with an explicit error or streamed document reference above
that bound. A large editor document cannot silently be truncated to fit.

Text editor checkpoint: document ID, full unsaved content or a durable revision plus
validated edit tail, cursor/selection using logical text positions, viewport,
unsaved status and supported undo metadata. Preserve revision monotonicity. Do not
mix bytes from different document revisions during export.

## Migration functions

Migrations explicitly map `(schema_A, version_n)` to `(schema_B, version_m)` and are
bounded/deterministic given declared inputs. Run candidate migration with constrained
authority and without external side effects. Validate output invariants, resource
references and length limits before publication.

Initially support identical schema and explicit adjacent migrations only. No
automatic general chain search or magical inferred struct conversion. Fixture tests
cover minimum/maximum/empty/old/corrupt states. Removing a field defines a default
or intentional loss policy. User-document loss requires an explicit product decision,
not a successful migration return code.

Rust memory layout can change across builds and does not preserve object identity.
Even `repr(C)` does not make an arbitrary object graph a stable serialized format.
Use explicit encoding and versions. [S02](../01-research/sources.md#s02)

## Independent version axes

Track package release, protocol major/minor, required feature bits, state schema,
kernel ABI, scene schema, resource-handle protocol and CPU features independently.
Semantic version strings alone cannot express the complete compatibility relation.

Within a protocol major version, unknown optional fields can be skipped under
defined length rules; unknown required features reject. Maintain explicit supported
version ranges, not a promise that all old versions always work. A coordinated
dependency update prepares the whole required set and publishes one route graph.

## Rollback and forward recovery

Before activation decision, preserve the untouched old state and resume it after
aborting. After activation, rollback is safe only if the old implementation can
understand current state or a tested reverse migration/replay scheme exists and
effects permit reversal. A checkpoint from five seconds ago would discard accepted
typing or file writes unless events are replayed correctly.

Use expand/contract schema changes where practical: new code can read old and new,
write a backward-compatible form during the support window, then later remove old
support. Record the irreversible boundary. For a failed irreversible change, use a
forward-fix generation or recovery UI; do not silently restore old state.

Linux livepatch state tracking illustrates why retaining old code is not sufficient
when state semantics change. [S18](../01-research/sources.md#s18)

## Resource references

Serialize logical object IDs plus expected type/rights/epoch, not process handle-table
slots. Broker checks ownership and narrows authority when regranting. Reconnect
secrets and devices through their own protocols. On reboot all volatile IDs require
reconstruction from durable logical references; stale resources fail explicitly.
