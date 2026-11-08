# Identity, ownership and authorization

Status: proposed engineering plan, updated at the owner's request on 2026-11-08.
P3-10 accepts the bounded synthetic enforcement subset below; account authentication and durable RBAC remain unimplemented.
Supports R10/R14 and R19's authorization boundary. See [M3 and later gates](../05-delivery/milestones.md),
[capability design](capabilities-and-ipc.md) and [ADR-0010](../04-decisions/0010-capability-security.md).

## M3 scope and identity model

M3 must establish authority provenance before apps/services depend on an implicit
all-powerful development user. Through M6, scope remains one local development user.
Two synthetic users, a service principal and ephemeral role assignments exercise
the boundary; they are not a login system or persistent account database.

| Concept | Required distinction |
| --- | --- |
| User principal | Logical owner identity; not a display name, PID or secret |
| Service principal | Service's own identity and grants; separate from a user it serves |
| Session | Trusted binding to a principal with a bounded lifetime/epoch; not the kernel boot ID or a supplied JSON identifier |
| Process instance | Specific execution instance bound by the trusted launcher to a principal/session and granted handles |
| Resource owner | Broker-controlled ownership metadata; knowledge of an owner/object ID does not grant access |
| Role assignment | Scoped policy input assigned by an authorized broker; not a privileged string supplied by an app |

Use versioned encodings and explicit identifier lifetimes; do not persist PIDs or
native Rust layouts as identities. Specify reuse, stale references and restart
behavior before freezing M3's independently built client/service ABI. Initial
in-memory IDs need no claim of cryptographic unpredictability: access requires
validated handles and broker checks, not guessed-ID secrecy.

## Policy and enforcement

Default to denial without an explicit grant. A policy broker may translate a
principal's scoped role assignment into capability grants within the broker's
delegation ceiling. Kernel handles enforce object rights; services enforce their
own ownership, scope and current lease/policy requirements. Ownership can inform
policy but never bypass these checks. A user's apps receive only their explicit
grants, even when that user has an administrative role.
Reserve installer disk-write and account-administration scopes separately from
ordinary app and development-session authority. M3 tests denial without these
grants using synthetic operations; the actual installer/account services arrive at M7.

Derive caller context from trusted launch/IPC provenance. Payload fields such as
owner, principal, session or role are untrusted assertions or requested targets.
They cannot replace the trusted sender context. Specify how that context follows
channel delegation/transfer; the sender need not be the channel's creator. A service acting for a
user checks both the user's request authority and its own delegation; possessing
a powerful service handle must not let a caller borrow that power implicitly.
Child launch, handle duplication and transfer cannot amplify authority. Cross-user
sharing requires an explicit authorized grant with documented scope and lifetime.

Role removal or session expiry stops new grants and invalidates brokered operations
at their next mandatory check, using policy/session epochs and leases. Define
in-flight operation behavior and bound any cache/lease delay. Closing a route or
logging out does not automatically revoke copied direct handles or shared mappings.
Resources requiring revocation must use mediated access or another tested mechanism;
record the remaining authority and its teardown behavior explicitly.

On broker restart fail closed until trusted policy/context is restored. A replacement
service obtains a fresh process binding and reauthorized grants; an app checkpoint
cannot restore credentials, role assignments or old privileged handles. M3 defines
this contract; M5/M6 must test live replacement and durable recovery respectively.
Record bounded grant/deny/revocation diagnostics without credentials or document data.

## M3 work items and acceptance

These expand the existing [local backlog](../05-delivery/backlog.md), not new Jira
issues or completed tests. M3 acceptance requires guest enforcement with independently
built clients/services; hosted policy-model fixtures alone are insufficient.

