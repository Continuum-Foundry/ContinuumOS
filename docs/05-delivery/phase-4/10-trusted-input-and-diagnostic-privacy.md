# P4-10 — Protect setup/login input and diagnostic privacy

Issue type: Story. Epic: P4. Status: Not started. Owner: Security/Input/Graphics.
Depends on: P4-09. Backlog: B16 trusted-input subset.
Requirements: R08/R10/R14/R15/R17. Test families: T05/T06/T10/T11.
Labels: `phase-4`, `trusted-input`, `privacy`.

## User story

As a future setup/login consumer, I need protected input/focus and capture policy
proved with synthetic data before real credentials are ever accepted.

## Implementation scope

1. Admit protected system surfaces only through trusted launch and narrow broker
   grants. Define focus entry/exit, modal routing and an OS-owned trust indication;
   ordinary app decorations, claimed roles or semantic IDs cannot acquire the grant.
2. Revoke/cancel ordinary capture before protected focus; reject background event
   observation/injection and unauthorized screen/semantic capture. Restrict clipboard
   access or return unsupported; no broad clipboard implementation is required.
3. Redact/exclude synthetic sensitive keys/text from ordinary logs, serial traces,
   inspectors, semantic values, debug exports, checkpoints and error payloads. Apply
   privacy before formatting, including failure paths and observer hooks.
4. Define broker/shell/input failure and protected-owner death behavior: no silent
   fallback sending sensitive input to ordinary apps. Keep keyboard-complete escape/
   recovery and visible focus without pretending a real login has succeeded.

## Acceptance criteria

- [ ] Only trusted protected-surface grants obtain protected focus; synthetic
  ordinary/same-user/other-user/dev-role attempts deny in the guest.
- [ ] Active capture, screenshots, semantic inspection and clipboard attempts cannot
  observe the synthetic protected stream or influence its routing.
- [ ] Canary text/keys are absent from all ordinary diagnostics/checkpoints/exports,
  including parse errors, crashes, timeouts and enabled instrumentation.
- [ ] Focus transitions, owner death and broker/input failure cancel safely and
  preserve an explicit usable recovery path without routing private input elsewhere.
- [ ] Keyboard/visible-focus tests and a documented M7 consumer/security handoff pass;
  no real password/account or trusted physical secure-attention claim is implied.

## Tests and evidence

Use distinctive synthetic canaries and an independent scan of full local diagnostic
outputs, not just retained summaries. Exercise forged protected flags, stale grants,
concurrent capture and owner restart. Retain redacted metadata/denials only.

## Exclusions and handoff

Real authentication, entropy/credential storage and installed setup remain M6/M7.
P4-13 must prove development credentials cannot acquire these protected roles.
Read [security model](../../02-architecture/security-and-trust.md#installer-and-account-boundaries).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
