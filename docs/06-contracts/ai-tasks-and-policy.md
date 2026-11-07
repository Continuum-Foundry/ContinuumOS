# AI tasks, routes, policy and effect records

Status: Proposed, unfrozen semantic draft, 2026-11-07. No syscall/message numbers,
wire encoding or SDK extension are assigned. Review under B67 before independent
codecs, bounded parsers and guest clients. Existing frozen SDK subset is unchanged.
Extends [typed actions/context](actions-and-context.md); rationale and failure
cases are in [AI architecture](../02-architecture/ai-platform.md) and
[ADR-0031](../04-decisions/0031-ai-policy-and-funding.md).

## Ownership and records

All lists, strings, graph depth, context bytes, reservations and history have
declared profile limits. Reject overflow, unknown critical fields/versions and
unsupported semantics. These are required field meanings, not Rust layouts.

| Record | Required semantics and owner |
| --- | --- |
| Task | Coordinator owns ID/schema, authenticated user/service principal and tenant/session, goal, parent/child links, allowed resource references/effects/routes, policy revision, deadline, status, logical progress, artifacts, expected revisions, receipt/reservation references and unresolved effects. |
| Grant | Trusted broker owns issuer, subject, audience/resource, operations, purpose, scope, epoch, expiry and delegation ceiling. Serialized references need current reauthorization; names/IDs alone confer nothing. |
| Context | Resource owner owns source identity/revision/provenance, consumer/purpose, access and permitted reading/indexing/storage/training/onward use, expiry/retention, bounded excerpt/reference and derived-data lineage. |
| Route configuration | Registry owns destination class (device/private/public), approved endpoint/tenant, exact model and capability manifest, runtime/configuration identity, data-handling profile, price-bound version, limits and eligible funding sources. Model-supplied URLs cannot become routes. |
| Effect proposal | Planner supplies typed action/input, target/destination, expected revisions, idempotency/correlation key, dependencies, predicted effects/cost and supported compensation. Executor/resource owner validates at use time. |
| Final approval or standing rule | Policy broker owns authenticated approver/issuer, task/effect/request identity, action class, destination, payload digest or bounded rule predicates, source/target revisions, data scope, selected payer, policy epoch, cost ceiling and expiry. One-shot approvals are durably consumed for exactly that effect; standing rules specify repetition/count limits. Changed reviewed semantics invalidate approval. |
| Preference | Memory service owns subject/scope, structured value, source/provenance, confidence, confirmation time and corrections/exclusions. Never authority, budget or executable policy. |
| Policy decision | Deterministic broker owns matched issuer/rule revisions, principal/resource/action/route, result/reason/remedy, obligations and expiry. Prompt text is not an executable rule or trusted policy source. |
| Budget/funding | Authenticated issuer owns meter/unit, limit/entitlement, period/reset identity, subject/scope, payer, start/expiry, authorized revision and authoritative ledger location. Separate total-use and personal-charge scopes. |
| Reservation | Accounting broker owns immutable request/context digest, ancestor task chain, selected funder, worst-case amounts in distinct meters, every matched ceiling, price/policy revisions, dispatch status and expiry/reconciliation obligations. |
| Receipt | Resource owner/provider adapter owns request/effect ID, authoritative resource revision or remote correlation, outcome certainty, artifacts, billable actuals in original units, settlement identity and remote deletion/cancel evidence where available. |

## Task and effect transitions

Tasks expose waiting, executing, paused, awaiting-review, verifying, completed,
partial, blocked, cancelled and outcome-unknown with per-effect certainty. A
prepared draft is prepared for review, not a completed send. Every terminal label
is consistent with outstanding effects; cancellation may leave an unresolved
remote effect that still needs reconciliation.

Pause closes new step admission and checkpoints supported logical state. Cancel
requests cooperative stop. Termination stops an owned local worker. Revocation
blocks future mediated use; it cannot undo committed effects or recall data.
Record these separately. Lock pauses ordinary interactive work; a bounded
background grant may continue only its permitted work. Logout ends personal
interactive grants. Resume after reboot/replacement rebinds references through
current policy and never restores raw handles, credentials, stacks or KV caches.

Before commit validate current principal/session, grant, context provenance/use,
resource revisions, endpoint, effect approval, policy and budget. Resource owners
serialize mutations under their writer/revision rules. Duplicate keys are scoped
to the owner and retained according to a declared horizon. Reconcile a potentially
committed non-idempotent effect before retry; otherwise report unknown. No universal
exactly-once or undo contract is implied.

Final external send/publish review concerns a consequential user effect such as
sending a message, publishing an artifact or transferring a result to a recipient.
Already authorized model/tool transport follows its explicit route/context/use
grant and policy; it does not need a send-review prompt for every inference packet.
That grant still cannot authorize unrelated publication or additional disclosure.

A one-shot final approval cannot be replayed with a fresh request ID to repeat a
send. Consume it durably with the bound effect/dispatch intent before external
dispatch; after a crash reconcile that effect instead of minting a new approval.
Standing rules are separate scoped delegations with explicit repetition and budget
ceilings, not reusable one-shot tokens. B67 defines these semantics; B81/B74 test
consume/crash/replay behavior. M21 rejects unsupported external effects outright.

## Policy and resource admission

Evaluate mandatory restrictions from resource owner, organization/tenant, device,
workspace/app, user and task, plus applicable source/provider conditions. A rule
may restrict only its issuer's authority; a permissive user preference cannot
override a mandatory owner/organization restriction. Incompatible or unavailable
mandatory decisions deny admission with an actionable reason. Funding grants
capacity, never access permission. Check all tool/search/URL/log disclosures too.

