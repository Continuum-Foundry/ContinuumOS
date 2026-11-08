# Accounts, scoped authorization and enterprise delivery

Date: 2026-11-08. Status: owner-requested planning amendment; technical choices
remain Proposed. B83–B89 and M27/M28 are Not started. No P-level stories are added.

## Preserved gates and scope

Keep P4-03 next after accepted P4-02 and preserve P4-03–P4-19 order, existing
counts/deadlines, evidence and validation policy. M7/B41 still supplies mandatory
local accounts, sessions and scoped RBAC for M8. Existing B01–B82 dependencies and
M8/M14/M22/M15 release exits remain unchanged. Cedar qualification and expanded
presets/administration are separately gated additions, not new release blockers.
R01–R27 retain their existing scope; generic enterprise authorization does not
become conditional on AI R20–R27. Preserve the original kernel, independent ISO,
native desktop, live-development requirements and 256 MiB cold-idle target.

The backlog now contains 89 broad tasks. M27/M28 are independent profile gates;
milestone numbering is not a universal serial order. No delivery dates or native
server support are implied. Decompose later milestone stories only at the owner's
request. Architecture: [identity and authorization](../02-architecture/identity-and-authorization.md).
Decision: [ADR-0034](../04-decisions/0034-native-authorization-and-scoped-roles.md).

## Ordered work and prerequisites

| Gate / work | Prerequisites | Required outcome |
| --- | --- | --- |
| B83 engine evaluation | B13,B37; host research may precede guest integration | Exact Cedar/Casbin/OpenFGA comparison; bounded independent semantic fixtures; license/runtime review; original-guest probe for candidate selection; predeclared numeric latency/memory/complexity budgets and measured outcome or stop/replan |
| Existing M7/B41 | Existing M6/storage and trusted-input prerequisites | Existing durable account/login/session/scoped-role acceptance stays required; no Cedar dependency or expanded preset release obligation is added |
| B84 rich local authorization | B41,B51,B83 | Administrator/Standard/Guest presets, scoped role inheritance/delegation, authenticated action-bound elevation, effective permissions inspector, safe policy activation/recovery and accessible audit/preview controls; standalone local acceptance possible |
| B85 enterprise identity | B41,B53,B83 | Named identity adapter and explicit enrollment, issuer/audience/tenant/group mapping, credential/key lifecycle, bounded offline behavior and offboarding |
| B86 enterprise policy/fleet | B49,B51,B84,B85 | Authenticated scoped remote management, signed/versioned policy distribution, rollback/stale-controller protection, delegated administrators and audit integrity |
| M27 / B87 enterprise acceptance | B32,B84,B85,B86 | Two independently administered tenants, positive and negative identity/policy/remote-management journeys plus partition/recovery/resource matrix; independent of AI federation |
| B88 workload/data-center authorization | B49,B53,B83,B85,B86 | Named workload bootstrap/rotation and controller interfaces; fleet/cluster/project/resource scopes, short-lived grants, separation of duties, headless typed actions and explicit trust/availability prerequisites |
| M28 / B89 data-center authorization acceptance | B32,B87,B88 | Actual original-guest enforcement against declared controller/identity services; compromised workload/controller, partition/replay/stale leader/offboarding and resource accounting; no server runtime/HA claim |

Host simulators may test unavailable controller/identity peers but cannot stand in
for original-guest enforcement or production infrastructure qualification. Do not
assume TPM/attestation, secure wall time, a credential vault, directory integration,
general server networking, a resident graph database or high availability exists.
Declare missing prerequisites and stop/replan the affected profile.

## Acceptance trace

These are future acceptance cases, not executed results. See
[validation](validation.md#authorization-profile-acceptance).

| Concern / existing requirement | Work | Required check |
| --- | --- | --- |
| Engine correctness and bounded work / R09,R10,R14 | B83 | Independent allow/deny/error and schema fixtures, forbid precedence, missing context, malformed policy, bounded graph/entity/cache pressure, original-guest ABI/runtime and exact dependency/license review |
| Accounts and presets / R10,R11,R15 | B41 existing; B84 additions | Rename/delete/recreate, guest logout/crash/cleanup, first owner, cross-user and same-user app isolation, keyboard/semantic workflows; no preset or identity string grants privilege |
| Roles and elevation / R10,R14,R19 | B84 | Cross-scope/tenant denial, role/group cycles, unauthorized role assignment, narrowed delegation, changed target/revision after preview, authentication expiry and lock/logout |
| Policy lifecycle / R10,R11,R12,R17 | B84,B86 | Interrupted durable activation, broker crash/restart/reload, old snapshot/revoked-policy restore, bounded cache expiry, in-flight outcomes and explicit surviving direct-handle authority |
| Explainability and audit / R15,R17,R19 | B84,B86 | Effective permissions agree with provider enforcement; unauthorized introspection denies; bounded secret-free audit events, remote audit loss/pressure and tamper/retention behavior |
| Enterprise trust / R10,R13,R14 | B85,B87 | Two tenants, forged/replayed/wrong-audience claims, nested group changes, enrollment/rotation/revoke/offboard, stale sync, issuer loss and offline lease/time rollback |
| Fleet administration / R10,R12,R17,R19 | B86,B87 | Restricted delegated admin, policy source precedence/mandatory ceilings, unauthorized local widening, stale/compromised controller, partition/reconnect and authorized emergency recovery |
| Workloads/data-center / R09,R10,R13,R14,R17 | B88,B89 | Workload versus human identity, deployment/operator/auditor separation, bootstrap/rotation, replay/expired grant, stale leader, cross-tenant resource access and controller compromise |
| Resource/continuity / R04,R05,R07,R09,R12 | B83,B84,B87,B89 | Predeclare limits; measure idle/default/active/peak memory and decision latency, admission/pressure/failure, supported broker replacement with fresh grants and unchanged boot identity; unavailable reload support remains an explicit gate |

M27 does not require M25 AI federation or local inference. M28 does not require
B80/M26 AI serving feasibility and cannot establish native server readiness.
A later approved server program must qualify its runtime, drivers, SMP, storage,
networking, HA and operations. Shared identity/credential work may be reused only
with exact tested scope and evidence; M7 passwords are not a general secret vault.
