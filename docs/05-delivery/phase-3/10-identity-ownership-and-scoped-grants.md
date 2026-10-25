# P3-10 — Enforce scoped identity, ownership and policy grants

Issue type: Story. Epic: P3. Status: Not started. Owner: Unassigned.
Depends on: P3-09 Done. Suggested labels: `phase-3`, `identity`, `authorization`.
Backlog: B11–B13 authority completion, B15/B37 seeds.
Requirements: R10/R14/R18; preparatory R19. Test families: T03/T06/T11/T14 design/enforcement subset.

## User story

As a resource owner, I need every app and service to prove current scoped authority
so sharing works explicitly and neither ownership nor a role name grants ambient access.

## Implementation scope

1. Implement a minimal ephemeral policy broker with two synthetic user principals,
   two separate apps of user A, a service principal and trusted authority sessions.
   Bind records to kernel launch/IPC provenance. Specify ID/generation/session
   lifetime and trusted restoration; host boot-correlation IDs are not credentials.
2. Translate authorized scoped role assignments into explicit attenuated grants
   within broker delegation ceilings. Scope by resource/owner/session/operation and
   policy epoch. Restrict role administration itself; a user role never gives every
   app all of that user's authority. Cross-owner sharing requires an explicit grant.
3. Add a bounded synthetic resource/action provider exercising ownership, read/write,
   revision and context/deadline checks. Check both caller rights and service
   delegation on forwarded operations; reject confused-deputy use of the service's
   stronger authority. Payload owner/principal/session/role fields remain untrusted.
4. Enforce brokered session expiry and role/policy invalidation at mandatory use-time
   checks, including removal between preview and execution. Define queued/in-flight
   outcomes, maximum lease/cache delay and grant denial after invalidation. Document
   which ordinary direct handles/mappings remain valid and their teardown limits.
5. Reserve narrowly typed disk-installation, account-administration, file-operation,
   process/service-control and diagnostic scopes. Test denied synthetic disk/account
   requests from apps and development-session claims; an authorized synthetic narrow
   grant must not expand to other scopes. Do not implement the later services.
6. Fail closed after policy-broker death/restart until trusted state is restored.
   Emit bounded grant/deny/invalidation records without credentials or document data.
   Define fresh authorization for replacement processes; checkpoints cannot restore
   roles, credentials, privileged handles or old process bindings.

## Acceptance criteria

- [ ] Real independently addressed guest fixtures distinguish A, B, A's second app
  and the service; forged IDs/roles/session claims and stale process references fail.
- [ ] Cross-owner access is denied without a grant; explicit shared read succeeds
  while write fails; A's ungranted second app is denied despite the same owner.
- [ ] Scoped role changes, child delegation and service-on-behalf requests enforce
  both caller and broker/service ceilings, with no confused-deputy escalation.
- [ ] Expiry, revoked role, stale policy epoch and preview-to-execution policy change
  deny brokered operations at the declared check; in-flight outcomes are explicit.
- [ ] Ordinary apps/development claims cannot acquire installer/account authority;
  narrow positive fixtures cannot exercise unrelated privileged synthetic operations.
- [ ] Broker restart fails closed, surviving direct authority is enumerated, and
  replacement/checkpoint authority restoration is rejected without fresh grants.

## Tests and evidence

Retain a principal/app/service × owner/resource × operation × scope/session/epoch
decision matrix with positive and negative results from actual IPC. Include channel
delegation, unauthorized role assignment, wrong-resource role, child escalation,
service acting for a weaker caller, expiry while queued, policy removal at use time
and broker restart. Replay payload/context bytes against a new process generation.
Test synthetic privileged operations only; no real disk/account changes are needed.

## Exclusions and handoff

No account UI, credentials, authentication protocol, durable RBAC or universal
direct-handle revocation. P3-11 consumes current grants for route/fence operations;
M4 B15 binds authenticated dev sessions to this context; M5 B37 proves replacement
and UI/automation parity using fresh grants. Sources:
[identity plan](../../02-architecture/identity-and-authorization.md),
[action contract](../../06-contracts/actions-and-context.md),
[ADR-0010](../../04-decisions/0010-capability-security.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
