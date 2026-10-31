# P3-10 ephemeral scoped policy

Status: Done under local Codex self-review for the declared experimental subset. Prerequisite: the retained
P3-09 local review. The broker and synthetic action provider share one isolated
CPL3 Rust process. This is an experimental profile, not accounts or a production
service installation.

## Trust and lifetime

The trusted boot adapter launches six independently addressed processes: broker,
policy administrator, A, B, A's second app, and a forwarding service. A and A2
share principal/session but have distinct process bindings. The kernel maps an
explicit little-endian roster read-only in each fixture. Only the administrator's
exact process/generation/principal/session/service tuple may restore policy, lease
sessions, assign scoped roles, remove policy or invalidate sessions. No role name,
owner identifier, development claim, checkpoint field or boot correlation string
can select that tuple. This boot fixture is the trusted restoration mechanism;
there is no credential or durable policy restore protocol. The bootstrap
administrator is an out-of-band fixture restoration authority: its authority
session lasts only while its exact kernel launch binding is admitted. Ordinary
broker lease/role changes cannot revoke that root; process death ends it. The
at-most-60-second leases and monotonic session invalidations govern brokered
subject grants. Expiring credential-backed administrative sessions remain M7.


The reusable allocation-free policy core is compiled into the user broker. It
holds at most 32 assignments, 16 forwarding tickets, 16 session leases and 16
monotonic session invalidations. Role templates are Reader (1), Writer (2),
Controller (4), and explicit child delegation (8); combinations remain bounded
by the broker's resource/operation ceiling. Each assignment binds the complete
process tuple, one typed scope, resource, owner, operations, expiry and policy
epoch. Ownership alone grants nothing. Synthetic FILE resources are (1,A) and
(2,B); DISK, ACCOUNT, PROCESS, DIAGNOSTIC and SERVICE control scopes are distinct
and have only synthetic resource 1, owner A. No later service is implemented.

Kernel dispatch 34, `context_live(pointer,40)`, copies five little-endian u64
words (process slot, generation, principal, authority session, service 0/1).
It returns 1 only while that exact launch binding remains admitted, else 0. It
grants no rights, authenticates no payload and extends no session. The provider
uses it on trusted receive metadata at every decision; forwarded execution also
checks the original caller. Session leases last at most 60 elapsed seconds and
can only shorten in an incarnation. The fixture uses 20 seconds. Every grant,
preview and execution checks current session, lease and policy. No authorization
cache exists: maximum cache delay is zero provider decisions.

Policy removal increments a checked epoch and conservatively removes all role
assignments and tickets, including child grants. Session invalidation is monotonic
and rejects further grants in that incarnation. Child grants cannot expand their
parent's resource, owner, operation, session or expiry. A forwarding ticket binds
an authorized caller request to a specific service process, exact operation,
resource, owner, revision, epoch and deadline; it is single use. Execution checks
both caller and service grants. IDs are bounded references, not secret credentials.

## Use-time outcomes and restart

The provider validates all authority and expected revision immediately before
changing its bounded synthetic revision counter. This mandatory check is the
authorization boundary: an operation admitted there may finish if caller death
or elapsed expiry occurs immediately afterward. Requests still queued at removal
are denied. The broker processes policy changes serially with actions, so a role
removal processed before execution cannot be overtaken by that execution. There
is no claim of atomic kernel-liveness sampling plus a user-mode commit. Preview
creates no authority.
Queued requests are checked when consumed; expiry/removal before that check denies
without mutation. A completed commit stays completed when policy is removed later.
The single-threaded provider performs no asynchronous external effect or deferred
commit; no kernel continuation is retained. The counter is synthetic, ephemeral
and has no durability or document-data claim.

Broker death closes its channels. Clients fail closed and verify their ordinary
direct local RW mappings still work, then terminate. The next trusted bootstrap
creates fresh process generations, sessions and broker epoch with empty policy.
Even the administrator must explicitly restore and reassign grants. Replayed old
process references, grant/context bytes and checkpoint restoration attempts do not
restore authority. This is bounded restart testing, not live state migration.

Policy invalidation/death does not revoke ordinary kernel object handles, endpoint
duplicates, mappings or already committed effects. Those direct authorities obey
their original rights until explicitly closed or their owning process is torn
down; immutable backing can outlive its creator through retained references.
No universal direct-handle/mapping revocation is claimed.

## Reproduction

Keep the host awake. Run workloads serially in the authorized signer/xorriso
context, without changing deadlines or host power/security configuration:

```powershell
. ./tools/Enter-Dev.ps1
cargo xtask test --suite m1 --case policy
python tools/Review-P3-10.py --policy <printed-directory>
pwsh -NoProfile -File tools/Test-P3-10.ps1
python tools/Review-P3-10.py --acceptance <printed-acceptance.json>
```

The [experimental protocol](../../../contracts/policy-v0.json) is exactly twenty
little-endian u64 words (160 bytes), with protocol revision zero in word 19.
Unsupported revisions reject before policy lookup. The fixture controller
schedules each request; grant decisions still use the actual kernel-stamped
request sender. There is no general service discovery or independently frozen SDK.

The case builds three separately linked Rust user images, audits the kernel and
modules, packages an optical ISO and runs bounded offline QEMU. Two broker
incarnations exercise real IPC decisions, actual endpoint delegation, queued
expiry, identical 160-byte request replay with independently checked fingerprints,
direct mapping survival and exact teardown. Queued-expiry probes confirm an
actual queued message, record a sample before its 500-ms lease expires, park
until after expiry using the existing timer/wait path, then consume it. These
authorization test leases do not change the 30000-ms guest deadline or log limit. A separate metadata-only
eight-flavor development check is available through `Check-P3-10Flavors.ps1`;
it does not replace separately built guest regressions. Preserve all failures
locally, then promote only the reviewed compact evidence under the retention
policy. Independent SDK/codec freeze is P3-12; routes/fences, sustained stress,
guest UI/reload, authenticated development transport and real accounts remain open.

The required P3-09 regression uses registered Blocked-state observation and explicit
wait barriers before child/parent/supervisor teardown. A prior harness-passing run
missed one parked-owner case; its trace is retained as an incomplete coverage run.

## Accepted local observations — 2026-10-30

P3-10 passes local self-review for its ephemeral scoped policy subset: 120 real IPC decisions, 67 expected denials, two broker faults/restarts, identical 160-byte replay across fresh process/session bindings and two exact resource settlements. All 82 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel/wait/supervisor regressions pass. The CPL3 broker/provider enforces exact scoped role assignments, child-grant attenuation, caller/service forwarding ceilings, finite sessions, use-time expiry/epoch/revision/context checks and synthetic privileged-scope separation. Queued grant and session expiry cross their actual elapsed fence; 136 focused evidence mutations reject. Trusted restoration is explicit and ordinary direct mappings survive broker death until owner teardown. No real accounts, credentials, durable RBAC, universal direct revocation, independently frozen SDK, guest UI/reload or desktop efficiency target is accepted; P3-11 is next.

Fixed runtime precharge is 10581256 bytes, below 16 MiB; this is not whole-guest idle memory. See the [compact evidence](../../../evidence/phase-3/P3-10/2026-10-30/README.md). Full generated artifacts remain local.
