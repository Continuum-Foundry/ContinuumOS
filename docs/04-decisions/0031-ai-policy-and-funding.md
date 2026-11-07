# ADR-0031: Deterministic AI policy, effect review and funding admission

Status: Proposed. Date: 2026-11-07. Owner-selected product policies; guest engine unimplemented.

## Context and decision

Sophisticated models and cooperating agents do not replace authorization,
source-use restrictions or accounting. Use a deterministic policy engine outside
model control. Every applicable mandatory restriction must pass; permissions,
destinations, effect review, resources, budgets and funding are separate decisions.
Authenticate policy issuers and constrain each issuer to its scope. Natural-language
rules are proposals until compiled/reviewed into versioned machine policy.

Routine authorized reads/drafts/reversible actions proceed within a task grant.
Owner choice: review the concrete final external send/publish, with exact scoped
standing rules available. Bind review to action, destination, payload digest,
resource revisions, policy epoch and expiry. Purchases, permanent deletion,
privilege/access changes and new-code execution have explicit gates.

One-shot review is also bound to task/effect/request identity and payer, and
consumed durably before dispatch. Replaying it with a new ID cannot repeat a send.
Standing rules separately specify repetition/count limits. Lost replies require
reconciliation of the original effect, not fresh approval-token reuse.

Build OS-bundled allowance support without inventing a funded amount. Recognize
OS sponsor; enterprise/department/team; device/fleet; workspace/project/app;
user total-use and separate personal-charge; task/session; parent/school/library;
grant/consortium sponsor; provider credit; network/energy/resource limits. Keep
issuer, scope, payer, meter, period, expiry and revision. Server-authoritative
sponsor ledgers prevent desktop counter resets from minting allowance.

Atomically reserve bounded worst-case usage against all applicable ceilings and
the selected entitlement before dispatch. Children share ancestor ceilings;
recheck before use, settle actuals idempotently, retain uncertain holds and record
overruns honestly. Separate units and total-use/personal-charge rules. Never bill
the user silently after a sponsored pool empties. Providers without bounded costs
need enforcement or explicitly accepted exposure; local cancellation cannot undo
remote charges.

## Alternatives and consequences

A model's judgment cannot enforce mandatory policy. A single minimum dollar cap
loses scope, meter and payer semantics. Post-hoc cost logging cannot protect
concurrent budgets. Full review of every harmless step undermines workflow value.
The chosen engine needs durable reservation/reconciliation, authenticated issuers,
price-bound adapters and distributed-ledger failure handling beyond the host oracle.

## Gate and reconsideration

B67/B68 independent policy fixtures, guest admission and adversarial effect tests;
B72/B73 remote billing semantics; B79 federation settlement. All critical boundary
cases must pass. The [28-case host reference](../../experiments/ai-policy/README.md)
models only in-memory budget semantics, not authorization or production finance.
See [draft contract](../06-contracts/ai-tasks-and-policy.md) and
[architecture concerns 1, 3, 13, 16 and 20](../02-architecture/ai-platform.md).

## Audit clarification

B81 owns durable intent/reservations before remote dispatch, crash/late/duplicate
settlement, period attribution and old-snapshot/issuer reconciliation; B68's
ephemeral admission cannot establish those properties. B82 separately qualifies
endpoint credentials. A verified quote-bound breach blocks new and queued work
even below budget caps. Unsupported mixed financial meters fail closed in the
single-funder host reference. No offline rollback-resistant allowance is assumed
without an authoritative anchor; data restore cannot mint funding or authority.
