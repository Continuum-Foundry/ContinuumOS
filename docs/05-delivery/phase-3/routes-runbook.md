# P3-11 service-route and writer-fence experiment

Status: the declared P3-11 subset passes local self-review; see the
[acceptance evidence](../../../evidence/phase-3/P3-11/2026-10-31/README.md).
The [story](11-proxy-routes-epochs-and-writer-fencing.md) and
[ADR-0027](../../04-decisions/0027-ephemeral-service-routes-and-mandatory-writer-fences.md)
define the bounded synthetic subset. M5/M6 retain state migration, retirement,
durable activation and external-effect recovery.

## Implementation and commands

`kernel/src/routes.rs` is an allocation-free authority state machine, also compiled
into the user fixture. `experiments/user-images/routes.rs` uses explicit 160-byte
little-endian messages and real public syscalls. `kernel/src/routes_guest.rs`
only bootstraps private address spaces/channels/readonly rosters and schedules,
observes and reaps them using the registered fixture hooks. Public operations
cannot select privileged return state. No new syscall is introduced.

Dot-source `tools/Enter-Dev.ps1`. Keep the host awake. Run acceptance serially in
the existing approved signer/xorriso execution context, with network offline:

```powershell
cargo xtask test --suite m1 --case routes
python tools/Review-P3-11.py --routes <printed-artifact-directory>
pwsh -NoProfile -File tools/Check-P3-11Flavors.ps1 -Descriptors <artifact-directory> -Output work/p3-11-flavors
pwsh -NoProfile -File tools/Test-P3-11.ps1
python tools/Review-P3-11.py --acceptance <printed-acceptance-json> --destination evidence/phase-3/P3-11/<run-id>
```

The flavor helper checks metadata separately for ordinary, user-boundary,
scheduler, capability, channel, wait, supervision, policy and route builds. It is
developmental evidence. Each xtask case still builds, packages and executes a
guest; the helper does not replace those regressions. Final acceptance rechecks
ten prerequisites, all M1 rows and the separate prior-story cases before routes.
Never count an interrupted probe, host preflight timeout or guest deadline failure
as acceptance; preserve its distinct context and leave all limits unchanged.

## Authority and transport

Six private processes are authority/policy/resource owner (0), trusted fixture
administrator (1), proxy (2), client (3), old service (4) and candidate (5).
The bootstrap installs exact generation/principal/session/class rosters read-only.
Five authority channels, two proxy/service paths and one retained client/old path
use the existing eight-pair transport ceiling. Numeric logical service ID 1 is
neither a raw endpoint nor authority. Knowing the name does not grant access.
Protocol, major and mandatory-feature mismatch report IncompatibleProtocol.

The client identity comes from actual receive metadata. Claimed fields must match
that identity. Service commits arrive from the actual service process, not its
endpoint creator or a serialized identity. Current binding queries supplement
P3-10's finite scoped grants/sessions at admission and use. The administrator is
the exact trusted bootstrap binding; product administration remains M7.

The authority owns the only mutable counter. A service receives no direct mapping
of that state. Writer authority is an exact instance/epoch lease; it is never
conferred by a channel, publication, or grant ID alone. Fence clears the writer
and increments epoch before another writer can be granted. Publication is a
separate step. Every mutation requires the current writer, forwarded request,
epoch and current origin/service policy. No policy cache bypasses the commit check.
The synchronous commit check is the authorization boundary: an admitted operation
may finish after subsequent elapsed expiry or caller death. Policy and route
changes serialize in the single-threaded authority; queued operations recheck
when forwarded and committed. This is not instruction-by-instruction revocation.

## Request outcomes and bounds

The [draft wire profile](../../../contracts/routes-v0.json) records word offsets.
Fixture control commands are test-only; P3-12 must independently exercise the
admitted protocol before freezing any SDK. Native Rust layouts never cross IPC.

