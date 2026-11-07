# Native AI platform: architecture and decision review

Date: November 7, 2026, America/New_York. Owner-requested AI direction incorporated into requirements and roadmap; proposed technical architecture. Technical choices remain provisional until their acceptance experiments pass.

ContinuumOS should offer an immediately accessible assistant that works through native typed actions, preserves work across supported interruptions and updates, and accounts for its resource use. Gemma is the initial local qualification target. Private organizational or consortium infrastructure and approved GPT, Claude, Grok and Gemini APIs are separate destinations behind one native interface. Applications and durable task records remain independent of the selected model.

The owner selected final review for external sends/publication with scoped standing rules, automatic preference learning with an editable memory view, and a foundation for an OS-bundled allowance governed by declared budgets. No allowance amount, funded service, subscription, provider account, deployment or calendar commitment is established here. Existing M8/M14/M22/M15 release exits and accepted evidence remain unchanged. The [AI roadmap amendment](../05-delivery/ai-roadmap.md) and audit add conditional R20–R27, B67–B82 and M23–M26; M21/B39 qualifies the Gemma local path independently of remote routes.

The decisions below use primary specifications, provider documentation and original engineering reports. External facts are attributed locally; the proposed policies and acceptance gates are engineering judgments for this OS. A published standard or upstream example does not prove a ContinuumOS implementation.

## Components and trust boundaries

| Component | Owns and imports | Failure/replacement boundary |
| --- | --- | --- |
| Native assistant surface | Selected context, progress, review, memory and stop controls through SDK semantics | Sleeps when idle; conventional desktop remains usable after failure/off. |
| Task coordinator | Bounded logical task graph, proposals and task records | No ambient device/admin authority; checkpoint/rebind at supported boundaries. |
| Context/credential brokers | Source-scoped reads and separately brokered endpoint credentials | Reject stale sessions/epochs; secrets never enter model prompts/checkpoints. |
| Deterministic policy/accounting broker | Issuer-authenticated restrictions, effect approvals, resource admission, reservations and receipts | Narrow authority; fail closed on unavailable mandatory policy; durable ledger gates separate. |
| Isolated model workers / route adapters | Qualified local runtime or approved private/public inference transport | Bounded queues/resources; no direct effect authority; cancellation limits visible. |
| Typed executor and resource owners | Use-time validation, revisions, idempotency and authoritative effects | Existing owner single-writer rules; reconcile uncertain remote commit. |
| Outcome verifier | Owner artifacts/revisions/receipts and independent deterministic checks | Model opinions assist quality review; cannot certify permission or effects. |

This is a service graph above existing kernel/IPC, not a new privileged kernel
subsystem. Neither compositor/input nor boot/recovery depends on inference. Later
durable storage and credentials must pass their M6/M7/M11 contract gates; M21's earlier
subset is explicitly ephemeral. First-party guest integration remains Rust;
upstream runtime language/dependencies need inventory and exact-target probes.
CPU/backend qualification must match accepted FP/SIMD context support; current
x87/SSE evidence does not establish AVX/AVX-512/XSAVE ownership. Unsupported
features reject before dispatch or require their own kernel acceptance. A larger
local-profile machine capacity does not raise the model-unloaded desktop target.
The draft package/transfer limits are not already a large-model protocol: B67
must define bounded shards, streaming verification and any versioned resource/
mapping extension; B39 proves its minimal guest loader subset before use. Existing
64 MiB object and 2 GiB package-memory limits stay intact. B70 qualifies ordinary
delivery later; no large-file API or frozen-SDK extension is assumed.

