# P3-05 preemption and CPU-context experiment

One CPU; original Rust kernel; pinned Q35/TCG optical boot; 2 GiB capacity,
unchanged 30000 ms launch deadline, no NIC or shared host filesystem. No UI runs.
The [story](05-preemptive-scheduling-and-cpu-context.md) and
[ADR-0021](../../04-decisions/0021-preemptive-owner-threads-and-eager-fp.md) define
scope. P3-04's separate integer-only/#NM regression stays available.

Run from the workspace in PowerShell, with the existing development signer:

```powershell
pwsh -NoProfile -File tools/Test-P3-05.ps1
python tools/Review-P3-05.py --acceptance build/m3/P3-05/acceptance-<run>/acceptance.json
```

Focused bring-up: dot-source `tools/Enter-Dev.ps1`, then
`cargo xtask test --suite m1 --case scheduler`. Generated products remain under
ignored build/work. xorriso and certificate access require the existing host
execution context outside the sandbox. Do not install or deploy a host policy.

## Dispatch and unsafe boundary

The queue permits 32 thread identities; the current backend admits one owner per
process and 16 processes. No public create/priority API exists. Unsupported
priority requests fail; there is no hidden service boost. New/requeued tasks
join the tail. With N continuously runnable tasks, rotation visits each within
N+1 opportunities. The fixed acceptance ceiling is 64 nominal quanta (320 ms).

`task_resume` saves the surviving supervisor SysV continuation and kernel root,
installs the current owned root and guarded entry stack, copies a checked 160-byte
frame, restores eager FP state, zeroes data selectors and enters through IRETQ.
TSS RSP0 and syscall stack belong to that owner. FS/GS bases reset on each dispatch;
TLS/FSGSBASE are unsupported. Map/free may publish another page-table bank; capture
saves the actual active CR3 rather than retaining a retired bank.

The returning IRQ wrapper saves all 15 GPRs, saves CR0, sets TS, clears DF, aligns
the Rust call and passes the exact frame. LAPIC acknowledgement precedes scheduling.
A user timer captures private GPR/IRET and FP state into fixed supervisor storage;
it performs no allocation, logging, memory-table walk or queue operation. It
returns directly to the surviving stack/root. Hardware-frame RF is normalized because debug resumption is unsupported; syscall
return validation remains strict. Saved return permissions/flags are
checked outside hard IRQ before reentry. Unsupported return states terminate
locally. The original M1 IRQ path restores its saved CR0 and all registers.

Kernel syscall/exception code runs to completion. IF is masked around structural
memory changes. The test briefly enables a timer with a live copy pin; the kernel
IRQ can only update counters/pending, and cannot inspect or change memory state.
Copy unpins before switching. Yield and syscall return are safe points; exit/fault
returns to the supervisor before reclaim. Registry/memory work precedes queue
operations, with no structural lock spanning a user wait. Normal IRQ depth is
one; test IRQs nested in a syscall defer rather than recursively dispatch.

Only `task_save_fp` and `task_resume` may emit kernel FX instructions. Buffers are
private, 16-byte aligned and 512 bytes; no user memory is accepted as a restore
image. x87 register payloads/tags/status/control/IP/DP, MXCSR and XMM0–15 are saved
eagerly. Fresh state contains cleared payloads and default controls. Unsupported
required feature bits reject before staging. CPUID FPU/FXSR/SSE/SSE2 and disabled
OSXSAVE are checked by the shared admission policy; host negatives remove each
required bit. AVX and x87 faults, and unsupported SSE control settings, are process-local. Kernel
FP arithmetic remains an instruction-audit failure. No broad third-party runtime
or arbitrary CPU-dispatch library is admitted.

## Workloads and observations

Both trace-on and trace-off boots run ten waves of three distinct processes at
identical user VAs. Two non-yielding hogs retain all GPRs, arithmetic/DF flags,
stack/private progress and different x87/XMM/control images through 16 dispatches
before trusted termination. They perform no syscall/yield on their successful
path. A control process tests ordinary return, pinned discovery copies, yield,
allocation/free, unsupported operations, x87 exceptions, unsupported SSE controls, AVX, CLI, fresh state,
kill at syscall return and a noncanonical saved stack. Faulted/killed slots are
reused in the same boot, with fresh FP state and increasing process generations.
Other runnable owners continue after every offender. Each wave returns every
space/allocation/mapping/table/stack/pin/frame inventory to its starting value.
The final empty queue executes STI/HLT and observes the next real timer.

