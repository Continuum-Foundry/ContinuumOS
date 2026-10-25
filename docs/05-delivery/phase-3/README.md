# P3 epic — Isolated processes and stable interfaces

Issue type: Epic. Status: In progress. Owner: Unassigned.
Created: 2026-10-24. Milestone: M3. Suggested labels: `phase-3`, `milestone-m3`.
These are repository-local, Jira-ready stories; no Jira issues have been created.
This directory follows the [Phase 2 story format](../phase-2/README.md).

## Outcome and starting point

Run independently built Rust clients and services in hardware-isolated guest
processes, with preemption, bounded allocation, rights-bearing handles, copied IPC,
waits and supervision. Enforce trusted caller/session provenance, scoped grants,
ownership and brokered invalidation. Demonstrate proxy-route epochs and a fenced
writer without claiming live application state transfer or a desktop.

Planning baseline: `a5c3ea2`, inspected 2026-10-24 with a clean working tree.
M1's [complete local acceptance](../../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md)
passes 33 rows/42 launches. M2's [final hosted acceptance](../../../evidence/phase-2/P2-13/2026-10-24/README.md)
and [bounded performance follow-up](../../../evidence/phase-2/performance-remediation/2026-10-24/README.md)
pass their declared local gates, retaining performance misses. Neither executes
the UI in the guest. The selected freestanding ELF's non-reclaiming probe arena
is not a guest heap. Current kernel code has owned frames/page tables, exceptions
and delivered-tick uptime, but no general heap, scheduler or user isolation.
Reinspect actual source, tools and evidence availability before implementation.

## Ordered stories

| ID | Story | Depends on | Backlog coverage | Status |
| --- | --- | --- | --- | --- |
| P3-01 | [Recheck prerequisites and define runtime contracts/profiles](01-runtime-prerequisites-and-contract-plan.md) | P1-07 and P2-13 Done; Phase 0 recheck | B01/B02 follow-up, B06/B11–B13 design, B32 setup | Done |
| P3-02 | [Build bounded allocation and owned user address spaces](02-bounded-allocation-and-user-address-spaces.md) | P3-01 | B05 follow-up, B11/B13 memory foundation | Done |
| P3-03 | [Load static user images and bind trusted launch context](03-static-elf-loader-and-launch-context.md) | P3-02 | B11 loader/launch | Not started |
| P3-04 | [Enter user mode and enforce the syscall/fault boundary](04-user-entry-syscalls-and-fault-containment.md) | P3-03 | B11 entry/syscalls, B13 failure groundwork | Not started |
| P3-05 | [Preempt tasks and preserve integer/FP/SIMD context](05-preemptive-scheduling-and-cpu-context.md) | P3-04 | Remaining M3 B06 scheduling/context | Not started |
| P3-06 | [Enforce handle rights and memory-object lifetimes](06-handles-rights-and-memory-objects.md) | P3-05 | B12 object/handle/mapping subset | Not started |
| P3-07 | [Implement bounded channels and atomic authority transfer](07-channels-and-atomic-handle-transfer.md) | P3-06 | B12 messages/transfer/provenance | Not started |
| P3-08 | [Complete waits, timers, cancellation and backpressure](08-waits-timers-and-cancellation.md) | P3-07 | B12 waiting/deadlock subset, B06 deadlines | Not started |
| P3-09 | [Supervise process trees and enforce resource ceilings](09-supervision-quotas-and-process-lifecycle.md) | P3-08 | B13 supervision/quotas | Not started |
| P3-10 | [Enforce scoped identity, ownership and policy grants](10-identity-ownership-and-scoped-grants.md) | P3-09 | B11–B13 authority completion, B15/B37 seeds | Not started |
| P3-11 | [Fence service writers behind proxy routes and epochs](11-proxy-routes-epochs-and-writer-fencing.md) | P3-10 | B13 routes/invalidation/restart | Not started |
| P3-12 | [Prove independent SDK interoperability and runtime closure](12-independent-sdk-and-runtime-conformance.md) | P3-11 | B02/B11/B12 ABI closure, B15/B37 handoff | Not started |
| P3-13 | [Stress isolation, teardown and runtime resource bounds](13-adversarial-lifecycle-and-resource-accounting.md) | P3-12 | B06/B11–B13 robustness, M3 B32 subset | Not started |
| P3-14 | [Run and review Phase 3 acceptance](14-phase-3-acceptance.md) | P3-01 through P3-13 | M3 gate, M3 B32 subset | Not started |

