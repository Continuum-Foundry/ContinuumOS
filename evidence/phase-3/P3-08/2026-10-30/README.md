# P3-08 — Waits, timers and cancellation pass locally

**P3-08 is Done under local Codex self-review for its declared experimental subset.**
This is not external approval or the M3 exit. See the [criterion/unsafe review](review.json),
[waits runbook](../../../../docs/05-delivery/phase-3/waits-runbook.md) and
[ADR-0024](../../../../docs/04-decisions/0024-elapsed-clock-and-bounded-wait-completion.md), which remains Proposed.

P3-08 passes local self-review for bounded waits, elapsed HPET deadlines, events/timers, cancellation and blocking channel helpers. The 18-task/six-wave guest proves registration-boundary notifications, immutable completion, pressured transfer ownership, bounded request cycles, parked-owner termination and exact settlements. All 64 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel regressions pass. General supervision, policy/session liveness, independent SDK freeze and guest UI remain open; P3-09 is next.

The guest uses real CPL3 clients, copied little-endian records and the shared validated
return boundary. Registered completion is latched before a caller can signal then
reset readiness in another syscall. Runnable admission preserves that immutable
outcome. Closed watched handles, cancellation, elapsed deadlines and readiness have
explicit precedence; termination discards the owner's wait before teardown.
Readiness reserves no bytes, handles or queue capacity. Channel helpers compose
atomic attempts and waits with the same absolute deadline. Timeout/cancel preserves
ownership before commit; accepted enqueue does not confirm service effects.

## Exact observations and artifacts

Source base: `fbfb89100fa19a488df8ded620a9ffaaa9483af9` plus [dirty input hashes](source-provenance.json).
The tested source includes the owner's three pre-existing signing-policy edits;
they remain outside this change. No host policy was deployed. Final documentation
hashes are recorded separately from the measured runtime/build snapshot.

Build ID: `38da373e53e65a90feef9ad05cb39654c2e8d25fb663e96217d85bdc4ce3a982`.
Kernel SHA-256: `e439625ea5108cb239fae614e8b4a2740667e990ca2f29dea9e189ce7a6bdeee`.
Launched ISO SHA-256: `63d78cf9ff5003e7b3c404ef894cf9f6ac4342c265f12f658c71436b15dcf282`.
Raw serial SHA-256: `1044658ae1027b088705f2a57a674d79816406306b0629a787e7cc9f8cbee9f0`.

All raw wait serial bytes are retained in numbered `waits-serial-*.txt` chunks;
concatenate them numerically to recover that hash. No raw records are omitted.
Full binaries, ISOs, disassembly and generated logs remain local under ignored
build/work. [Launch/package context](launch-context.json) preserves actual arguments,
one-vCPU Q35/TCG/OVMF, isolated optical boot, unchanged deadlines and reaped children.

The focused guest records 64 parks, 63 admission wakes,
1 parked-owner kill and 586 wait-family calls, with six
zero object/frame/space/table/stack/pin settlements. Fixed-runtime conservative
precharge is 10581272 bytes, not whole-guest idle memory.
The HPET resolution is 10 ns. Across 100 reads, observer total
was 333700 ns. IRQ-off elapsed comparison advanced
30128700 ns while delivered ticks stayed unchanged.
Maximum observed readiness-to-dispatch-preparation interval was
19155400 ns, measured before diagnostic output and IRETQ;
it is not first user-instruction, input/display or hard-real-time latency.

The combined independent raw/artifact consumer rejects 157
evidence mutations. Earlier [development failures and interruptions](development-findings.json)
remain explicit and do not count as acceptance. All 64 host tests and the complete
33-row/42-launch M1 gate pass, including clean kernel/ISO rebuilds, fault diagnostics,
memory protection, timer/uptime, separate original boundary/scheduler/capability and
channel profiles. The ordinary and each narrower feature profile build separately.

## Limits

One CPU, one owner thread per process, 32 wait records and eight items per set.
Events and timers are level-ready; reset/cancel/rearm changes future observations.
Controllers retain cancellation requests until acknowledgement; resets cannot erase
already recorded completion. FIFO simultaneous runnable admission does not reserve
channel resources or promise priority inheritance. Explicit timeout/cancel/termination
bounds demonstrated application cycles; no universal deadlock detector is claimed.
No SMP, wall-clock, public async executor, general process-tree supervision, independent
SDK freeze, current policy/session enforcement, guest UI/reload or desktop efficiency
acceptance follows. P3-05 FP/debug limitations remain.