Admission bounds four requests (640 wire bytes), eight eventual results and four
non-evicting caller high-water marks. A full queue/result inventory rejects with
QuotaExceeded; it does not silently drop work. A pending exact duplicate reports
WouldBlock. An exact completed duplicate returns its immutable result until the
one-second elapsed retention deadline; altered duplicates deny. After expiry,
lower/equal IDs remain stale for that caller incarnation. A request deadline is
checked again at forwarding/commit. Policy/session invalidation while pending
cannot borrow a new route. Reconnecting requires the current epoch and explicit
fresh authorization. A result cache is not an authority cache.

Forwarding selects the current published writer and binds the request to it.
Commit is the sole synthetic effect. Commit denials preserve value/revision and
produce an explicit retained result. Cancel before commit records Cancelled; after commit
it returns the recorded outcome. Reply validation binds the entire request,
caller, service/protocol/version, ID and epoch, and checks the authority result.
A stale/reordered reply cannot become a new result. Reply rejection can follow a
successful commit: the correlation-failure case deliberately advances the counter
and rejects the returned correlation. It does not roll back the committed result.
Exact completed duplicates are historical result queries after current caller
policy/deadline checks; they may retrieve their original epoch's retained result.
They never forward to the new writer. A reply to current routed execution must
match its current request and epoch. Cancellation, retry and a lost reply never
prove nonexecution of arbitrary external operations.

## Fault and direct-authority inventory

The deterministic guest schedule covers authority death before fence, after
fence, after candidate grant, after publication and after commit before reply;
service death after forwarding; and proxy death before commit or after commit.
An authority that survives rejects pending work explicitly, or returns the
retained committed result after reply loss. A dead authority closes its channels;
unknown outcomes are explicit and never automatically replayed. Trusted bootstrap
starts an empty resource/policy incarnation, with fresh process/session bindings.
Literal old serialized authority is rejected. No durable counter survives this
restart, and no state-preserving reload is claimed.

Retained old service endpoints stay connected during the successful switch.
Delayed old-epoch writes and old-instance writes using the new epoch both fail
at the authority. Each worker also holds an independent private RW mapping;
it remains accessible through route/policy death and is reclaimed on owner exit.
There are no resource RW aliases, device grants or shared mutable resource rings.
Kernel handles, duplicates and mappings keep their documented direct lifetime;
this experiment adds no universal direct revocation.

The independent reviewer consumes raw correlated serial records, exact module/ISO
bytes, private-root lifetimes, sender metadata, transition ordering, request IDs,
counter revisions and complete teardown inventories. Mutated evidence must reject.
Full logs, artifacts and disassembly stay local; promote only a compact reviewed
bundle under the existing evidence policy. Runtime accounting is not whole-guest
cold-idle memory and does not accept the 256 MiB desktop target.

The route fixture deliberately omits routine IPC/capability call records and
wait park/wake/resume/completion records through its registered record hook.
It omits routine clock sampling calls as well; elapsed expiry records and actual
timer-arm/wait calls remain. It retains sender metadata, authority/state/commit
records, faults and settlements. Other flavors keep their original complete
trace. The guest deadline and 2 MiB serial limit remain unchanged.

## Accepted local result

P3-11 passes local self-review for bounded ephemeral service routes and writer fencing: 131 real decisions, 25 authoritative mutation attempts, 19 commits, 54 process runs across nine incarnations, eight injected faults and nine exact settlements. The guest separates fence, writer grant and publication while both services are alive; delayed old-epoch and retained old-endpoint writes deny. Bounded queue/result retention, duplicate/reorder/cancellation/reply outcomes, current policy/session checks and trusted fresh bootstrap pass. All 93 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and nine separate prior runtime regressions pass. The focused reviewer rejects 117 evidence mutations. Independent private RW mappings retain their ordinary lifetime until owner teardown. This accepts no durable activation, state migration, safe old-code retirement, independent SDK freeze, guest UI/reload or desktop efficiency target; P3-12 is next.
