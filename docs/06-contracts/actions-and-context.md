# Draft contract: typed actions and scoped context

Status: version 0.1 design draft. R19. No permanent wire IDs or workflow engine are frozen.

P2-02's [shared hosted corpus](hosted-ui-fixtures.md) declares six bounded version-1
edit/search/list/launcher/settings action references and independent expected
states. P2-03 implements that hosted model; the JSON names convey no capability
authority, executable callback or production action-provider implementation.

## Action descriptor

An action has provider/component ID, action ID, protocol major/minor, input/output
schema IDs, required capability types/constraints, expected revision fields, effect
class, cancellation semantics, idempotency policy, preview support and optional undo/
compensation contract. Descriptor registration requires provider authority. The registry
does not permit a caller to invent a descriptor with elevated rights.

Example logical operation:

```text
documents.move@1(
    source: DocumentCapability,
    destination: DirectoryCapability,
    expected_document_revision: Revision,
    request_id: RequestId
) -> MoveResult | Conflict | AccessDenied | Unsupported
```

This is an illustrative signature, not an implemented file operation. Actual handles
are transferred or resolved by authorized IPC; a JSON object containing a number
does not become a valid document capability.

## Invocation envelope

Carry action/version, caller identity, correlation/request ID, plan/step ID if any,
typed payload, expected state revisions, deadline, service epoch and attached authority.
The executor validates it, but the provider also enforces its own object rights and
revisions. Reject stale epochs/revisions before mutation. Never rely on a preview
performed earlier as a substitute for execution-time validation.

Minimum outcomes: success with authoritative resulting revision, access denied,
unknown/unsupported action/version, invalid input, conflict, expired context,
quota/deadline/cancel outcome, unavailable provider and uncertain external result.
Undo tokens identify explicit provider-supported operations; they are not generic
permission to restore an arbitrary old document version.

## Context descriptor

Fields: logical context ID, source/provider, source revision, provenance, scope/purpose,
authorized audience, expiration/lease, permitted data/actions and size bounds. Resolve
data through narrowed authority. Redact secrets and exclude model-generated provenance
claims from trusted origin metadata. Recheck source revision/lease at use time.

Context is data. Text requesting new privileges or changing policy cannot authorize
execution. A model-produced plan is untrusted input until normal policy and capability
checks pass. Human UI and accessibility paths share the same state/rights rules.

## Plans and execution records

A bounded plan contains ordered steps/dependencies, action versions, exact targets,
expected revisions, requested authority, previews, deadlines and declared effects.
Bind any required user authorization to the reviewed operations/targets; a later
change requires revalidation. Do not require redundant approval for ordinary actions
already covered by authority/policy.

A result log records which steps were accepted, executed and durably completed,
result versions and supported compensation. Idempotency is owner- and retention-scoped.
Batch plans are not automatically atomic across providers. Cancellation cannot undo
an external send or irreversible delete by changing a status field.

## Reload integration

Action interface and state-schema versions are independent. Quiesce invocations using
the same finish/cancel/replay rules as other service calls. Carry result/deduplication
records across compatible replacement. A stale plan referring to old semantics or
resources is rejected or explicitly adapted, never silently reinterpreted.

## Acceptance fixtures

Invoke an edit/settings operation through button, shortcut and automation and compare
results/authority. Test fabricated context/handles, permission escalation, stale preview,
revision conflict, duplicate requests, unsupported action versions, cancellation after
commit, provider replacement and a malicious model proposal if inference is enabled.