Use this default execution order and accept one story before advancing. Each
story owns a complete boundary and its negative tests, rather than a collection
of one-function tickets. Fourteen stories separate memory/loader/CPU bring-up,
object and IPC lifetimes, policy and routing, then independent conformance,
adversarial stress and aggregate review. P3-13 owns sustained fault/resource
experiments; P3-14 aggregates and reviews final-source evidence. This avoids
making the final review a second implementation milestone. Dependencies describe
gates, not a request to run agents concurrently.

P3-01 [passes local acceptance](../../../evidence/phase-3/P3-01/2026-10-25/README.md):
current prerequisites, full M1 baseline, draft runtime/SDK, locked profiles and
bounded host lifecycle faults. Use its [runbook](runtime-prerequisites-runbook.md).
P3-02 [passes local memory-backend acceptance](../../../evidence/phase-3/P3-02/2026-10-25/README.md):
reclaiming runs, private staged tables, guarded stacks, aliases/pins, 1000 exact
guest settlements and the complete current M1 gate. Use the
[memory runbook](owned-memory-runbook.md). Next is P3-03; no loader, user-mode
execution, public rights, scheduler or M3 exit is accepted yet.

## Fixed scope and defaults

- Original first-party Rust kernel and user services, narrowly reviewed x86-64
  assembly, one vCPU, pinned QEMU/OVMF optical-boot path. Use the existing bounded
  offline harness and workspace-local tools. No guest compiler or host OS syscall
  fallback. Keep kernel/ISO and hosted UI builds independently available.
- Static, trusted-package ELF images; eager zero-filled user pages, guarded stacks,
  W^X/NX, fallible resource admission and reclaiming process-owned allocation.
  No dynamic linker, general demand paging, swap, SMP or kernel hot reload.
- Preemptive bounded scheduling and eager FP/SIMD context isolation are working
  defaults subject to P3-01/P3-05 review. Declare supported instructions/features
  from actual emitted code; never execute UI math before that gate passes.
- Versioned explicit encodings and independently built consumers. Draft syscall,
  object and IPC contracts evolve with experiments; freeze only the exercised
  subset at P3-12. Unimplemented device/IRQ/DMA and later desktop families return
  explicit unsupported outcomes and receive no implementation credit.
- One local development user in the product. Two synthetic users, two apps of
  one user and a separate service principal exercise enforcement. Sessions/roles
  are ephemeral trusted fixtures; credentials, login and durable RBAC remain M7.
- Kernel provenance and service checks jointly enforce authority. IDs, ownership,
  role strings, development-session claims and checkpoint bytes cannot mint grants.
  Reserve disk-installation/account-administration scopes with synthetic denied
  operations; perform no installer, account-store or physical-disk work.
- Brokered expiry/revocation and route fences have mandatory checks and explicit
  in-flight outcomes. Closing a handle or route does not revoke every duplicate
  or shared mapping. Track surviving direct authority and its cleanup honestly.
- Route switching uses synthetic in-memory state and a broker-owned writer lease.
  M3 proves routing/fencing and fresh authorization, not M5 state-preserving
  behavior replacement, safe old-code retirement or M6 durable transactions.
- Keep the 256 MiB cold-idle desktop target. M3 accounts kernel/runtime/test-service
  memory and measures scheduler/IPC costs; it cannot accept desktop/frame/input
  targets without M4's actual UI, display and input workloads.

## Evidence and story completion

Use `Not started`, `In progress`, `Blocked`, or `Done` in each story and this index;
update both together. All checklists begin unchecked. Done requires every criterion,
required negative cases, reproducible evidence and an explicit recorded review.
Failed experiments remain findings with a next bounded experiment; they do not
waive the milestone or become a security guarantee. ADRs remain Proposed until
their documented technical gates and project review pass.

Full generated logs, binaries, ISOs, source copies and disassembly stay under
ignored `build/m3/<story-id>/<run-id>/` or `work/`. Retain a compact reviewed bundle
under `evidence/phase-3/<story-id>/<run-id>/` only when implemented, following the
[retention policy](../evidence-retention.md): 4 MiB/200 files/256 KiB per file defaults.
Follow the [historical evidence availability notice](../../../evidence/README.md).
Planning creates no acceptance evidence, implementation commands or frozen ABI IDs.
Each implementation story supplies its actual bounded command/runbook before Done.

