# P3-11 — Ephemeral service routes and writer fencing pass locally

**P3-11 is Done under local Codex self-review for its declared synthetic subset.**
This is not external approval or the M3 exit. See [criterion and unsafe review](review.json),
[runbook](../../../../docs/05-delivery/phase-3/routes-runbook.md) and
[ADR-0027](../../../../docs/04-decisions/0027-ephemeral-service-routes-and-mandatory-writer-fences.md), which remains Proposed.

P3-11 passes local self-review for bounded ephemeral service routes and writer fencing: 131 real decisions, 25 authoritative mutation attempts, 19 commits, 54 process runs across nine incarnations, eight injected faults and nine exact settlements. The guest separates fence, writer grant and publication while both services are alive; delayed old-epoch and retained old-endpoint writes deny. Bounded queue/result retention, duplicate/reorder/cancellation/reply outcomes, current policy/session checks and trusted fresh bootstrap pass. All 93 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and nine separate prior runtime regressions pass. The focused reviewer rejects 117 evidence mutations. Independent private RW mappings retain their ordinary lifetime until owner teardown. This accepts no durable activation, state migration, safe old-code retirement, independent SDK freeze, guest UI/reload or desktop efficiency target; P3-12 is next.

## Exact artifact and source identity

Source base: `c780d4d2c8597ec7f24b9db74e4a0d53c1d39bea` plus [measured dirty-input hashes](source-provenance.json).
The three pre-existing owner signing-policy edits are preserved and excluded from staging.
Runtime/build inputs are frozen separately from final documentation hashes.

Build ID: `47f90c881cb63ad4c82d47ceab387281ff96ecd538c6963da8e99a8eb4900f01`.
Kernel SHA-256: `2be6c5266aed12506192235eeaca7d4f41c723dfe9313fac847cddd351415060`.
Launched ISO SHA-256: `dc56a836f08e3e60cd2da391f6c7914d3886690907095f892317c5a2bd8febc7`.
Full raw route serial SHA-256: `6ab80c074bb3136a5549cdaaa11556a87e1b9b0eb9d6b1201a7bba83dffe0d87`.

Concatenate numbered `routes-serial-*.txt` files numerically for the exact complete
serial stream of the declared diagnostic profile. Routine IPC/capability calls,
wait park/wake/resume/completion and clock-sampling calls are deliberately omitted
by the registered route fixture hook; sender metadata, timer arms/waits, authority
state/decision/commit/reply, process faults and settlements remain. No deadline or
log-size limit changed. Full artifacts, regression logs, disassembly and development
traces remain local in ignored build/work. [Launch context](launch-context.json)
records one-vCPU Q35/TCG/OVMF optical boot, original 30000-ms guest bounds, exact
package bytes and reaped host processes. Fixed runtime precharge is
10581256 bytes, not whole-guest idle memory; the 256 MiB desktop target remains open.

## Criterion evidence

| Criterion | Exercised evidence |
| --- | --- |
| Authorized compatible routing | Real client IPC; unauthorized control case 6, unavailable-before-grant/publication cases 58/59; incompatible major/features/service 7/8/54; initial publication-before-grant rejection |
| Mandatory single writer | Wave 0 case 25 explicitly records fence/no-writer, candidate grant and publication before delayed old commit; retained direct endpoint case 26 denies, new instance case 27 commits; both service lifetimes overlap |
| Explicit request/reply outcomes | Queue pressure 16–20, pending duplicate 56, completed duplicate 11, changed duplicate 12, reorder 14, cancellation 21 and postcommit cancellation 57, elapsed result retention 51/52, stale/correlation reply rejection 25/29 |
| Current authority | Actual sender/origin/service checks; queued policy invalidation 32, fresh reauthorization 33/35, stale policy 34, session invalidation 36/37; identical serialized old authority case 53 across eight fresh bindings |
| Failure cuts and recovery | Authority deaths before fence, after fence, after grant, after publication and after commit; service death before commit; proxy deaths before commit and after commit before reply; explicit rejection or unknown result, fresh bootstrap and nine exact settlements |

The focused raw-evidence consumer rejects 117 mutations; aggregate review also
runs the prior story consumers. All 93 kernel host tests pass, including eleven
route tests. Ten prerequisites and complete M1 (33 rows/42 launches) pass with
separate ordinary/boundary/scheduler/capability and later runtime build/boot paths.
Metadata-only flavor checks supplement those actual guests.

## Limits and retained findings

The counter, policy, routes and results are ephemeral in one CPL3 authority;
proxy, client and both service instances have separate private address spaces.
The trusted fixture controller supplies the failure schedule. Result retention is
one elapsed second with four pending requests, eight results and four non-evicting
caller high-water marks. Exact completed duplicates query historical results;
current routed replies must match current correlation/epoch. Commit denials leave
state unchanged; reply rejection after a commit does not undo that commit.

The synchronous check is the authorization boundary; an admitted operation may
finish after later expiry/death. Policy and route changes serialize at the owner.
Authority death loses ephemeral counter/results and leaves an in-flight outcome
unknown. The fixture never blindly replays it. Proxy loss can leave a published
route unavailable until trusted bootstrap. New bindings need explicit fresh grants.
Independent private RW mappings retain their kernel lifetime through broker death,
then owner teardown reclaims them. There is no resource RW alias or universal
handle/mapping revocation. No credentials, durable RBAC, general discovery,
independent SDK freeze, state migration, safe retirement, durable activation,
external exactly-once effects, SMP, guest UI/reload or desktop efficiency claim follows.

[Development findings](development-findings.json) retain the restricted signer
failure, initial host child-start timeout, compile-hook mismatch and rejected
fixture probes. A timing-dependent full-result assumption was removed; deterministic
host capacity and actual guest queue pressure remain. Session reauthorization uses
the original finite lease. Active-chain authority death and dead-proxy handling
were corrected before final acceptance. No interrupted/failed probe counts as a
passing guest. The final serial host run passes the unchanged child-start fixture.
M1's deliberate before-entry failures retain their expected-failure classification.
