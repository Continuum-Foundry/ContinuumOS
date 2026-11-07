# AI platform amendment and delivery gates

Date: 2026-11-07. Status: owner-requested product direction incorporated into the
plan; technical designs remain Proposed. All new broad tasks are Not started.
The retained host budget reference passes its own behavioral cases only. It is
not completion of B68 or evidence of a guest AI implementation.

## Scope and preserved baselines

AI becomes a native platform capability with an assistant available by default in
an AI-enabled product profile. Users can turn their assistance off. Presence does
not require a resident model, continuous observation, cloud access or a funded
account. Gemma is the default local model family; GPT, Claude, Grok and Gemini are
the public flagship options. Private organization/consortium endpoints form a
third destination. Each configuration must qualify; this plan promises no exact
model version, commercial entitlement or native runtime port.

R01–R19, M8, M14, M22 and M15 exits, current evidence, and P4-02 as the next
implementation story stay unchanged. R20–R27 are conditional AI-profile
requirements. B39/B67–B82 do not block those existing releases. The original
kernel, native desktop, ISO, live-update and conventional recovery goals remain.
The 256 MiB model-unloaded cold-idle target includes dormant AI services and
controls. Separately qualify all local-resident, active, actual-default-boot and
loading/replacement peaks; machine capacity is not an idle allowance.
The 2 GiB reference VM need not support local Gemma residency. Local-profile
capacity and supported CPU/backend ISA are qualification outputs; current x87/SSE
context support does not imply AVX/XSAVE support or a guest runtime port.
Current package/update drafts also retain 64 MiB file/object and 2 GiB package-
memory ceilings. B67 defines bounded sharding/streaming and any versioned memory/
mapping extension; B39 must prove the needed guest subset before load. B70 later
qualifies ordinary delivery. Never enlarge old parser limits or frozen SDK IDs
silently; see the [package gate](../06-contracts/package-manifest.md#ai-artifact-and-memory-profile-gate).

## Ordered gates and earliest starts

Milestone numbers identify tracks, not a universal serial order. Calendar delivery
dates remain unestimated until exact-target integration and workload measurements
support them. Planning and independent host fixtures may start now without
displacing P4-02. These are broad backlog entries; decompose P5–P26 stories only
at the owner's milestone-decomposition request, before implementation.

| Gate | Broad work and accountable area | Earliest prerequisite | Required outcome |
| --- | --- | --- | --- |
| Contract preparation | B67 AI/SDK; B68 Security/Accounting | Existing typed-action and supervisor contracts | Independent bounded task, effect, routing and budget fixtures; adversarial policy cases. B68's guest subset must pass before B39 dispatch. |
| M21 local qualification | B39 Runtime/AI | B06/B21/B37/B67/B68; first M5 state-preserving guest reload | Exact Gemma artifact/runtime/backend on original guest; scoped typed proposals, off/crash/unload, denied escalation, desktop responsiveness, 100 compatible replacement cycles and complete resource accounting. Development-only ephemeral progress and session off; explicit invocation, no auto-warming/reactivation. Authorized fixed-fixture digest/parse checks precede any load; B70 handles ordinary delivery later. |
| Durable personal foundation | B69 Data/Lifecycle; B70 Packages; B71 Desktop/Accessibility; B81 Accounting | M6 persistence, M7 accounts, M9 exchange, M10 verified packages and M11 session/resource services as named in backlog | Separate editable preference learning, durable tasks, verified package lifecycle, native review/stop/memory UX and independent recovery. |
| M23 personal AI destinations | B72 Private; B73 Public; B74 QA; B82 Credentials | Base B74 plus every enabled route/tool gate below; remote routes require B53/B54/B81/B82 | Named profiles qualify independently, without a native Gemma or private-route dependency for public/thin clients. Full M23 coverage still needs all planned route classes and declared flagship configurations; a partial profile cannot claim complete program/provider delivery. |
| M24 cooperating models | B75 Orchestration; B76 QA | B67/B68/B69/B81 and every selected worker/tool route; aggregate exit after B74 | Bounded role/task graph, scoped subdelegation, parent budgets, deterministic effect verification, partition/cancel/duplicate/compromised-peer cases; demonstrate benefit against single-model baseline. |
| M25 enterprise federation | B77 Identity/Policy contracts; B78 Federation; B79 QA | B77 host contract study can begin now; guest integration needs B69/B72/B75/B81/B82 and explicit enrollment; aggregate exit after B76 | Two independently administered trust domains, tenant isolation, selected service/artifact sharing, residency/disclosure/retention, offboarding/revocation and independent ledgers. |
| M26 native data-center feasibility | B80 Platform/Research | Contract work B67/B77; independent of desktop AI deployment | Gap/cost/license/runtime/hardware study and bounded exact-target probes; propose a separate server program or evidenced stop. This is not a delivered server OS. |

## Conditional route qualification

The [planning prerequisite fixture](../06-contracts/examples/ai-qualification-gates.json)
is checked with the backlog graph; it is not a production authorization/certificate.
B74 is common profile acceptance. A named AI-enabled profile must have at least
one qualified model route and pass every enabled route and external-tool gate.
Do not mark the full M23 program complete from a narrower profile's acceptance.

| Enabled capability | Additional gate beyond B74 | Required dependency/evidence |
| --- | --- | --- |
| On-device Gemma | B39 | M5 live-reload prerequisite, native ISA/runtime/model and resource qualification. No networking or persistent remote credential dependency for strict-offline use. |
| Private inference | B72 | B69/B70/B81/B82/B53/B54: durable intent/reservations, qualified packages, endpoint/credential brokerage, network/time/trust; no B39 dependency. |
| Public flagship inference | B73 | Same remote foundations, independently qualified actual provider configurations; no B39 or B72 dependency. |
| Remote tools with any model | B53,B54,B81,B82 plus tool-specific adapter conformance | Includes web/search/fetches with a local model; permit only tested endpoints/effects and bounded costs. Local inference alone is not offline qualification. |

M21 is a development-only ephemeral demonstration, not the persistent personal
profile. B39 verifies an exact authorized fixed development artifact before parsing
and cannot dynamically download artifacts or execute model-supplied code. B70 later owns ordinary
verified package/trust/update delivery. M23 adds persistent personal controls.

B81 makes financial intent/reservations durable before dispatch, with restart,
period and old-snapshot reconciliation against authoritative issuer state. B82
qualifies endpoint/credential handling independently of account passwords; accepted
session-only credentials may require reauthentication after resume. Persistent
secrets require a separately tested key/storage path. No TPM, encrypted home or
irreversible offline counter is assumed.

## Decision coverage and acceptance trace

The numbered concerns correspond to the full source-backed
[AI architecture decisions](../02-architecture/ai-platform.md). Test procedures
are defined in [validation](validation.md#ai-profile-acceptance); draft protocol
semantics are in [AI tasks and policy](../06-contracts/ai-tasks-and-policy.md).

| Concern | Requirement | Work | Required check |
| --- | --- | --- | --- |
| 1 Task delegation and final external review | R21 | B67,B39,B71 | Changed payload/recipient/policy invalidates review; exact standing rule permits only its scope. |
| 2 Selected context and quiet presence | R20,R21 | B67,B39,B71 | Protected surfaces/other users excluded; bounded subscriptions sleep and revoke. |
| 3 Identities, credentials and grants | R21 | B67,B68,B72,B73,B82 | Forged principals, token audience, ambient admin and cross-owner attempts reject. |
| 4 Cancel, terminate, revoke, unknown effects | R23,R24 | B67,B69,B72,B81 | Crash before/after remote commit; retain uncertainty and outstanding holds. |
| 5 Lock, logout, suspend, background work | R21,R23 | B69,B71 | Personal grants end on logout; lock continuation needs bounded grant; no implicit keep-awake. |
| 6 Automatic editable preferences | R22 | B69,B71 | Poisoning, correction, exclusion and deletion; learning cannot grant authority. |
| 7 Separate retention and deletion | R22,R23 | B69,B79 | Source revocation, backup restore, tenant change and disclosed holds; derived indexes invalidated. |
| 8 Completion verified by owners | R23 | B67,B74 | Lying tool, missing artifact, stale receipt and unsent draft never count as completed send. |
| 9 Eligible model routing | R20,R25 | B39,B72,B73 | Outage/local absence never silently changes disclosure or payer; adapter semantics tested. |
| 10 Versioned model/policy updates | R23,R25 | B70,B74 | Signed incompatible update, stale trust and insufficient replacement headroom reject. |
| 11 Portable task continuity | R23 | B69,B74 | 100 compatible cycles, fresh grants, unchanged boot identity and accepted edits preserved; power-loss tested separately. |
| 12 Concurrent edits and retries | R21,R23 | B67,B69,B72 | Expected revisions and owner deduplication; uncertain non-idempotent effects reconciled before retry. |
| 13 Untrusted tools/content | R21 | B67,B39,B72 | Indirect injection, SSRF, credential forwarding and fabricated tool authority reject. |
| 14 Persistent off and conventional recovery | R20 | B39,B71,B74 | Stop admissions/observation/learning/warming; report remote limits; offline/off install/save/recovery works. |
| 15 Model/tool supply chain | R25 | B39,B70 | Malformed/oversize packages, untrusted code and unsigned imports reject through common verifier. |
| 16 Multiple budgets and sponsorship | R24 | B68,B72,B73,B79,B81 | Atomic reservations, concurrent children, all applicable caps, chosen payer, expiry/revoke/duplicate/unknown/overrun cases. |
| 17 Efficiency and local hardware paths | R25 | B39,B70,B74 | 256 MiB model-unloaded target and independent resident/active/default/peak profiles; desktop retains reserve. |
| 18 Native accessible controls | R20,R21 | B71,B74 | Keyboard/semantic do/guide/explain, review, memory, stop and denial remedies; human usability review. |
| 19 Federation and model cooperation | R26,R27 | B75,B76,B77,B78,B79,B80 | Separate trust/data/budgets; bounded subagents; no authority by model vote; server scope kept explicit. |
| 20 Source/provider terms and disclosure | R21,R22,R26 | B67,B72,B73,B79 | Reading/indexing/training/onward use separately checked, including queries, URLs, tools and logs. |
| 21 Whole-system value and qualification | R20–R27 | B74,B76,B79 | Repeated journeys, deterministic critical cases, human quality and cost/time/correction comparisons. |

## Product defaults and decisions still requiring evidence

Final send/publish review with scoped standing rules, automatic editable ordinary
preference learning, and a policy foundation for OS-sponsored allowance are owner
choices. No bundled dollar amount or automatic personal billing is authorized.
The 30-day completed personal task metadata/receipt retention and 95% ordinary
trial success threshold are proposed starting defaults, not measured outcomes or
relaxations of critical boundary checks.

Exact model size/quantization/context/runtime/CPU requirements, optional preloading,
hardware profile ceilings, provider agreements, residency/retention guarantees,
funding amounts, distributed ledger recovery and native server scope require
their named experiments and reviews. A capable machine may offer a verified local
download; capacity alone never silently authorizes downloads, tool disclosure or
cloud billing. Thin clients use only an eligible private/public route, or retain
conventional controls when none is configured. A capable machine also needs user
consent to storage/network use and a measured desktop reserve.

The retained [host reference](../../experiments/ai-policy/README.md) covers budget
semantics only. It is independently testable preparation for B68, not a production
policy engine, funded allowance, guest port or distributed accounting system.

## Audit corrections

The [2026-11-07 audit](ai-plan-audit.md) fixes route coupling, durable-spend and
credential prerequisites, M21 bootstrap/off semantics, derived-data restrictions,
receipt certainty, backup/period reconciliation and host reference admission bugs.
It adds B81/B82 without changing any original desktop release or baseline.