Each run records:

- Source commit/dirty-input hashes, compiler/profile/features, dependency/tool locks,
  image/module/ISO hashes, ABI revision, configuration and fixture identities.
- Exact guest artifacts and host launch arguments, CPU model/features, one-vCPU
  profile, VM capacity, timer source/resolution, seeds, bounds and instrumentation.
- Boot/build/session correlation plus distinct principal, authority-session, process,
  thread, object/handle generation, policy epoch, route epoch and request IDs as
  appropriate. Host-provided boot correlation is not an authorization credential.
- Before/after resource inventories, exact expected/observed errors, fault context,
  trusted sender attribution, grant/scope decisions and mandatory fence outcomes.
  Use synthetic data and bounded diagnostics without pointers/secrets/document text.
- Raw selected guest traces and independent assertions as well as summaries. Record
  guest-fatal, process-fatal, expected rejection, timeout and harness failure separately;
  a missing completion record or unexpected VM reset fails its scenario.
- Monotonic guest measurements, host VM lifecycle and cleanup. Do not subtract clocks
  across host/guest or equate delivered-tick uptime with accurate elapsed deadlines.

Preserve M1's memory/IST/interrupt/diagnostic invariants and M2's portable/host
separation. Rerun Phase 0 before extensions; run affected M1 regressions for each
kernel/build/launch change and the complete M1 gate at final acceptance. Changes
to shared UI/build inputs require affected M2 tests, not a claim of new guest UI.
Use existing signing/enrollment within authorization; do not change host policy
implicitly. No Jira creation, remote upload, publication or physical-PC work is
authorized by this decomposition.

## Traceability and later handoff

Epic requirements: M3 portions of R03/R10/R14/R18, preparatory R05/R06/R09/R17/R19.
Test families: T01/T02/T03/T06/T11/T13, runtime subset of T09 and synthetic route
fault subset of T08; timer/context subset of T04 and provider-enforcement subset
of T14. Full product security, live
updating, durability, accessibility and efficiency remain at their later gates.

| Existing work | Owning stories / boundary |
| --- | --- |
| B06 M3 remainder | P3-05/P3-08 scheduling, CPU state and deadline semantics; M19 SMP stays separate |
| B11 | P3-02–P3-04 memory/loader/syscalls, P3-09 child launch, P3-10 trusted authority, P3-12 independent ABI |
| B12 | P3-06–P3-08 objects/IPC/waits, P3-10 provider enforcement, P3-12 independent codecs |
| B13 | P3-09 supervision/quotas, P3-10 policy, P3-11 routes/fences, P3-13 sustained teardown |
| B15/M4 | Trusted caller/session binding, development-authority ceiling, typed errors and reserved input/security interfaces; no authenticated transport yet |
| B16/M4 | Owned mappings, allocator, execution-thread/FP state, clocks/waits and explicit presentation/driver obligations; toolkit guest confirmation stays M4 |
| B18–B21/M5 and B37 | Scoped provider checks, revision/deadline types, fresh replacement grants and route-fence contract; checkpoint migration and UI/automation replacement parity stay M5 |
| B41/M7 and B45/B51/M9/M11 | Reserved authority/types only; no account, file, installer or administration implementation |
| B32 | P3-01 profiles, P3-13 runtime accounting, P3-14 milestone review; broader desktop metrics stay open |

Read [M3](../milestones.md#m3--isolated-processes-and-stable-interfaces),
[backlog](../backlog.md), [validation](../validation.md),
[kernel/execution](../../02-architecture/kernel-and-execution.md),
[identity/authorization](../../02-architecture/identity-and-authorization.md),
[capabilities/IPC](../../02-architecture/capabilities-and-ipc.md),
[objects contract](../../06-contracts/ipc-and-objects.md),
[actions contract](../../06-contracts/actions-and-context.md),
[desktop service seeds](../../06-contracts/desktop-services.md),
[runtime hook inventory](../../../experiments/ui-link/hooks.json),
[ADR-0010](../../04-decisions/0010-capability-security.md) and
[reconciled direction](../../00-overview/reconciled-plan.md).
