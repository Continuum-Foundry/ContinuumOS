# ADR-0029 — Keep stress accounting and admission profiles explicit

Status: Proposed. Date: 2026-11-01. Gate: P3-13 local stress review and the
separate P3-14/M3 review. The bounded P3-13 experiment passes local self-review;
this does not approve the M3 exit or the architecture for broader workloads.

## Context

The independently built SDK closes a bounded interface subset. Its short runs
cannot establish sustained reclamation, fairness or resource peaks. The trusted
bootstrap and prepaid supervision profiles have different admitted workloads:
using a supervision unit for thousands of messages would bypass its four-message
lifetime ceiling, even if a benchmark later freed every object.

## Decision

Keep those profiles distinct in one reproducible suite. Independently built SDK
workers execute the fixed IPC and lifecycle workload under trusted bootstrap.
A public-wrapper port of the complete supervision fixture separately exercises
the original quotas, teardown and quarantine through experimental runtime-v0
operations. It neither increases those ceilings nor extends the SDK freeze.

Keep the original one-vCPU Q35/TCG/HPET profile, guest deadline, stress counts,
fairness ceiling and zero unexplained retained-object/frame tolerances. Publish
the experiment inputs before guest measurements. A failure requires diagnosis
and remediation; it does not authorize wider acceptance bounds.

Count physical backing once globally. Report process ownership, direct reference
attribution and queued holds as separate views. Charge preallocated kernel tables,
stacks, sample buffers, allocator metadata and unused capacity through their
containing image; do not add an active page-table subset to that same image twice.
Keep genuinely free eligible RAM distinct from reserved/unmanaged capacity and
host process memory. Capture staged, queued, death and settled inventories.

The stress adapter retains three verified boot-image plans in image-owned static
storage, with a separate 4096-byte ceiling enforced at compile time and recorded
in the experiment profile. `VerifiedImage` privately holds the checked plan and the exact immutable
byte borrow; the boot adapter supplies only RO/NX modules with no direct-map
aliases. Hash/ELF verification occurs once per module and is timed explicitly.
Each launch still checks commitment, slot/generation, return code, transactional
staging and publication. Ordinary launch callers retain complete verification.
This cache cannot accept mutable buffers or replaceable module contents; future
reload requires a separately verified image lifetime. Report its fixed retained
bytes and include them in the kernel-image charge; it is not a settled frame leak.

Use guest HPET intervals for guest scheduling, waits and IPC. Keep host elapsed
lifecycle observations separate. Both paired instrumentation profiles retain
mandatory outcome and timing checks; the tracing profile adds read-only inventory
observation. Fixed-stride samples retain their exact selection rule, total
observation count and maximum; buffer overflow fails instead of silently dropping
data. Nearest-rank quantiles describe the retained samples, not an unsampled tail.

Termination requested during a live copy pin remains deferred until the operation
reaches a quiescent boundary. Install experimental syscall/return/copy behavior
through the registered fixture hooks. Public operations never select privileged
return state. No kernel borrow or pinned user reference survives a context switch.

## Consequences and limits

The stress coordinator records zero FS/GS/KERNEL_GS bases after installation and
disabled CR4.FSGSBASE before the first user entry. All admitted GDT bases are zero;
resume restores null selectors and the profile exposes no TLS/MSR writer. These
single-CPU invariants allow it to omit repeated writes of the same zero bases.
Other historical coordinators retain their resets. Supporting nonzero TLS bases
requires a different context contract and per-owner save/restore; this profile
does not silently admit it.

Snapshot MXCSR in the guarded syscall-entry prefix before setting TS, instead
of copying the full state or clearing/resetting TS again just to inspect its mask.
Only eager resume sets the private ownership marker; integer entry/leave clear
it. The snapshot accepts no pointer and writes one supervisor-only single-CPU
word. Integer boundary builds omit STMXCSR. Rust still checks the same mask before
an operation; full state saves remain eager at switches and termination.
Instruction audits allow the single STMXCSR only in that guarded entry prefix;
the existing unsupported-mask, preemption and cross-process FP regressions remain
required. This is a P3-05 mechanism optimization, not a wider user CPU contract.

This adds bounded test observation and explicit accounting, not a product heap,
new public protocol or security certification. It accepts no desktop memory,
frame/input latency, reload, durability or physical hardware target. Retain failed
attempts and exact source/artifact identities. Full outputs remain local; compact
promotion follows the existing evidence budget and handling hold.


## Local experiment result

The bounded [P3-13 experiment](../../evidence/phase-3/P3-13/2026-11-01/README.md) passes local Codex self-review. Exact source/ISA/ISO checks, all ten continuous guests, the separate public supervision guest and final-source regressions pass without widening bounds. This ADR remains Proposed pending the separate milestone/project review. The native host observer uses an optimized release build of the same validating source; its debug-build timeout and earlier failed experiments remain retained.

## P3-14 observer experiment

An additional P3-14 attempt reached only wave 78 of its first stress guest before
the unchanged 30-second deadline. The same ISO completed 100 waves in a separate
final-review-only diagnostic; that diagnostic is not acceptance. A captured-log
experiment measured about 16.9 ms per full owned-record-tree check versus 3.9 ms
for the same validation with export-tree capture disabled. The live observer now
uses that shared validation path, while final assertions/export retain full capture.
No validation branch, record, poll sleep, quota, guest sample count or deadline is
removed. Host parity tests pass; fresh guest and aggregate acceptance remain the
decision gates. These observations do not establish a cause for the earlier
pre-entry/partial-boot timeouts or accept a desktop performance target.

## P3-14 owner-accepted closure

P3-14 is Done by explicit owner acceptance with a legacy host-lifecycle exception. All current guest/runtime regressions, complete M1 review (33 rows/42 launches), ten stress guests and five aggregate failure probes pass local Codex self-review. The legacy P3-01 Python child misses its original one-second startup bound; that fixture is waived for this story closure only. The strict full gate and M3 exit remain open. No failed check is labeled passed, and no deadline is extended.

See the [evidence, failed fixture and explicit owner exception](../../evidence/phase-3/P3-14/2026-11-04/README.md). M4 guest UI/reload, M5 migration and safe retirement, M6 durability, M7 accounts/installation and all desktop efficiency targets remain open. Only the existing exact SDK subset is frozen; broader ADRs remain Proposed.
