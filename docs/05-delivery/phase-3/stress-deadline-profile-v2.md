# Scoped TCG stress deadline, revision 2

Decision date: 2026-11-06, America/New_York. Scope: P4-01's current prerequisite
recheck of the two long M3 stress flavors. The owner explicitly authorized
investigation/repair and a deadline change if needed to continue P4-01. This is
a versioned bounded host-execution profile, not a performance-target waiver,
security certification or new SDK contract.

## Findings and decision

The original 30000-ms recheck stopped at wave 82 in the first trace-enabled
stress guest. A fresh single-pass backing-accounting calculation removes repeated
owner/queue scans without changing counters or authority. It matches the original
object-centric oracle across duplicate/moved handles, mappings, wrapped queues,
slot generations, creator death and exact teardown. All 96 kernel host tests and
fourteen separate metadata flavors pass the focused check.

The optimized 30000-ms trace probe still stopped after 88 complete waves. A later
fresh complete gate stopped in the trace-disabled guest near wave 91. In contrast,
an exact-ISO diagnostic replay of the optimized trace guest completed in
24120.695 ms, using 22937.5 ms of QEMU process CPU time. Its 60-second diagnostic
cap was not acceptance. Independent raw review verifies all 100 waves, 200 worker
lifecycles, 10000 measured round trips, outcomes and settlements in that replay.

These observations demonstrate variable completion cost and substantial TCG CPU
work. They do not establish a particular host scheduling, thermal, security-tool
or power cause. The 30000-ms bound has repeatedly prevented completion of the
unchanged fixed-count correctness workload on this host. Retain the accounting
improvement and admit a separate 60000-ms wall bound for those two long stress
flavors, with the complete fixed workload and guest-local timing checks intact.
This permits the required repeatable regression experiment without host-policy,
power-setting, sample-count or quality changes. It does not claim that arbitrary
host load can meet any wall-time bound.

Follow-up inspection corrects dispatch observation after blocking: use the
wait-completion timestamp for a woken owner, rather than its earlier return when
it became blocked. Legitimate blocked time is excluded; every actual ready delay
remains sampled and subject to the unchanged 320-ms bound. A dedicated host oracle
tests both long blocking with short ready delay and genuinely excessive ready
delay. The full kernel suite now has 97 tests. New bound-failure records identify
kind/actor/wave/value without clipping or suppressing a failure. The earlier
line-70 panic remains a failed run with an unavailable exact sample; the later
observer repair is not claimed as proof of that panic's cause.

## Exact containment

| Consumer or test | Deadline / invariant |
| --- | --- |
| `stress-test`, `stress-trace` only | 60000 ms under `m3-stress-tcg-v2`, schema 2 |
| Ordinary M1, memory/loader/boundary/scheduler/capability/IPC/policy/route/SDK guests | Existing 30000 ms |
| Separate public supervision stress fixture | Existing 30000 ms |
| M3 SDK replay controls and wrong-outcome/missing-record probes | Existing 30000 ms |
| Deliberate negative guest deadline | Existing 1 ms |
| Normal / negative fixed host lifecycle children | Existing 1000 / 100 ms |
| Guest fairness, wake, dispatch, IRQ-off and IPC round-trip limits | Existing 320000000 ns |
| Stress counts | Five trace-off/on pairs, 100 waves and 10000 measured + 1000 warmup round trips/guest |
| Inventory retention tolerances | Zero retained frames/objects; exact existing settlements |
| Desktop targets | Unchanged 256 MiB cold idle, 8/16-ms CPU tails, 25-ms input and 500-ms resource visibility targets |

The ordinary [M1 input lock](../../../config/m1-inputs.lock.json) and historical
[M3 baseline profile](../../../config/m3-profiles.json) stay unchanged. The
[stress source](../../../config/p3-13-stress.json),
[aggregate input](../../../config/p3-14-acceptance.json) and
[M4 experiment profile](../../../config/m4-profiles.json) name the new scoped
deadline explicitly. Existing kernels still expose the same SDK ABI and samples.

The launcher selects the longer bound from the trusted packaged build flavor,
checks exact supported profile values, and verifies its SHA-256 against the
kernel build's source inputs before launch. The launch manifest records the
profile name/hash and original deadline. Independent reviewers check build,
package and launch flavor agreement, current profile identity and exact deadline.
An ordinary guest cannot acquire the stress deadline merely by adding a manifest
field. Unit tests exercise allowed flavors, malformed profiles, wrong types and
unchanged ordinary/public-supervision limits. Full M1/M3 acceptance still runs.

## Interpretation and retained failures

The 2026-11-06 historical strict M3 result remains an original-limit pass for its
own source/artifacts. All later 30000-ms failures remain failed. Current revision-2
reports must say `original_limit_acceptance: false` and name both the ordinary
30000-ms and long-stress 60000-ms bounds; passing revision 2 cannot retroactively
pass revision 1. A missing completion, exhausted guest-local bound, malformed
artifact, incorrect inventory or 60000-ms timeout still fails with no automatic
retry. No standing waiver is introduced.

Full local records are under:

- `build/m3/P3-14/acceptance-1794009093298631100/`: initial failed baseline.
- `work/p4-01-accounting-fix/`: host oracle benchmark, focused build/test records,
  the optimized failed trace probe and exact-ISO CPU replay.
- `build/m4/P4-01/acceptance-1794013637127735200/`: later failed original-limit
  full gate, nested M3 run and trace-disabled failure.

The final complete gate, `pwsh -NoProfile -File tools/Test-P4-01.ps1`, and its
source-locked reviewer pass. See the [P4-01 evidence](../../../evidence/phase-4/P4-01/final-1794022944926085700/README.md).
All ten final stress case intervals are 45389–51188 ms, demonstrating why this
is revision-2 acceptance rather than an original-limit pass.
Full logs/ISOs/source copies stay local. M4 rendering/input/device/authentication,
P4-02 capacity admission and all desktop performance acceptance remain separate.
