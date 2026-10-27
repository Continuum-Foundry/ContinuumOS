# P3-05 - Preemptive owner tasks and eager CPU context pass locally

**P3-05 is Done under local Codex self-review for its declared experimental
subset.** This is not external approval or the M3 exit. Read the
[criterion/unsafe review](review.json) and
[scheduler runbook](../../../../docs/05-delivery/phase-3/scheduler-runbook.md).

Two fresh offline optical boots execute 60 task lifecycles in twenty three-owner
waves. Forty non-yielding ring-3 hogs retain integer/arithmetic/DF/stack/private-VA
and x87/XMM/control sentinels through sixteen dispatches before trusted termination.
Healthy controls, faults, syscall/copy-return kill and same-boot slot reuse pass.
A third optical boot removes required SSE2 and is rejected before any user entry.
All space/allocation/commit/resident/mapping/table/stack/pin/frame inventories
return exactly to baseline after each wave. Empty-queue halt resumes on a real IRQ.

## Exact artifacts and observations

Final scheduler run: `${WORKSPACE}\build\m1\scheduler-1793142880744692500-50192`.
Source base: `bbfe4d3e84d7bd098a56a0c7e7f25919ae272047` plus [dirty input hashes](source-provenance.json).
The input snapshot includes the owner's pre-existing Windows-policy edits;
those are outside the P3-05 staged change. Tests used the existing signer and
performed no host policy deployment. Full ISOs/binaries, patches, generated
source copies and disassembly remain local under ignored build/work.

Build ID: `e48b284f45f3229a912df1cf254ba498759f514b926bef97a5fd8e10a101536f`.
Kernel SHA-256: `2191369dc8e79fbe973bfdb1de6b1b0a3c3b1ec88c9b646998959ae0757ce3d2`.

| Scheduler boot | Exact launched ISO SHA-256 | Host lifecycle ms |
| --- | --- | --- |
| on | `80d39db3e07f794c54b0354f0bced233e4b9301037123faf934b7d9fc416cb17` | 12750 |
| off | `ed7f0b09d681f0a139ba77ac866bf81fa741b0899b684d716ea6f54b668202c8` | 12646 |
| cpu | `949a81af5043bf9a9ca5fd73fe3cf905976d36a9417c8d4b189343f04b66be7c` | 7626 |

The normal profile is pinned Q35/TCG, one vCPU, 2 GiB capacity, no NIC/shared
filesystem, fresh firmware variables and the original 30000-ms deadline. Only
the declared CPU-negative launch changes the CPU argument by adding `-sse2`.
One kernel entry per guest and correlated build/session records are checked;
all child processes are reaped. Host lifecycle is not guest CPU performance.

| Diagnostic observation | Trace on | Trace off |
| --- | ---: | ---: |
| Dispatches | 379 | 379 |
| Maximum runnable wait, ns | 50601700 | 40703800 |
| Conservative masked-interval upper bound, ns | 28674100 | 31685500 |
| Rust trap/syscall body maximum, ns | 13298400 | 16125300 |
| Nested kernel IRQs / pin-held IRQs | 30 / 25 | 29 / 25 |
| 100 observer reads, total ns | 483300 | 451300 |

These diagnostic elapsed observations include TCG/host and observer effects.
They remain below the locked 64-quanta / 320-ms ceiling. No cost is subtracted.
Whole dispatch residence conservatively includes user execution and assembly;
it bounds masked subintervals and is not an exact IF-transition or CPU-time trace.
Delivered-tick uptime remains distinct from the diagnostic HPET clock.

## Gate and limits

All ten final-source prerequisites pass. Complete M1 passes 33 rows/42 launches,
including five normal boots, both clean optical boots, fatal/IST, timer and
negative cases. Clean kernel bytes match; ISO bytes differ only in the permitted
session field. The 1000-cycle owned-memory, static-loader and four P3-04 user
boundary regressions pass independent raw audits. All 34 kernel host tests pass.
The reviewer rejects 100 evidence mutations.
Selected failed-development traces are [preserved](development-findings.json).
This is not release reproducibility, desktop efficiency or a 256-MiB idle result.

The subset uses equal-priority FIFO and one owning thread per process, with
32 queue slots and 16 process backings. There is no public thread creation,
priority elevation, shared-process threading, blocking kernel continuation or
SMP. Syscalls finish before dispatch; pin-held IRQs only mark pending. Map/free
publication saves the active CR3, and teardown runs on the surviving root/stack.
FP data are private aligned eager FXSAVE64/FXRSTOR64 images; kernel arithmetic
remains integer-only and guarded.

**Native unmasked SSE trapping (#XM) is not accepted.** The actual DIVSS probe
did not trap on pinned TCG. All six SSE exception masks are required; clearing
any mask terminates at the next kernel boundary. This software policy is not
relabeled as a hardware exception. Real x87 #MF, unsupported AVX #UD and CLI #GP
are exercised. Nonzero native x87 opcode/IP/DP debug metadata are also not
execution-validated: the emulator stores zero metadata. Debug/TLS/XSAVE/AVX
features remain unsupported; hardware RF is normalized on IRQ capture only.
A runtime needing unmasked SSE traps or native debug metadata needs another gate.
No third-party math/renderer is admitted by this result.

Next is P3-06. Public memory/handle rights, blocking IPC, general supervision,
SDK freeze, guest UI/reload and broader M3 acceptance remain open. ADR-0021
remains Proposed. Historical evidence bundles are unchanged.