Related: [model/runtime survey](../01-research/ai-model-and-runtime-survey.md),
[task/policy contract](../06-contracts/ai-tasks-and-policy.md),
[ADRs 0030–0033](../04-decisions/README.md), and
[21-concern delivery trace](../05-delivery/ai-roadmap.md#decision-coverage-and-acceptance-trace).

## Decisions and acceptance concerns

**1. Autonomy within an explicit task**

OWASP identifies excessive functionality, permission and autonomy as distinct causes of harmful agent actions. OpenAI's integration guidance recognizes scoped prior authorization, while its guardrail guidance puts checks at the action boundary. [OWASP excessive agency](https://genai.owasp.org/llmrisk/llm062025-excessive-agency/), [OpenAI integration recipes](https://developers.openai.com/api/docs/guides/tools-computer-use-integration).

Decision: a task receives a bounded delegation describing resources, allowed effects, destinations, budget and lifetime. Routine authorized reads, draft edits, previews and reversible settings proceed without repeated approval. Following the owner's choice, sending or publishing externally stops at a concrete final review unless a current scoped standing rule covers the exact action class, destination, data scope and limits. A general request to research or draft does not authorize transmission of its result. Purchases, permanent deletion, new access grants, privileged administration and running newly obtained code have explicit policy gates.

Gate: exercise accepted routine work, changed recipients, changed payload after final review, expired standing rules and attempts to turn a draft into a send. A one-shot review token binds task/effect/request identity, action, destination, payer, payload digest, resource revisions, policy epoch and expiry; consume it durably before dispatch and reject replay under a new request ID. Standing rules have explicit repetition/count limits. Modification invalidates review. Reconsider confirmation frequency using interruption and wrong-action measurements without relaxing authority boundaries.

**2. Context observation and quiet presence**

The existing action architecture defines provenance, audience, purpose, expiry and narrow resource references. Native semantics can supply useful context without treating the whole display as a document. [Existing action architecture](../02-architecture/actions-and-optional-ai.md).

Decision: the assistant affordance is available by default when the AI feature is installed. Activation starts with a selected item or explicitly scoped workspace. The OS context broker supplies semantic state and bounded excerpts. Background workspace subscriptions are separately visible, bounded and revocable. Continuous screenshots, global keystroke capture, microphone recording and whole-disk indexing are not default observation paths. Protected login/password surfaces stay outside ordinary AI context. Native action APIs take precedence; visual automation is an explicitly qualified fallback.

Gate: compare semantic context with the visible selected state; test private fields, other windows, second users, screen lock, source removal and revocation during streaming. Event subscriptions must sleep while idle. The model cannot enlarge its own observation scope.

**3. Identity, credentials and delegation**

OAuth security guidance restricts tokens by audience, resource and action. MCP identifies token passthrough and confused-deputy problems. These align with the OS's existing distinction between service identity and the user being served. [RFC 9700](https://www.rfc-editor.org/rfc/rfc9700.html), [MCP security practices](https://modelcontextprotocol.io/docs/2026-07-28/tutorials/security/security_best_practices).

Decision: run the coordinator, model worker, tool adapter and verifier as separately bounded services. The assistant acts for a user/task through narrowed grants. A user's administrator role does not automatically become assistant authority. Credentials remain in a broker; adapters obtain only their required credentials, and model context carries references rather than raw secrets. Authentication to a remote service does not imply authorization for every resource there.

Gate: forged identity, cross-user and same-user cross-app access, wrong token audience, forwarding beyond the caller's grant, broker restart, expiration and stale policy epochs must reject. Guest credential handling depends on the existing entropy/account/secret-storage gates; this review does not establish protected credential storage.

**4. Pause, cancellation, termination and uncertain outcomes**

A2A cancellation can fail. The MCP Tasks draft also makes cancellation cooperative. These are interoperability observations, not guarantees that a particular adapter honors cancellation. [A2A cancellation](https://a2a-protocol.org/latest/specification/#315-cancel-task), [MCP Tasks draft](https://tasks.extensions.modelcontextprotocol.io/specification/draft/tasks).

Decision: pause prevents new steps and checkpoints supported work; cancellation requests that in-flight work stop; termination stops an owned local worker; revocation prevents future mediated authority use. None silently undoes a committed external effect. The UI distinguishes stop requested, local worker stopped, remote status unknown, verified cancelled, partial completion and completed.

Gate: inject cancellation immediately before dispatch, during execution, after remote commit and before the reply. Preserve correlation IDs, effect receipts and budget reservations until the outcome is known. A timeout cannot be relabeled a confirmed cancellation.

**5. Lock, logout, suspend and unattended work**

Task lifecycle, user-session authority and device power state are different boundaries. The OS already requires fresh authorization after supported replacement and documents limits to revocation. [Identity and authorization](../02-architecture/identity-and-authorization.md).

Decision: ordinary interactive tasks pause on lock. A task may continue while locked only with an explicit bounded background grant and context available without reading protected surfaces. Personal interactive grants end on logout. Managed background work uses a separately authorized service identity, disclosed to its owner. Suspend does not automatically become permission to keep the machine awake. After reboot, supported tasks resume from logical checkpoints only after identity and authority are re-established.

Gate: lock/logout races, expired sessions, background grants, a stuck tool and suspended/resumed clocks. Lock-screen notifications reveal only permitted status. Persistent continuation waits for storage/session-recovery prerequisites; earlier demonstrations must say ephemeral.

**6. Automatic personal learning and memory ownership**

OWASP's memory-poisoning analysis documents how retained untrusted material can affect later sessions. Remembering information must therefore preserve its origin and scope. [OWASP memory analysis](https://genai.owasp.org/2026/05/13/memory-is-a-feature-it-is-also-an-attack-surface/).

Decision: enable automatic learning of ordinary preferences within authorized interactions, as requested by the owner. Use structured, editable memory; background fine-tuning of model weights is a separate operation requiring its own data, licensing and budget authorization. Each memory has a subject, scope, provenance, last-confirmed time, confidence and correction history. Users can inspect, edit, delete, disable learning or exclude a workspace. A page or tool result cannot create a trusted global instruction. Learned preferences cannot grant authority, choose a payer, alter policy or override an explicit current request. Sensitive personal attributes and secrets are excluded from automatic durable learning by default. Work and personal memories remain separate; cross-organization learning is opt-in under both policies.

Gate: poison a document, retrieve it repeatedly, then change projects and restart the assistant. Its instructions must not become trusted policy. Corrected/deleted memories must stop appearing in derived indexes and pending prompts. Confidence is a retrieval aid, not authority.

**7. Retention, deletion and durable work**

Decision: store active task progress separately from personal preferences, chat history and retrieval indexes. Prefer authoritative document revisions and resource references to extra copies. Preserve bounded unsaved content only where the supported task requires it and the workspace policy permits it. Personal task metadata/receipts have a proposed 30-day completed-task default; unresolved-effect receipts and holds remain until reconciliation or an explicitly documented resolution; saved user artifacts follow their owning application's retention. Full prompt/response capture is session-only by default unless history is enabled. Preferences persist until corrected/deleted or their source access expires. Enterprise records follow declared tenant retention and legal-hold rules, shown to the user.

The 30-day period is a product default for review, not a duration established by research. Persistent personal storage requires accepted account/storage protection; this plan does not assume encrypted homes already exist. Deletion invalidates derived indexes/caches and requests remote deletion where supported. Receipts distinguish local removal, remote request and confirmed remote removal. Turning AI off does not silently delete user documents or promise removal of data already disclosed.

Gate: delete source access, delete a memory, clear history, restore backup, change tenant and apply a retention hold. Reintroduced backups must not silently resurrect a revoked preference or authority. Private indexed content remains governed by source permissions.

**8. Proof of completion**

Agent evaluation guidance distinguishes the final environment outcome from what an agent says happened. [Anthropic evaluation guidance](https://www.anthropic.com/research/demystifying-evals-for-ai-agents).

Decision: application/resource owners issue authoritative results and revisions. The coordinator verifies the relevant saved file, service state, test result or external receipt. It reports completed, prepared for review, partially completed, blocked, or outcome unknown accurately. Verifiers are independent of the model's prose; an independent model can assess content quality but does not certify permission or a committed effect.

Gate: inject a lying tool response, a stale success record, a missing artifact and a successful draft with an unsent message. Outcomes must reflect actual state. Preserve useful artifacts when one step fails.

**9. Model routing and unavailable capability**

Gemma's memory requirements vary by configuration and context; the publisher's weight estimates omit additional runtime/context costs. Cloud data handling also depends on features and endpoints, including third parties. [Google memory considerations](https://ai.google.dev/gemma/docs/core), [OpenAI data controls](https://developers.openai.com/api/docs/guides/your-data).

Decision: evaluate permission first, capability second, resources/cost third. Gemma is the initial supported local target; enterprise infrastructure may offer other qualified families. Public GPT/Claude/Grok/Gemini routes are separately qualified. An unavailable local model cannot trigger unapproved disclosure. A task may wait, use an already eligible alternative, run a reduced supported operation with a clear capability label, or preserve progress and explain its limit. Route changes cannot silently change contractual handling or the payer.

Gate: provider outage, rate limit, expired entitlement, disconnected private server, missing local model and cloud-prohibited data. Test API-semantic differences rather than accepting an 'OpenAI-compatible' label as complete conformance.

**10. Model and policy updates**

TUF provides signed metadata, role separation, expiry and update-integrity patterns. The OS already distinguishes verified update content from activation and state migration. [TUF specification 1.0.36](https://theupdateframework.github.io/specification/v1.0.36/), [OS live-update model](../03-live-update/model-and-guarantees.md).

Decision: a release configuration identifies the exact model/quantization, tokenizer, prompt template, runtime/backend, tool schema, policy profile and evaluation suite. Stage and verify it, run regressions, then switch at supported task boundaries. An in-flight final approval remains bound to its reviewed semantics; incompatible changes invalidate it. Keep a qualified previous configuration where state compatibility permits. Model bytes can be downgraded only through an authorized recovery procedure; code rollback cannot undo external effects or accepted user data.

Gate: correct signatures on an incompatible package, revoked signer, stale/frozen metadata, changed tool semantics, insufficient replacement headroom and injected migration failures. Fleet canaries and rollback require their own later operational qualification.

**11. Durable tasks and supported live replacement**

Decision: the stable record contains the goal, permitted resources, task/schema version, current step, artifacts, revisions, request IDs, receipts, unresolved effects, remaining budget reservations and logical continuation. Keep stacks, native closures, raw handles, credentials and model KV caches out of the portable authority record. Rebind resources through the current broker. Compatible model/service replacement reuses the record; incompatible changes stop with an explicit reason.

This extends the OS's existing state rules. Checkpoint compatibility is explicit; it is not automatic transfer of model internals. [State and compatibility rules](../03-live-update/state-and-compatibility.md).

Gate: worker death, route replacement, schema change and policy revision across at least 100 compatible cycles, with unchanged boot identity for supported live changes and no lost accepted edits. Durability across power loss remains a separate M6-class property.

**12. Concurrent edits, retries and external effects**

HTTP specifies conditional operations and restricts automatic retries of non-idempotent requests. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html).

Decision: use typed action IDs, expected revisions and owner-scoped idempotency keys. Recheck source and destination at commit. Parallel analysis is allowed; authoritative mutation follows each resource owner's writer/revision rules. If a reply is lost after a possible external commit, query by request ID or reconcile authoritative state before retrying. When the service cannot resolve uncertainty, preserve 'unknown' and request a decision instead of resending blindly. Undo is an explicit compensating operation where supported.

Gate: delayed replies, duplicates, crash after send, another app's edit, out-of-order results and deduplication expiry. No generic distributed exactly-once or universal undo promise is made.

**13. Untrusted content, tools and native execution**

Decision: documents, websites, retrieved memories, agent messages and tool outputs remain untrusted data. The planner emits bounded typed proposals. A separate executor validates target, schema, identity, current scope, revision, route, effect and budget. Tool descriptions aid discovery but confer no authority. Restrict outbound endpoints; a page cannot invent a model URL or redirect credentials. General shell execution is a developer/workspace capability with explicit bounds; native typed actions are the default desktop interface.

Checks belong at effect boundaries, because entry/output filters do not cover every delegated tool invocation. [OpenAI guardrail boundaries](https://developers.openai.com/api/docs/guides/agents/guardrails-approvals), [MCP security practices](https://modelcontextprotocol.io/docs/2026-07-28/tutorials/security/security_best_practices).

Gate: indirect injection, malicious tool descriptions, fabricated handles, shell expansion, changed targets and credential forwarding. Structured output limits shape; it is not a guarantee of correct intent.

**14. The off control and recovery**

Decision: global off immediately closes admission for new user AI work, stops observation and learning, disables automatic model warming, requests remote cancellation and terminates owned local workers through the supported lifecycle. Retain protected receipts needed to reconcile already dispatched work, with status visible in ordinary settings. Strict offline forbids outbound reconciliation too; unresolved status and holds remain until explicitly permitted. Unload/scrub supported local model/context resources and report failures honestly. In-flight remote actions may still finish. Task-specific stop, workspace off, local-only and cloud-off remain separate controls. Changes persist across supported reboot/session restoration.

The personal user can turn their assistance off. Managed service requirements must be separately named and disclosed; an organization cannot call continued personal observation 'off'. Recovery and ordinary manual controls remain available without inference.

Gate: exercise off with queued, streaming, waiting-for-review, dispatched and uncertain tasks, including restart. Queued work must not resume merely because the assistant UI restarts. Verify actual resource release and suppressed observation, not just a toggle's appearance.

**15. Model and extension supply chain**

Safetensors avoids pickle's executable object-loading mechanism; signing and transparency identify artifacts and publishers. These mechanisms do not establish that trusted content is harmless or that every parser is correct. [Safetensors](https://huggingface.co/docs/safetensors/index), [Sigstore security model](https://docs.sigstore.dev/about/security/).

Decision: distribute immutable manifests with content digests, publisher identity, dependency/license notices, resource/CPU requirements and declared data/tool access. Model loading uses bounded data parsing, with arbitrary remote-code execution disabled in ordinary packages. Treat tokenizer code, converters, adapters, prompt templates and tools as independently reviewed dependencies. Use the existing package/update trust architecture; do not introduce a model-specific unsigned update bypass. Air-gapped imports use the same verification policy with an explicitly managed trust root and expiry process.

Gate: malformed dimensions, oversize allocation, decompression bombs, unexpected code, mismatched files, stale metadata, revoked publishers and interrupted loading. Initial runtime/backend choice remains an exact-target experiment; neither Rust bindings nor upstream CPU support prove a guest port.

**16. Budgets, funding and the policy engine**

Existing gateways demonstrate key/user/team/server limits and provider translation. Their presence does not establish our required reservation, privacy or delegation semantics. [LiteLLM request architecture](https://docs.litellm.ai/docs/proxy/architecture), [virtual keys](https://docs.litellm.ai/docs/proxy/virtual_keys).

Decision: create a deterministic OS policy broker with versioned typed inputs and reasoned decisions. Keep natural-language policy interpretation advisory. The broker evaluates current identity and task authority, data/destination rules, effect-review rules, model capability, resource admission, budget availability and funding entitlement. A permit must satisfy every applicable mandatory restriction; a funder supplies capacity, not authority. User preferences cannot override an organization or resource owner's applicable restriction, and a model cannot edit the effective policy.

Budget sources include OS-sponsored allowances; enterprise/department/team pools; installed-device/fleet policies; project/workspace/app quotas; user total-use limits; separate personal-charge limits; task/session limits; parent/education/library/shared-device controls; research grants and consortium sponsors; provider credits or promotions; and metered-network/energy/resource constraints. Each carries issuer, scope, payer, meter, period, expiry and authorized revision. Company-wide sponsorship requires a server-side authoritative ledger, not a replenishable counter trusted to the desktop.

A personal-charge ceiling of zero can coexist with OS sponsorship. A total-use ceiling applies across all eligible payers. A sponsored entitlement must not silently become a personal charge when exhausted. Alternatives are pause, an eligible lower-cost route or a separately authorized funding change. Neither a dollar amount nor an actual funded allowance is invented in this review.

Admission reserves worst-case declared usage against all applicable limits atomically. Concurrent child agents share ancestor ceilings. Dollars, provider credits, token counts, time and local memory are distinct dimensions; values are compared only within a defined unit and scope. The dispatcher rechecks policy/entitlement before use. Settlements reconcile actual usage and release unused reservations. Uncertain remote work keeps reservations. An overrun is recorded and stops further admissions; it is not hidden by capping recorded cost. Hosted tools/search charges and retries need their own bounded estimates. A provider without a trustworthy cost bound cannot promise an absolute financial cap; require provider-side enforcement or an explicitly accepted exposure limit.

Gate: concurrent reservations; OS, enterprise and user caps; separate personal charges; exhausted promotions; source changes; credit/currency mismatch; expiration; revocation between admission and dispatch; lost receipts; duplicate settlements; delayed provider charges; clock changes and process/leader failure. No policy engine can reclaim money already committed externally by toggling a local flag.

The companion [budget reference model](../../experiments/ai-policy/budget-policy-reference.cjs) implements only the bounded in-memory budget-admission/reservation portion. Its [tests](../../experiments/ai-policy/budget-policy-reference.test.cjs) cover 28 cases. Inputs must already come from a trusted authenticated broker and price-bound adapter. It performs no network requests or actual payments. It is a single-process research oracle, not a guest service, authorization verifier, distributed ledger, persistence implementation, periodic allowance reset or production financial guarantee. Terminal records are bounded and never silently evicted. A Rust/guest implementation should be independently checked against its behavioral cases after contract review.

**17. Efficiency, local inference and user-facing responsiveness**

Decision: retain 256 MiB as the model-unloaded baseline target, including dormant AI controls/broker/policy state. Qualify local-resident, active-inference and actual-default-boot profiles separately. Count weights, scratch/context buffers, index caches, model-loading/replacement peaks, queues and device/shared memory. Cloud/private-server clients still account for local TLS, context and tool costs. Model execution must not sit on the compositor/input path. Bounded queues, backpressure and admission priority protect interaction and saving work. Unload before exhausting the desktop reserve; do not silently substitute cloud disclosure.

Gate: ordinary desktop, cold/warm model startup, long prompts, concurrent tasks, update peaks and repeated unload/reload under the existing efficiency targets. Choose the smallest *qualified* configuration for the supported workload; a small binary or published phone result is insufficient. Hardware-specific resident ceilings, context limits and time-to-first-result remain measured qualification outputs.

**18. Accessibility, understandable control and conventional operation**

WCAG supplies relevant keyboard/focus principles for web content, while this native OS has its own accessibility contracts. It does not become WCAG-certified merely by borrowing those principles. [WCAG 2.2](https://www.w3.org/TR/WCAG22/).

Decision: provide keyboard-complete task progress, review, memory, policy explanations and stop controls with native semantics. Users may choose 'do it', 'guide me' or 'explain it' within the same action boundary. Keep conventional installation, saving, permissions and recovery usable when AI is off, offline, unconfigured or failed. Progress should distinguish waiting, executing, reviewing and verifying. Show destinations and funding when relevant, and explain a denial with a concrete remedy that respects policy. Background suggestions are quiet, rate-limited and dismissible.

Gate: pointer-free and semantic-client workflows, focus on approval changes, reduced motion, non-color status, complex text, lost connectivity and inference failure. Validate usability with people; model-generated explanations alone do not establish accessibility.

**19. Enterprise federation and cooperating models**

SPIFFE describes authentication across independently administered trust domains. A2A supports agent/task exchange; authorization and resource policy remain implementation responsibilities. [SPIFFE Federation](https://spiffe.io/docs/latest/spiffe-specs/spiffe_federation/), [A2A specification](https://a2a-protocol.org/latest/specification/).

Decision: each member retains its own trust root, tenants, records, context policy, keys and budget issuer. Federation exposes approved services and artifacts; it does not pool all member data or automatically share caches and memory. Apply disclosure policy to derived summaries and embeddings as well as original documents. Offboarding stops new delegation, revokes mediated access, settles outstanding spending and records remaining remote effects/deletion obligations. Remote parties cannot be forced to forget already received information by revoking a local grant.

Cooperating models receive bounded subtasks and scoped context. A coordinator maintains the task record; resource owners authorize mutations. Use specialized models or reviewers when workload evidence justifies their extra cost. Model voting never grants permissions or proves correctness. Federation does not imply federated training or permission to contribute data/weights to a shared training run. Native data-center execution remains a separately scoped OS program; connecting to an existing private inference service does not establish a GPU/runtime/container port.

Gate: two independently administered members, denied cross-tenant search, revoked trust, compromised peer/tool output, budget exhaustion in a child, network partition, delayed results, duplicate effects and member departure. Keep tightly coupled GPU collectives inside an appropriately managed cluster; federation is a service boundary.

**20. Websites, provider terms and downstream disclosure**

The IAB AI-CONTROL workshop report describes fragmented mechanisms for publisher preferences across crawling and inference. It is an informational report, not a universal consent protocol. Provider residency/retention settings also have feature and third-party boundaries. [IAB report](https://www.rfc-editor.org/rfc/rfc9969.html), [OpenAI data controls](https://developers.openai.com/api/docs/guides/your-data).

Decision: retain origin, access grant, source revision and applicable use/retention rules with context and research artifacts. Treat reading, summarizing, indexing, training, storing and onward disclosure as separate operations. Enforce reviewed rules and supported publisher signals; do not promise automatic legal interpretation of every website's prose. Use supported APIs and authorized sessions. Apply checks to search queries, tool arguments, URL/image fetching, tracing and diagnostics, not only the main model request. Model-local does not imply offline when tools call external services.

Gate: restricted source in a public-search query, credentials embedded in a URL, a redirected fetch, tool-induced uploads, prohibited retention, missing attribution and a provider feature whose policy differs from ordinary inference. Exact model/package licenses are pinned with artifacts. The repository's existing U.S.-only handling hold is unchanged; no remote service is certified by this research.

**21. Acceptance, evidence and delivery order**

Decision: qualify the complete model/coordinator/tool system on three initial journeys: complete a bounded cross-app task; diagnose and perform a supported recovery; resume interrupted work through compatible replacement. Compare with the same conventional workflow. Use deterministic state/effect checks, repeated model trials and human review for content quality. Track success, correction burden, unnecessary approvals, wrong-target attempts, unauthorized effects, resource/cost per completed task, latency and recovery continuity. Model self-reported confidence is not an acceptance oracle. [Anthropic evaluation guidance](https://www.anthropic.com/research/demystifying-evals-for-ai-agents).

Proposed initial qualification gate: a preregistered representative ordinary-task corpus, at least three trials per task/configuration, at least 95% successful ordinary trials, no lost accepted user state, and zero observed unauthorized committed effects or unexplained disclosures in the required positive/adversarial matrix. This 95% is an engineering starting target, not a published result or a guarantee for high-consequence tasks. Supported-task declarations narrow when a configuration fails; high-consequence actions retain review. Preserve capability failures and regressions separately. Each critical deterministic boundary/failure case must pass rather than being averaged into the success percentage.

First deliver the context/authority/task/effect/budget contract and independent host fixtures. After M5, native Gemma qualification and route-independent personal foundations have separate paths. Private/public guest adapters additionally require network/time/trust, durable dispatch B81 and endpoint/credential B82 gates, without waiting for local Gemma or each other; host fixtures can precede guest integration. Every enabled route/tool must qualify before its named profile passes. Add model cooperation and federation through separate acceptance. Durable tasks, real accounts, fleet operations and server execution retain their own dependencies. M3 foundations, M4 semantic actions, M5 replacement, M6 persistence, M7 accounts and M12 networking contribute interfaces; they do not already deliver this AI feature. B39/E10/T15 are expanded by the [owner-requested AI amendment](../05-delivery/ai-roadmap.md); its broad work items are Not started and calendar dates remain unestimated.

The current P4-02 next step and M8/M14 exits are preserved. No production native budget engine or AI service is accepted by this local reference model.

**Review outcome and remaining experiments**

The owner's three product choices are incorporated. No further product answer is required to complete this research review. Actual bundled-allowance amounts, commercial terms and model/hardware resource ceilings need pricing and exact-target measurements before rollout. Retention duration and ordinary-task success thresholds are explicit proposed defaults for later product review. The local budget tests establish only the reference semantics exercised; guest authority, durable accounting, multi-process consensus, hosted billing and whole-system efficiency remain unverified.

The host reference sources are retained in `experiments/ai-policy/`. Generated results stay local/ignored. These sources do not establish a guest service or change accepted evidence, frozen SDK IDs, signing policy or remote handling authorization.

Run the reference checks from the repository root with `node --test experiments/ai-policy/budget-policy-reference.test.cjs`.

## Audit refinements — 2026-11-07

[The audit](../05-delivery/ai-plan-audit.md) corrects dependencies and edge cases.
The [conditional route matrix](../05-delivery/ai-roadmap.md#conditional-route-qualification)
governs qualification: Gemma is the local default, but remote/thin-client profiles
do not require a native Gemma port, and public routes do not require private-model
implementation. Every enabled route/tool must still qualify. M21's fixed-fixture
development subset is ephemeral, including session off; no automatic reactivation
or warming. Production persistent controls and package lifecycle wait for M23.

B81 adds crash-safe intent/reservations, actuals, period and old-snapshot/issuer
reconciliation before remote dispatch. Unresolved liabilities never become new
allowance after restore/rollover. No offline anti-rollback guarantee is claimed
without a trusted anchor. B82 adds exact-target credential/endpoint brokerage;
account passwords and login are insufficient. Session-only secrets need fresh
authentication after resume; protected persistent storage has a separate key gate.

Derived data inherits every contributing source's restrictions; models cannot
declassify by paraphrasing. Owner/provider receipts state exactly what was observed:
accepted, confirmed delivered or unknown, with authenticated provenance. Strict
offline forbids outbound reconciliation too; keep unresolved state/holds. A verified
quote-bound breach blocks new/queued work even below aggregate spending caps.
These clarify the 21 decisions above and supersede any broader earlier reading.