Reserve worst-case usage atomically against the selected funder and every
applicable mandatory ceiling before dispatch. Concurrent children share prepaid
ancestor ceilings. Distinct currency, credits, tokens, time, memory, energy and
network units remain distinct unless an explicitly versioned conversion applies.
Recheck policy, grant, entitlement, prices and resource availability at dispatch
and effect boundaries. Never switch to a personal payer or disclosure destination
silently when a pool empties.

State transitions: reserved → dispatched → settled; reserved → released only
before dispatch; dispatched → uncertain retains holds until authoritative
reconciliation. Duplicate admissions cannot dispatch twice; settlements are
idempotent, conflicting receipts reject. An actual overrun is recorded honestly
and blocks further admissions; it is not clipped to the estimate. Bound terminal
deduplication storage and deny new work rather than silently evicting protected
records. Retention must cover the adapter's late-receipt/retry horizon.

Distributed reservation, durable log/replay, leader failure, clock/period rollover,
delayed charges and cross-member settlement require separate implementation and
failure acceptance. OS sponsorship uses a server-authoritative entitlement ledger.
If a provider cannot bound committed charges, require provider-side enforcement
or explicit bounded exposure policy; a local kill switch cannot guarantee a remote
financial hard cap. Local inference still needs compute/resource admission.

Atomic reservation is within an authoritative ledger or already prepaid grant.
Across independent issuers, durably obtain every required reservation before
dispatch; a partial reservation or network partition cannot authorize a partial
effect. Journal partial holds, reconcile/release them only when safe, and otherwise
keep them visible. Do not assume a global ACID transaction across consortium
members or treat lease expiry as undo of a remote commitment. B79 qualifies the
chosen cross-member protocol and failure policy before federated spending.

## Federation and interoperability

Federated task envelopes additionally bind originating/member trust domain,
receiving audience, tenant mappings, scoped delegation chain, source/derived-data
handling, result provenance and the member responsible for each effect and bill.
Validate each hop; peer messages and summaries remain untrusted content. Member
departure revokes future mediated access and preserves outstanding settlement,
retention and deletion obligations with honest confirmation status.

MCP tools, A2A tasks, OAuth and SPIFFE are candidate adapters/patterns, not native
authority grants. Pin supported protocol versions/extensions at adapter gates;
an evolving task draft cannot silently become a frozen OS ABI. API compatibility
labels do not substitute for semantics, cancellation and data-policy conformance.

Global/workspace/route off persistently denies new AI work and stops observing,
learning and warming within that scope; report local termination/unload and
cooperative remote cancellation separately. Protected reconciliation of existing
effects may continue without new delegation. Re-enabling does not automatically
dispatch queued old work. A managed nonpersonal service has its own disclosed
identity/policy and cannot masquerade as a user's disabled assistant.

Control names must distinguish inference locality from network activity. Local
inference only excludes remote model routes; any permitted web/tool access stays
visible and separately governed. Public-cloud off excludes public model/hosted-tool
disclosures, without silently enabling a private endpoint. Strict offline AI also
denies AI-initiated outbound tools/search/fetches. Resource-owner revocation,
organization/device/parent rules, exhausted budgets, trust expiry and resource
pressure can restrict or suspend eligible AI work; none grants permission to evade
the user's persistent personal-assistant off choice.

## Audit refinements: qualification, persistence and trust

The [qualification prerequisite fixture](examples/ai-qualification-gates.json)
is planning data only. B74 is base profile acceptance; every enabled model route
requires its B39/B72/B73 qualification and evidence, with at least one route.
Remote tools require B53/B54/B81/B82 and tool-specific conformance even with local
inference. All planned route classes/configurations are required before claiming
full M23 program coverage; profile-specific acceptance may precede that coverage.

Before any remote effect or billable inference, B81 durably records task/effect
intent, idempotency/correlation, policy/price/period bindings and reservations.
Persist a dispatch-intent marker before crossing the external boundary; after a
crash the result may be unknown even if no acknowledgement was journaled. Recover
and reconcile before retry/release. Bind amounts to their original accounting
period/basis and carry unresolved liabilities across rollover. Backup restore must
not recreate grants, credits or released holds. Reconcile issuer/ledger epochs
before new remote admissions; without a trusted external anchor no offline
anti-rollback financial guarantee is made. A quote-bound breach halts new/queued
admission even when aggregate caps still have room; record actual usage honestly.

B82 owns remote endpoint authentication, audience-bound credential acquisition,
rotation/revoke and key/secret handling. Declare session-only versus tested protected
persistence. Session-only credentials are not checkpointed; resume may require
reauthentication. Account-password hashes, TLS alone, ordinary file backup and
an assumed encrypted home do not establish a credential vault. Managed service
credentials have their own issuer, scope, unlock/off policy and offboarding.

Derived outputs inherit the intersection of all contributing source-use/audience
restrictions, including summaries, embeddings, tool arguments and checkpoints.
Reject or omit restricted contributions when policies conflict; a model cannot
declassify them. Recheck current rights before disclosure; revoke affected caches/
pending material when a source expires. Keep a minimal tombstone/receipt when
needed to prevent backup resurrection without retaining deleted source content.

Receipts bind authenticated resource-owner/provider identity, request ID, revision
and the precise observation. Provider-accepted is not confirmed recipient delivery;
transport authentication is not proof of every business outcome. An untrusted
adapter/model claim is not an authoritative receipt. Preserve unknown/partial
states when independent owner evidence is unavailable.

Off permits only bounded, already-authorized reconciliation of existing effects,
with no new inference/delegation/effects. Strict offline also denies outbound
receipt/status queries: retain unknown outcomes and holds until policy explicitly
permits reconciliation. Do not auto-resume old work on re-enable. M21 offers only
ephemeral session controls and a fixed verified development fixture; persistent
personal controls and ordinary model delivery wait for B71/B70 and M23.
