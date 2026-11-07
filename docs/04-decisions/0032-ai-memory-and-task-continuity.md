# ADR-0032: Editable preference learning and verifiable portable tasks

Status: Proposed. Date: 2026-11-07. Owner-selected learning policy; durability unverified.

## Context and decision

The assistant should reduce repeated setup and preserve useful work across the
OS's supported live changes. Memory, permissions, chat history and task progress
have different owners and lifetimes.

Automatically learn ordinary structured preferences in authorized interactions;
provide an editable/deletable/excludable memory view and learning-off controls.
Retain scope, provenance, confidence, confirmation time and corrections. Exclude
secrets/sensitive attributes by default; separate personal, work and tenant memory.
Learned material cannot authorize actions, payers or policy. Fine-tuning requires
separate authorization. Retrieved content remains untrusted after repetition.

Separate durable task records from preferences, transcripts and indexes. Prefer
source references/revisions; save only permitted bounded content. Proposed defaults:
completed personal task metadata/receipts for 30 days, session-only transcripts
unless history enabled, preferences until corrected/deleted/source access expires.
Keep unresolved effect receipts/holds until reconciliation or explicit documented
resolution; disclosed enterprise holds may override ordinary retention. Delete
derived indexes too and distinguish remote deletion requested from confirmed.
Persistent storage depends on accepted protection; no encrypted-home claim is made.

Checkpoints contain logical steps, grants by reference, artifacts, revisions,
receipts and reservations, never raw handles/stacks/credentials/KV-cache authority.
Reauthorize on resume. Verify completion from resource-owner state and receipts.
Revision checks and scoped idempotency govern concurrent edits/retries; uncertain
non-idempotent effects require reconciliation. Pause/cancel/terminate/revoke differ.
Lock pauses interactive work unless explicitly delegated; logout ends personal
interactive grants. Reboot recovery and compatible live replacement are separate.

## Alternatives and consequences

Explicit-save-only memory contradicts the owner's preference. Unbounded transcript
retention increases exposure and fails the baseline. Model-internal state is not
a portable task ABI. Replaying uncertain writes can duplicate external effects.
The selected design adds deletion lineage, migration and outcome-reconciliation
work but supports continuity without tying user work to one model provider.

## Gate and reconsideration

B69 depends on durable storage/accounts/session recovery; B71 supplies native
controls; B74 requires 100 compatible replacement cycles plus separate power-loss,
poisoning/deletion/restore/lock/uncertain-effect cases. Proposed 30-day retention
needs product review against operational reconciliation and applicable obligations.
See [architecture concerns 4–8 and 10–12](../02-architecture/ai-platform.md),
[task contract](../06-contracts/ai-tasks-and-policy.md) and [roadmap](../05-delivery/ai-roadmap.md).

## Audit clarification

Every contributing source must permit derived use/disclosure; revocation invalidates
affected caches and pending checkpoint content. Ordinary data restore cannot restore
authority or financial credits: rebind grants and reconcile B81 against issuer state.
Keep only minimal unresolved receipts/tombstones where required. Owner receipts
distinguish provider acceptance from confirmed delivery. Strict offline blocks
remote status reconciliation too and retains unknown/held outcomes. B82 session-only
credentials require reauthentication after resume; persistent secrets need tested
key/storage protection. M21 session off is ephemeral; M23 personal off is persistent.
