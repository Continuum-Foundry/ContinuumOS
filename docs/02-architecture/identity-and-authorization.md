# Identity, ownership and authorization

Status: proposed engineering plan, updated at the owner's request on 2026-10-21.
No user isolation, account authentication or RBAC implementation is claimed.
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
All are Not started; no account UI or credential handling is needed to test them.

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
selects no credential protocol, crypto library, administrator role hierarchy or
permanent syscall/wire layout.
