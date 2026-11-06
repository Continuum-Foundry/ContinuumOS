# P4-13 — Authenticate development sessions and enforce authority

Issue type: Story. Epic: P4. Status: Not started. Owner: Security/Runtime.
Depends on: P4-12. Backlog: B15.
Requirements: R10/R13/R14/R17. Test families: T02/T06/T08/T11.
Labels: `phase-4`, `authentication`, `dev-authority`.

## User story

As the guest update service, I need authenticated session-scoped development
authority so replayed bytes and caller-supplied roles cannot authorize a change.

## Implementation scope

1. Select a maintained reviewed authentication construction for the exact target
   and transport. Verify primary documentation, pinned features/licenses, known
   vectors, entropy/nonce/key-lifetime needs and freestanding linking before use.
   Do not invent a handshake, use a plain digest as a MAC, or assume TLS exists.
2. Provision fresh session credentials through a declared trusted launch path using
   a verified entropy source. Keep credentials out of source, immutable reusable ISO,
   command-line/log/evidence dumps and ordinary app access. Boot/build IDs are
   correlation, not entropy/authentication. Define cleanup, expiry and revocation.
3. Authenticate header/payload/context, protocol/security requirements, boot/session
   binding, sequence and digest. Bind successful authentication to the actual M3
   caller/service context and narrowly granted L0/L1 development operations.
4. Recheck current session/policy at prepare and activation, fail closed on broker
   restart, and define reconnect/rotation without replay or privilege amplification.
   Keep development mode visible; disabled images reject mutating dev traffic.

## Acceptance criteria

- [ ] Exact-target primitive/handshake tests and independent vectors pass; protocol,
  entropy/provisioning, secret lifetime and threat assumptions receive explicit review.
- [ ] Fresh authenticated sessions authorize only their granted L0/L1 scope; bad
  credentials, altered header/payload/context, downgrade and pre-auth mutation deny.
- [ ] Replay within/across reconnect, boot and process/session incarnations fails;
  old credentials and revoked/expired policy cannot authorize later use.
- [ ] Forged owner/role, same-user other app, confused-deputy and development-to-
  protected-input/account/installer escalation fail through actual guest checks.
- [ ] Secret canaries remain absent from ordinary diagnostics/retained artifacts;
  dev-disabled profile has no update authority and UI continues after auth failure.

## Tests and evidence

Capture sanitized transcripts/decision codes and independent tamper/replay tests,
not keys. Exercise authority changes between preview/use, auth-service death,
quota/rate exhaustion, wrong-target boot and failed credential provisioning. A missing
secure construction or entropy path blocks this story; it is not a reason to bypass auth.

## Exclusions and handoff

No real desktop account or public distribution trust root. P4-14 receives current
session-bound authority and authenticated bytes. Read
[security model](../../02-architecture/security-and-trust.md#development-versus-distribution)
and [update security](../../03-live-update/failure-recovery-and-security.md#update-authority).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
