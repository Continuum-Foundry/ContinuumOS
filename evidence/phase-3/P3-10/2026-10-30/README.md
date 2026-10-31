# P3-10 — Ephemeral scoped policy passes locally

**P3-10 is Done under local Codex self-review for its declared experimental subset.**
This is not external approval, real accounts or the M3 exit. See the
[criterion and unsafe review](review.json), [policy runbook](../../../../docs/05-delivery/phase-3/policy-runbook.md)
and [ADR-0026](../../../../docs/04-decisions/0026-ephemeral-scoped-policy-and-exact-forwarding-tickets.md), which remains Proposed.

P3-10 passes local self-review for its ephemeral scoped policy subset: 120 real IPC decisions, 67 expected denials, two broker faults/restarts, identical 160-byte replay across fresh process/session bindings and two exact resource settlements. All 82 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel/wait/supervisor regressions pass. The CPL3 broker/provider enforces exact scoped role assignments, child-grant attenuation, caller/service forwarding ceilings, finite sessions, use-time expiry/epoch/revision/context checks and synthetic privileged-scope separation. Queued grant and session expiry cross their actual elapsed fence; 136 focused evidence mutations reject. Trusted restoration is explicit and ordinary direct mappings survive broker death until owner teardown. No real accounts, credentials, durable RBAC, universal direct revocation, independently frozen SDK, guest UI/reload or desktop efficiency target is accepted; P3-11 is next.

## Artifact and source identity

Source base: `99fa118fa5474ea8945c661145f7e52e279f6e54` plus [dirty input hashes](source-provenance.json).
The owner's three pre-existing signing-policy edits remain unchanged and outside
this change. No certificate, host security policy or power setting changed.
Final documentation hashes are separate from the measured runtime/build snapshot.

Build ID: `e9a3f71dabfa1a018591f2816471bb2daa27678cd4e11296daf5095dc0f85169`.
Kernel SHA-256: `cf2e64641a2ca7efec1e8a3efcf301fb8cf3705fb2fada65949737be9f86689e`.
Launched ISO SHA-256: `d74a340494994bb936dcabd4420a64adbc397d5b58cbea382b4c859717a51a35`.
Full raw serial SHA-256: `6d5648de8ba3bfa371cac18445e3512e7d6ee47c42a92cdd4f3ffd67f0cebbd7`.

The three Rust modules are separately linked and run in six private address roots
per incarnation: broker/provider, trusted administrator, A, B, A2 and forwarding
service. They intentionally share fixture source; this does not freeze an
independent SDK. Kernel receive metadata supplies the actual sender. The immutable
boot roster identifies the only authorized policy administrator.

All raw policy serial bytes are retained in numbered `policy-serial-*.txt` chunks;
concatenate numerically to recover the full serial hash. [Launch/package context](launch-context.json)
records one-vCPU Q35/TCG/OVMF, isolated optical boot, unchanged 30000-ms guest bounds,
unchanged log limits and reaped host processes. Binaries, ISOs, full disassembly,
regression logs and source copies remain local under ignored build/work.

## Review and limits

The decision matrix is retained in review.json. Two broker #UD faults close their
channels. Clients verify their local direct RW mappings still work, then terminate.
Each incarnation settles object/handle/mapping/backing/frame/space/allocation/
table/stack/pin/wait inventories exactly. Fixed runtime precharge is 10581256
bytes, not whole-guest idle memory. The 256 MiB desktop target remains open.

The focused consumer rejects 136 evidence mutations. The aggregate also replays
the regression consumers. All 82 host tests and ten fresh prerequisites pass,
followed by all 33 M1 rows/42 launches and separate narrower regression builds/boots.
The eight [metadata checks](development-flavors.json) are developmental and never
replace those guest builds.

[Development findings](development-findings.json) preserve failed and superseded
probes, the restricted signer failure, formatting failure and explicit interruption.
A harness-passing polling probe was superseded because expiry could precede enqueue.
Final queued-expiry evidence checks an actual queued message before expiry, parks
using accepted waits/timers and consumes after expiry. No interrupted or failed probe
counts as a passing guest. M1's deliberate broken-before-entry fixtures retain
their separate expected-failure status. No deadline or log limit was raised.

The broker and provider share one CPL3 process and use bounded ephemeral arrays.
The fixture controller schedules requests; policy authority comes from actual IPC
provenance plus current binding/session/grant checks. Session leases last at most
60 seconds, with no authorization cache. The out-of-band fixture administrator
is instead bounded by its exact kernel binding lifetime; subject policy does not
revoke that trusted restoration root. Real administrative sessions remain M7.
Policy removal conservatively invalidates
all assignments and child grants. Exact single-use forwarding tickets require both
caller and service authority. Disk/account/process/diagnostic/service scopes remain
synthetic; no real privileged service or host/disk/account change occurs.

The use-time check is the authorization boundary: an admitted synchronous operation
may finish after subsequent caller death/expiry. Policy changes serialize with actions;
queued operations recheck, and completed commits are not rolled back. Direct kernel
handles/mappings retain their rights until close or owner teardown; there is no
universal revocation. Replacement needs fresh launch/session/epoch and explicit
trusted restoration. Checkpoints restore no authority. The supervisor regression
uses explicit Blocked-state barriers; its earlier coverage-incomplete trace remains
retained. Authentication, durable policy, production discovery/availability, independent
SDK freeze, live replacement, SMP and guest UI/reload remain open. P3-11 is next.