Elapsed samples use the mapped 64-bit HPET and its validated period. Delivered
LAPIC ticks remain separate diagnostic uptime, initialized once per boot. No
public elapsed clock/deadline ABI is added. Dispatch residence includes entry,
kernel work, FP switching and user execution; it is not task CPU time. Runnable
wait includes observer work. `max_body_ns` starts in the Rust entry and ends
before assembly return. `max_masked_upper_ns` conservatively includes whole
dispatch and supervisor residence, including user time and all assembly; it
bounds masked subintervals instead of pretending to measure exact IF transitions.
Bootstrap/staging when nothing is admitted runnable is excluded. Samples include
host scheduling effects under TCG and provide no hard real-time claim.

Trace-on logs each completed dispatch outside IRQ; trace-off suppresses those
rows, retaining lifecycle/settlement/timing summaries. Both still sample the
clock. One hundred additional HPET reads record observer cost; no measured cost
is subtracted. Keep paired observations and misses. P3-13 owns sustained resource
and latency review; desktop CPU/frame/input/256 MiB targets remain unaccepted.

## Acceptance review and retained evidence

`Test-P3-05.ps1` reruns all ten prerequisites, complete 33-row/42-launch M1,
1000-cycle memory settlement, static loader and four P3-04 guest groups before
the paired scheduler boots and a declared CPU-negative boot with SSE2 disabled. `Review-P3-05.py` independently checks raw serial,
queue rotation, progress, outcomes, fresh identities, artifact/ISO hashes,
codegen, FP instruction locations and exact inventories. It also rejects evidence
mutations. Review is local self-review, not independent human approval. Keep full
logs, ISOs, disassembly and dirty source copies local; promote only a compact
reviewed bundle under the evidence-retention policy.

## Emulator limits and supported exception controls

The initial real `DIVSS 0/0` probe with invalid-operation unmasked returned rather
than raising #XM on pinned TCG. Its failure trace is retained. This agrees with
QEMU's [documented trapping limitation](https://lists.gnu.org/archive/html/qemu-devel/2020-07/msg02877.html)
and [v11.1.0 MXCSR helper](https://github.com/qemu/qemu/blob/v11.1.0/target/i386/tcg/fpu_helper.c).
Do not relabel a software check as a hardware #XM. This experimental profile
therefore requires all six SSE exception masks. Clearing any mask terminates at
the next user timer or syscall entry, before that syscall can perform effects.
Rounding modes and masked sticky exception flags are preserved. Real hardware
#XM containment remains unverified; a runtime needing unmasked SSE traps must
pass a new native exception gate. Ordinary masked math remains a P3-12 port gate.
Real x87 #MF is tested after explicitly enabling CR0.NE; legacy IRQ13 is unused.

The same QEMU helper stores zero x87 opcode/IP/DP metadata. Nonzero native debug
metadata is consequently not execution-validated here. The eager architecture
instructions include those fields, but this review only credits their default
zero values under TCG. Arithmetic data, tags, status, rounding, MXCSR and all
sixteen XMM registers have active distinct sentinels. No debugger/TLS/extended
state feature is accepted.

## Fixture observer and syscall hooks

Scheduler and capability fixtures register syscall policy through
`user_fixture.rs`; dispatch has no flavor cfg branches inside operation arms.
Capability authorization runs before scheduler yield/exit handling. Install
sets CR0.TS while enabling FX support; scheduler resume restores the owner's
private eager FP state and clears TS. The separate boundary regression retains
TS and its process-local #NM test.

The fixture installs a progress observer for its mapped control-page word at
0x70000008. `capture` and `finish` call that observer at the original sampling
points while the owner's root is active, before returning to the supervisor.
The switch mechanism contains no fixture address. Samples still feed
`task_switch.progress`, `task_end.progress` and the non-yielding hog's positive
progress assertion; inventory settlement does not depend on them. Capability
fixtures install the same observer, preserving their prior sample behavior.
