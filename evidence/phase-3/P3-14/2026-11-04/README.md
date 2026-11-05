# P3-14 — Done with an owner-accepted host-fixture exception

All current guest/runtime enforcement and five aggregate failure probes pass local
Codex self-review. This is not independent human/security approval. The strict
full gate has **not passed**: the legacy P3-01 normal Python child repeatedly
misses its original one-second startup bound. Failed attempts are retained;
no timeout was extended and no failed row is relabeled passed.

Completed: ten prerequisites, 30 native host tests (one existing font ignore),
95 kernel host tests, complete independent M1 review (33 rows/42 launches),
memory/loader/boundary/scheduler/capability/channel/wait/supervision/policy/routes,
independent SDK and runtime hooks, codecs, 11 SDK and 14 stress build flavors,
portable-link checks, public supervision and all ten continuous stress guests.
The stress set completes 2000 worker lifecycles, 1000 route/fence cycles and
100000 measured IPC round trips plus 10000 warmups. All measured IPC samples
are retained. The six current P3-14 command-job tests also pass.

All five deliberate failures reject: broken ELF, wrong hash, wrong process
outcome, omitted completion at the consumer, and a real 1-ms QEMU timeout.
Their valid control passes and all launched children are reaped. The three
positive replays retain the original 30000-ms deadline and exact accepted ISO.

The owner requested a stop, then explicitly requested completing remaining checks.
Completed current-source cases were revalidated in an explicit continuation;
the interrupted aggregate remains interrupted. All guest/observer build input
hashes remain unchanged. The only later implementation amendment is the host
fixture's exact current-profile pin and collision-resistant run directory IDs.
Its original P3-01 profile fields retain their original canonical hash; only
already-approved P3-06/07/09 execution-subset fields explain the new whole hash.
Experimental startup changes that did not establish a reliable pass were reverted.

The [review](review.json), [runtime review](sdk-review.json),
[stress review](stress-review.json), [M1 review](m1-review.json),
[host failures](legacy-host-failures.json), [development findings](development-findings.json)
and [source lock](source-lock.json) identify actual results and limits.

The owner explicitly approved “Mark Done with this exception” for the legacy
host-lifecycle fixture. P3-14 is Done with that recorded exception; the strict M3
exit remains open. This is not a standing waiver. Failed checks retain their
failed classification. See [owner acceptance](owner-acceptance.json).

M4 guest UI/input/resources and authenticated reload, M5 state migration and
safe retirement, M6 durability, M7 real accounts/installation, the 256 MiB
cold-idle desktop and desktop latency targets remain open. Only the exact
existing SDK subset is frozen. One vCPU/one thread per process, masked SSE
exceptions, unverified native x87 debug metadata and ephemeral synthetic
policy/routes remain the admitted subset. No remote upload is authorized.
