# ADR-0033: Separate enterprise federation, model cooperation and server execution

Status: Proposed. Date: 2026-11-07. Owner requested both federation and cooperating models.

## Context and decision

An enterprise consortium may expose inference from independently administered
members while models cooperate on a task. Trust federation, orchestration and a
native data-center OS are three separate capabilities.

Members retain independent trust roots, tenants, data/memory, keys, policies and
ledgers. Federate explicitly selected services/artifacts with scoped audience-bound
delegation and source/derived-data disclosure rules. Authentication is not resource
authorization. Offboarding revokes future mediated access and reconciles pending
effects, bills and deletion obligations; it cannot recall received information.

Cooperating models use a bounded task/role graph and attenuated context/grants,
prepaid ancestor quotas and one coordinator's task record. Resource owners control
effects. Peer output is untrusted; model votes never confer authority or prove
completion. Compare benefit/cost against a single-model baseline. No shared
cross-member cache, blanket data pooling or federated training is implied.

Evaluate MCP/A2A adapters and OAuth/SPIFFE patterns with pinned conformance scopes;
they do not replace native policy. Keep tightly coupled GPU collectives within an
appropriately secured cluster behind service boundaries. Enterprise BYO Gemma-like
and larger models require configuration/license/capability qualification.

Native original-OS data-center execution gets B80/M26 feasibility only. CPU/SMP,
GPU/runtime/drivers, server networking/storage, fleet trust/updates, availability
and operational evidence need a separately approved program. Existing Linux
serving runtimes cannot be assumed to run on this kernel.

## Alternatives and consequences

A common unrestricted trust/data pool weakens member boundaries. Treating model
cooperation as consensus proof fails effect verification. Requiring a native server
stack before private endpoints delays useful enterprise integration unnecessarily.
The selected design preserves expansion paths with explicit operational costs.

## Gate and reconsideration

B75/B76/M24 qualify cooperation; B77–B79/M25 use two independent trust domains,
partition/revoke/offboarding/tenant isolation and budget failures. B80/M26 produces
an evidenced server proposal or stop/replan, not a server delivery claim. No AI
track becomes a prerequisite for existing desktop releases.

See [architecture concern 19](../02-architecture/ai-platform.md),
[route survey](../01-research/ai-model-and-runtime-survey.md),
[roadmap](../05-delivery/ai-roadmap.md), [SPIFFE S105](../01-research/sources.md#s105)
and [A2A S104](../01-research/sources.md#s104).

## Audit clarification

Cooperation accepts any qualified route combination, without compulsory private
inference. Federation integration B78 explicitly owns enrollment, authenticated
policy issuers, tenant/audience mapping, key rotation/revocation and offboarding
on top of B82; M7 local accounts do not imply federated network identity. B81/B79
own durable local and cross-member settlement failure acceptance respectively.
