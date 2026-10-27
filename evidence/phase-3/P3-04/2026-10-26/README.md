# P3-04 — Integer user entry and syscall/fault boundary pass locally

**P3-04 is Done under local Codex self-review.** This accepts the bounded single-CPU
execution slice, not external approval or M3. Read the [criterion/unsafe review](review.json)
and [runbook](../../../../docs/05-delivery/phase-3/syscall-boundary-runbook.md).

Four fresh offline QEMU/OVMF optical boots execute 36 process runs: twenty healthy
runs and sixteen offenders. Every offender has an immediate healthy replacement
in the same boot. Actual CPL3 faults, rejected syscall return states, panic and
quota/allocator OOM terminate locally. One kernel entry per guest, correlated
build/session, monotonic delivered ticks and exact empty space/allocation/mapping/
table/stack/pin/frame inventories are independently checked.

## Exact inputs and artifacts

Final boundary run: `${WORKSPACE}\build/m1\user-boundary-1793059418368826400-28196`. See all [commands](acceptance.json).
Source base: `a15b813ae5b3f874bf84f62fc728a9962b67994c` plus exact [dirty input hashes](source-provenance.json).
The snapshot includes existing Windows-policy script edits, separate from the
P3-04 staged changes. Tests used the existing signer; no policy deployment ran.
Full binaries, ISOs, patches/source copies and disassembly remain local under
ignored build/work. Selected raw serial and launch context are in `launches/`.

Boundary build ID: `01c85c5d11dc57da032de15b2354db6de3bab301a54fa511c0de3f4c6dc289b5`.
Kernel SHA-256: `5b016f6db07d989a96707cc188327d61c71065e89a0d74e07d0df018c8c88243`.

| Guest group | Launched ISO SHA-256 | Host lifecycle ms |
| --- | --- | --- |
| a | `bffd0403c231900bfeec7acdf08d6bcf221f3e1989fa1cbcf5c084d9b8b7e9c7` | 9745 |
| b | `390273b8bf85b96f79ce3cffea50f60569d2465fe73d05660e0f57f15f6f04e6` | 7143 |
| c | `47d45b77b21a3af73ba8ec548b653783872cfcccf0c84e7b39d11ccf930f7047` | 11062 |
| d | `64f24a098fd0050de5dd1639b11a43d7a37b16aa756ef41d7758755b784012be` | 10050 |

All launches use the pinned Q35/TCG CPU/profile, one vCPU, 2 GiB capacity, no NIC
or host-shared filesystem, fresh firmware variables and unchanged 30000-ms deadline.
Lifecycle milliseconds are host observations, not guest CPU/performance acceptance.
All processes/VMs are reaped; hidden resets cannot count as post-fault progress.

## Criterion review

| Criterion | Conclusion |
| --- | --- |
| Real entry/syscall/runtime | Pass: independently linked Rust A/B fixtures execute discovery, allocation/free/realloc, bounded log, yield and exit. Ordinary Rust returns reach generated exit at `0x7ffd100f`. GPR/DF/IF sentinels, user IRQs and nested kernel IRQs pass. A service image is separately staged as foreign-memory fixture. |
| Pointer/identity denial | Pass: null/end/overflow/kernel/foreign/RO/mixed-page spans, successful crossing copies, zero/capacity/256-byte diagnostic fragments, fabricated/interior/double frees, unknown number/features and bad revision have exact typed outcomes. Trusted current state supplies caller binding. |
| Process-local failure | Pass: page/guard/RO/UD2/CLI/port/IDT/forbidden-FP faults, panic, quota OOM and trusted physical-allocator exhaustion each precede healthy replacement. Injection admits one extra MiB after exact staging; it does not exhaust 2 GiB RAM. |
| Kernel/return boundary | Pass: noncanonical/unmapped user SP, actual AC and explicit test-only noncanonical IP/IOPL mutations reject before IRETQ. Precise kernel copy read/write recovery passes. Kernel fatal, guard, double-fault/IST and returning-interrupt paths pass complete M1. |
| Reclaiming memory/unsafe review | Pass: realloc/growth/reuse bytes, failure preservation and exact inventories, effective ancestor permissions/pins, surviving-stack reap and page-batched exclusive staging receive explicit review. Three user ELFs and kernel instructions are audited; the named PXOR trap is the only user SIMD instruction. |

All ten fresh prerequisites pass. M1 passes 33 rows/42 launches, including five
normal boots and the second clean optical boot. Clean kernel bytes match; ISO
bytes differ only in the permitted 64-byte session field. This is not release
reproducibility acceptance. The 1000-cycle owned-memory and three-image staged
loader regressions pass independent table/page/content/identity reviews. All 30
kernel host tests pass. The reviewer rejects 71 evidence mutations.

[Development findings](development-findings.json) preserve failed experiments and
selected failure records with full local paths/hashes. Final acceptance has no
automatic retry/deadline extension. Process-local serial host tests retain every
case and the original host child/VM deadlines.

## Limits and next story

P3-05 is next. Preemption, fairness and FP/SIMD context are absent; initialize and
restore FP state before clearing the current TS guard. P3-06 owns rights-bearing
memory and real preemptive lifetime races. Public child authorization, general
supervision/IPC, ABI freeze, accounts, guest UI/reload and desktop performance/
256 MiB cold idle remain open. Synthetic diagnostic IDs/private allocation are
experimental. The P3-03 staging test retains UD2; executing launches explicitly
install generated process_exit. ADR-0020 remains Proposed.
