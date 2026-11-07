# AI plan audit and corrections — 2026-11-07

Scope: requirements R20–R27, architecture, ADR-0030–0033, task/policy contracts,
M21/M23–M26, backlog graph, qualification and retained host budget reference.
Local agent self-review, not an independent security assessment, guest acceptance
or approval of a model/runtime port. The owner's conditional PR exemption is
recorded with the exact payload/destination in the handling record before upload.

## Findings and fixes

| Finding | Correction and verification |
| --- | --- |
| Remote/thin-client delivery transitively required successful native Gemma through B69/B70/B71; public also required private implementation | Remove those unconditional edges. B74 accepts a named profile only after its enabled routes pass B39/B72/B73. Local, private and public paths can qualify independently after M5 and their own prerequisites. Full M23 program coverage still needs all planned route classes; partial profiles cannot claim complete provider coverage. Dependency and qualification-matrix negative tests cover this. |
| Paid dispatch lacked an explicit durable reservation/journal prerequisite | Add B81 and make both remote adapter gates depend on it and B69. Require crash-safe intent, uncertain dispatch, duplicate/late settlements, period attribution and restore reconciliation before spending. B68 remains bounded ephemeral admission; its host model is not a persistent ledger. |
| Account password handling was implicitly treated as a remote credential vault | Add B82 for authenticated endpoint/credential brokerage, session-only versus protected persistent credentials, rotation/logout/recovery and service identity boundaries. Both remote adapter gates require it; encrypted homes or existing account hashes do not establish this service. |
| M21 described persistent user off before persistence/accounts existed | Make M21 explicitly a development-only ephemeral profile with session off and no automatic reactivation/warming. Persistent personal off belongs to B71/M23 after storage/accounts. |
| B39 needed model integrity before later B70 ordinary package delivery | Require a bounded exact-hash authorized development fixture and tested parsing before any M21 load, with no dynamic download/remote code. B70 owns production package/trust/update lifecycle. No trust bypass or circular package prerequisite. |
| Existing package/transfer bounds cannot be assumed to fit useful Gemma | Current 64 MiB files/objects and 2 GiB package memory remain. B67 must define bounded sharding/streaming and any versioned mapping/resource extension; B39 proves the guest subset before load and B70 qualifies ordinary delivery. No inferred large-file API, oversize v1 blob, memory waiver or frozen-SDK bypass. Legacy-limit and planning-gate mutation tests cover accidental relaxation. |
| Mixed-source derivation and effect receipts left trust semantics underspecified | Every contributing source must permit the derived disclosure; revocation invalidates affected context/index/checkpoint material. Bind receipts to authenticated owners and distinguish provider acceptance, confirmed delivery and unknown. A signed/transport-authenticated response does not prove an arbitrary external outcome. |
| Recovery could restore old allowance/grants or forget unresolved liabilities | Separate preferences/tasks from authority and financial state. B81 reconciles old snapshots against issuer/ledger epochs before new remote spend; no invented offline anti-rollback guarantee without a trusted external anchor. Carry unresolved liabilities across accounting periods. |
| Off/strict-offline controls conflicted with automatic receipt reconciliation | Off allows only bounded already-authorized receipt reconciliation with no new inference/effects. Strict offline prohibits even those outbound queries, leaving status unknown and holds intact until explicitly permitted. Re-enable never automatically dispatches old work. |
| Final review lacked one-shot consumption semantics | Bind approval to task/effect/request identity and payer, consume durably before dispatch and reject reuse with a fresh ID. Standing rules carry separate repetition limits. B67/B81/B74 own the protocol and future crash/replay tests. |
| Atomicity across independent consortium issuers was ambiguous | Scope atomicity to the authoritative ledger/prepaid grant; obtain every required hold before dispatch, journal partial reservations and fail closed on partitions. B79 must qualify reconciliation without assuming global ACID or treating lease expiry as undo. |
| Host budget reference allowed unfunded foreign/credit meters and missed quote overruns below caps | Reject positive financial meters unsupported by the selected funder. Record actual usage and close new/queued admission after any verified quote-bound violation, including an unexpected charge on an otherwise free local call. Bound route inventory. Six regressions failed before the fix and pass afterward; original 22 remain. |

## Acceptance ownership and limits

The [roadmap](ai-roadmap.md) and [qualification prerequisite fixture](../06-contracts/examples/ai-qualification-gates.json)
define conditional profile dependencies, including external tools used with local
inference. The fixture is planning data, not a runtime authorization or certificate.
AI remains outside B35/M8, B56/M14, B66/M22 and B58/M15. R01–R19, the 256 MiB
model-unloaded target, desktop/frame/input/reload targets, accepted evidence,
P4-02 next, original kernel/ISO and independently usable conventional controls remain.

The ordinary-task 95% threshold remains a proposed starting point. Critical checks
must all pass, with at least three trials per declared task/configuration and human
quality/usability review; no statistical reliability or security guarantee follows.
Exact CPU/ISA/runtime/model memory, native cryptography and protected credential
storage, authoritative funding services, provider contracts and distributed
federation still require their named experiments. No calendar date is established.

## Reproducible checks

```text
node audit/check-docs.cjs
node --test audit/check-docs.test.cjs
node --test experiments/ai-policy/budget-policy-reference.test.cjs
pwsh -NoProfile -File audit/validate-contracts.ps1
node audit/check-governance.cjs
node audit/check-repository-hygiene.cjs --quiet
git diff --check
```

Run hygiene after staging the exact intended payload, retaining the installed
pre-commit check. Full generated output, initial failing budget regressions and
audit JSON stay under ignored local `outputs/`/`work/`. No historical acceptance
bundle is rewritten and no kernel/guest result is claimed for these host checks.