| Work item | Required cases |
| --- | --- |
| B11: launch/context | Bind user A, user B and a service to distinct process contexts; reject forged principal/session selection; ensure child authority stays within explicit delegated grants; stale/reused process IDs cannot select another caller |
| B12: handles/IPC | Reject payload owner/role spoofing and cross-owner access without a grant; allow explicitly shared read access while denying write; deny a second app of the same user without its own grant; verify sender attribution after channel transfer; preserve attenuation and atomic transfer failure semantics |
| B13: broker/policy | Authorize role assignment changes only within delegated scope; reject a role outside its resource/session scope; test service acting for a less-privileged caller; reject expired/revoked brokered operations and stale policy epochs; fail closed on broker restart |
| B13: teardown | Test session/process death, queued and in-flight requests, transfer races and stale leases; document direct-handle/shared-mapping authority that remains; do not report universal revocation |
| B15 handoff at M4 | Bind dev-session credentials to the trusted caller context; reject replay and claims to another desktop user/role; development update authority does not imply account administration |
| B37 handoff at M3/M5 | Preserve provider-side principal/session/scope checks across UI and automation; role removal between preview and execution denies the operation; replacement requires fresh grants and rejects checkpoint-supplied authority |

Keep positive and negative outcomes, identity lifetimes, scope, policy epoch and
the actual exercised boundary in acceptance evidence. The
[fourteen planned Phase 3 stories](../05-delivery/phase-3/README.md) carry these
rows into implementation gates and the [final acceptance matrix](../05-delivery/phase-3/14-phase-3-acceptance.md#required-acceptance-matrix).
P3-01 through P3-14 now accept their declared local experimental subsets; see the
[strict M3 review](../../evidence/phase-3/P3-14/2026-11-06/README.md).
The synthetic policy gate does not implement account UI or credential handling.

## Later accounts gate

The mandatory M7 accounts gate (B41) follows M6 storage/recovery and M4 trusted input
foundations. It is independent of optional inference and is required for B43 setup
and M8. The [approved amendment](../00-overview/reconciled-plan.md#installer-and-accounts-amendment--2026-10-21)
supersedes the earlier optional-account/single-development-user M8 scope. Require:

- Implement durable account create/disable/remove, persistent ownership and explicit
  data disposition; deleting/recreating an account must not inherit stale grants.
- Add login, logout, lock/unlock, credential verification/change and recovery with
  a reviewed entropy/credential design and trusted secure-input path.
- Select and validate guest entropy, credential storage/verification, secure input
  and recovery dependencies using M6 feasibility evidence before accepting real
  credentials; freeze no crypto algorithm or package through this plan alone.
- Create the first owner account through restricted graphical first-installed-boot
  setup before ordinary desktop access. Durable account/ownership/role/setup state
  must be coherent and idempotent across interruption; no default password,
  duplicate privileged account or incomplete-setup bypass is allowed.
- Implement scoped role administration and durable policy changes; verify that
  users, apps and services cannot assign themselves additional authority.
- Test simultaneous user sessions, explicit sharing, cross-user access denial,
  session expiry/logout, declared revocation limits and isolated clipboard/input/
  capture policy. Account disable must have an explicit existing-session outcome.
- Test interrupted account/policy writes and recovery without restoring revoked
  grants or leaving an unintended privileged account. Provide bounded audit events.
- Preserve account records, credential records and ownership through supported
  repair/reinstall. Require authorized recovery rather than an undocumented rescue
  login bypass. Keep credentials out of ordinary diagnostics and app checkpoints.

Encrypted homes require a separate key provisioning, storage and recovery design.
Neither login nor role policy alone establishes at-rest encryption. This document
selects no credential protocol, crypto library, engine dependency or permanent
syscall/wire layout. The following owner-requested product direction defines
account presets and scoped roles; exact implementations remain gated.

## Account presets and scoped roles

Date: 2026-11-08. See [ADR-0034](../04-decisions/0034-native-authorization-and-scoped-roles.md)
and the [authorization roadmap](../05-delivery/authorization-roadmap.md).
Preserve B41/M7's existing mandatory local-account exit. Cedar selection, expanded
presets and richer administration are separately qualified through B83/B84;
they add no dependency to M8/M14/M22/M15 or the ordered Phase 4 stories.

| Preset / identity | Proposed behavior |
| --- | --- |
| Administrator | Standard desktop authority until explicit, authenticated elevation for a scoped action; first owner receives an explicit administrative assignment during restricted setup |
| Standard user | Personal resources, permitted apps and explicit sharing; may hold delegated roles over selected resources |
| Guest | Opt-in isolated temporary session; no access to other homes, saved credentials or administrative operations; persistence/export/device access explicitly controlled |
| Service/workload | Noninteractive principal independent of a human login, with explicit launch identity and narrow grants; not a selectable human account preset |

Account identity is stable across renaming; deletion/recreation never recycles
authority. Presets are editable policy templates within administrative ceilings,
not numeric privilege levels. A user may hold several scoped roles. An assignment
records subject/group, role definition/version, resource scope, authorized issuer,
validity and conditions such as elevated authentication. Resolve groups and roles
from trusted state, not app-supplied strings or unverified identity claims.

Bound group nesting, role inheritance, scope depth and evaluation work; reject
cycles and unauthorized cross-tenant inheritance. Inheritance never widens a
resource scope or delegation ceiling. Separate permission to use a role from
permission to assign it; a role administrator cannot delegate beyond its explicit
assignment ceiling. Keep account, network, package/deployment, backup/restore,
security-policy, audit and recovery responsibilities separable. Administrative
status gives apps no ambient authority and implies no right to decrypt private data.

Elevation binds trusted user presence/authentication, exact action/target, session,
policy revision and expiry. Changed inputs require a new authorization check.
Define nested elevation, cancellation and lock/logout outcomes; no reusable
unbounded privileged shell token. Emergency recovery uses separately provisioned,
audited authority with explicit limits and post-use review, never a rescue bypass.
Guest logout/crash must revoke mediated grants and settle resources; publish
durability/export and cleanup limits without promising physical secure erasure.

## Native authorization broker and Cedar evaluation

Use one logical authorization contract across desktop UI, CLI, automation, remote
management and AI typed actions. Authentication establishes a principal/session;
authorization evaluates principal, action, resource and trusted context; the broker
issues only grants within its own delegation ceiling. Kernel handles enforce
object rights and resource services enforce current operation policy. A visible
disabled control is explanatory UI, not an enforcement boundary.

Prefer a user-mode local broker. Keep general policy parsing/evaluation out of
the kernel and compositor/input hot paths. Requests and decisions use bounded,
versioned encodings, trusted caller/app/workload provenance, request/resource
revisions and policy/session epochs. Services recheck at effect/commit where
required, including changes after preview. An allow decision is not itself a
forgeable capability. Broker crash/replacement requires trusted reconstruction
and fresh grants; app checkpoints cannot restore authority. Cache keys include
scope/tenant and epochs; bound stale authorization and document in-flight outcomes.
Direct copied handles/mappings retain their documented lifetime: no universal
revocation claim follows from logout or a policy change.

Cedar is the preferred **evaluation candidate**, not a selected guest dependency.
Its upstream Rust implementation and role/attribute patterns support this design;
its default-deny and forbid-overrides-permit semantics are useful inputs.
Cedar skips policies that produce evaluation errors. The proposed OS wrapper
denies affected requests on evaluation errors reported in diagnostics or incomplete trusted
context, rejects invalid policy activation and bounds resource use. This stricter
wrapper behavior must be independently tested. [S115](../01-research/sources.md#s115)

Compare Cedar with Casbin's configurable access-control models and OpenFGA's
relationship-based role/group modeling. Evaluate OpenFGA primarily for later
shared-resource graphs; do not require a remote server for local login or ordinary
desktop authorization. These engines do not supply account credentials, durable
ownership, kernel enforcement or recovery. [S116](../01-research/sources.md#s116),
[S117](../01-research/sources.md#s117)

B83 must review an exact version/commit, transitive licenses, features, allocator,
standard-library/OS dependencies, panic behavior and maintenance/update obligations.
Run shared allow/deny/error fixtures, then original-guest linking/execution and
broker integration probes before selection. Measure bounded policy/entity/cache
sizes, cold/warm decision latency, peak memory, idle work and restart/replacement
cost. Set numeric profile limits before acceptance, within the unchanged 256 MiB
cold-idle target including dormant policy state. If feasibility fails, record a
bounded alternative or stop/replan; do not silently port Linux or freeze an
untested Cedar-compatible interpreter as the security contract.

## Consumer controls and enterprise boundaries

B84 adds accessible preset/elevation/sharing flows and an effective-permissions
inspector: who can perform an action, on which resource, through which assignment,
under which conditions, and until when. Include policy preview/diff, bounded
delegation, safe activation/recovery and privacy-conscious grant/deny/change audit
records. Explanations must reflect authoritative decisions without exposing other
tenants' policy, credentials or protected data. Store no policy authority in AI
preferences; AI follows the same grants and effect checks.

M27/B85–B87 qualify enterprise identity and fleet policy independently of AI:
explicit enrollment/trust roots, authenticated issuer/audience/tenant mapping,
directory/group synchronization, role mapping, scoped remote administration,
signed/versioned policy distribution, rotation, revocation and offboarding.
Evaluate identity protocols/adapters at their gate; no AD/LDAP/OIDC/SAML/SCIM
integration is selected by this plan. External authentication/group claims are
policy inputs and cannot grant local administrator authority by themselves.

Define bounded offline leases with trusted-time/rollback behavior, freshness
requirements for privileged operations, partitions and reconnect reconciliation.
Keep conventional authorized local recovery available under declared policy;
network loss must not create an administrative bypass. Offboarding defines
existing-session/workload outcomes and cannot recall already disclosed data.
M25 remains AI federation; neither it nor M7 proves enterprise OS identity.

M28/B88/B89 separately qualify workload identity and data-center management
authorization: bootstrap/rotation, short-lived scoped credentials, tenant/fleet/
cluster/project/resource assignments, deployment/operator/auditor separation and
bounded delegation. Headless administration uses the same checked actions with
noninteractive authentication and explicit approval rules. Test compromised
controllers/workloads, replay, partition, stale leaders and policy rollback.
This accepts only the named authorization profile. M26 remains AI server
feasibility; a native server runtime, HA controller/storage, hardware trust or a
production data-center OS requires its own approved program and evidence.

## P3-10 ephemeral scoped policy subset

P3-10 passes local self-review for its ephemeral scoped policy subset: 120 real IPC decisions, 67 expected denials, two broker faults/restarts, identical 160-byte replay across fresh process/session bindings and two exact resource settlements. All 82 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel/wait/supervisor regressions pass. The CPL3 broker/provider enforces exact scoped role assignments, child-grant attenuation, caller/service forwarding ceilings, finite sessions, use-time expiry/epoch/revision/context checks and synthetic privileged-scope separation. Queued grant and session expiry cross their actual elapsed fence; 136 focused evidence mutations reject. Trusted restoration is explicit and ordinary direct mappings survive broker death until owner teardown. No real accounts, credentials, durable RBAC, universal direct revocation, independently frozen SDK, guest UI/reload or desktop efficiency target is accepted; P3-11 follows in the current-state review below. See the [acceptance evidence](../../evidence/phase-3/P3-10/2026-10-30/README.md).

## P3-11 ephemeral service routes and writer fencing

P3-11 passes local self-review for bounded ephemeral service routes and writer fencing: 131 real decisions, 25 authoritative mutation attempts, 19 commits, 54 process runs across nine incarnations, eight injected faults and nine exact settlements. The guest separates fence, writer grant and publication while both services are alive; delayed old-epoch and retained old-endpoint writes deny. Bounded queue/result retention, duplicate/reorder/cancellation/reply outcomes, current policy/session checks and trusted fresh bootstrap pass. All 93 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and nine separate prior runtime regressions pass. The focused reviewer rejects 117 evidence mutations. Independent private RW mappings retain their ordinary lifetime until owner teardown. This accepts no durable activation, state migration, safe old-code retirement, independent SDK freeze, guest UI/reload or desktop efficiency target; P3-12 is next. See the [acceptance evidence](../../evidence/phase-3/P3-11/2026-10-31/README.md).
