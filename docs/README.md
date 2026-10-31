# Original desktop OS: research and engineering foundation

Research date: **2026-10-17**. Project name: **ContinuumOS (working name)**.
Implementation update: **2026-10-20 — original Rust kernel ISO with checked owned
boot metadata, host session identity, fatal diagnostics and timer/uptime demonstrated
in QEMU**. See the [exception runbook](05-delivery/phase-1/exception-diagnostics-runbook.md) and
[P1-04 acceptance review](../evidence/phase-1/P1-04/2026-10-19/README.md). M1's declared
experimental baseline now passes local acceptance; this is not a usable desktop OS.
Phase 0's compile-only evidence remains separate.
Host maintenance: the [M1 harness module split](../evidence/phase-1/m1-module-refactor/2026-10-18/README.md)
passed the current regression suite. Build, package, launch and observe now have
separate modules; commands and evidence formats are preserved. P1-04 now adds
[fatal exception diagnostics](05-delivery/phase-1/exception-diagnostics-runbook.md).
Its local acceptance review passes. P1-05 now implements
[memory ownership and guarded mappings](05-delivery/phase-1/memory-ownership-runbook.md);
its [local acceptance review passes](../evidence/phase-1/P1-05/2026-10-19/README.md).
P1-06 now adds [interrupts and monotonic uptime](05-delivery/phase-1/interrupts-timekeeping-runbook.md); its [local acceptance passes](../evidence/phase-1/P1-06/2026-10-20/README.md).
P1-07's [complete local acceptance](../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md)
passes all 33 rows and 42 launches. Its initial blocked review remains preserved.
The owner authorized [local development signing and an App Control development policy](05-delivery/windows-development-signing.md)
on this Windows installation to address recurring host execution blocks. The
replacement base and exact-hash tool supplement are enforcing. Fresh signed host
rebuilds and clean Cargo build-script/proc-macro/test/run preflights pass, followed
by the full P1-07 acceptance gate and local self-review.
Plan revision: **2 — reconciled direction approved by the user; technical gates pending**.
The [2026-10-21 installer/accounts amendment](00-overview/reconciled-plan.md#installer-and-accounts-amendment--2026-10-21)
makes graphical VM installation, local account setup, repair and preserving
reinstall mandatory for M8. It adds planning tasks B41–B44, not implementation or
new P-level stories; existing M0–M2 evidence remains unchanged.
The subsequent [desktop/platform expansion](00-overview/reconciled-plan.md#desktop-and-application-platform-expansion--2026-10-21)
adds M9–M21 and broad B45–B60: M14 usable VM desktop, M15 one named PC, and separate
compatibility/browser/runtime experiment gates. The [feature matrix](05-delivery/desktop-feature-matrix.md)
records release boundaries; none of this is implemented by documentation completeness.
The requested project folder is named **ContinuumOS**; this is a working project name.

This library translates the user's idea into an implementable, testable design:
an original OS with a beautiful, efficient desktop, modern systems code, and a
development loop that updates supported components inside a running VM. A bootable
ISO packages a known system generation; an ISO is not the live update transport.

## Start here

Repository policies: [rights](../LEGAL.md), [governance](../GOVERNANCE.md),
[security](../SECURITY.md), [third-party notices](../THIRD_PARTY_NOTICES.md),
[release gates](../RELEASE.md), and [administrative controls](05-delivery/repository-controls.md).
The [U.S.-only handling hold](../EXPORT_CONTROL.md) blocks remote sharing except
for the specific owner-authorized transfers recorded there; no remote environment
is approved.

1. [Vision and scope](00-overview/vision-and-scope.md): user intent, assumptions, and boundaries.
2. [Requirements](00-overview/requirements.md): identifiers, priorities, and observable outcomes.
3. [Architecture summary](00-overview/architecture-summary.md): proposed stack and key tradeoffs.
4. [Decisions](04-decisions/README.md): alternatives, consequences, and reconsideration gates.
5. [Milestones](05-delivery/milestones.md) and [agent handoff](05-delivery/agent-handoff.md): where coding begins.
6. [End-to-end scenarios](00-overview/end-to-end-scenarios.md): how the pieces behave together.
7. [Reconciled plan](00-overview/reconciled-plan.md): approved changes and first live-reload demonstration.

## Read by area

| Area | Documents |
| --- | --- |
| Vocabulary | [Glossary](00-overview/glossary.md) |
| Evidence | [Research method](01-research/research-method.md), [source catalogue](01-research/sources.md), [prior art](01-research/prior-art-and-novelty.md) |
| Languages and reuse | [Language evaluation](01-research/language-evaluation.md), [dependency strategy](01-research/dependency-strategy.md) |
| UI selection | [Slint/custom comparison](01-research/ui-toolkit-comparison.md) |
| System structure | [Layers](02-architecture/system-layers.md), [kernel](02-architecture/kernel-and-execution.md), [capabilities and IPC](02-architecture/capabilities-and-ipc.md) |
| Platform | [Boot](02-architecture/boot-and-platform.md), [drivers](02-architecture/drivers-and-hardware.md) |
| Actions and recovery | [Typed actions / optional AI](02-architecture/actions-and-optional-ai.md), [boot recovery](02-architecture/boot-recovery.md), [action contract](06-contracts/actions-and-context.md) |
| Persistent services | [Storage](02-architecture/storage-and-data.md), [networking](02-architecture/networking.md), [security](02-architecture/security-and-trust.md), [identity and authorization](02-architecture/identity-and-authorization.md) |
| Desktop | [Graphics](02-architecture/graphics-and-compositor.md), [UI and design](02-architecture/ui-framework-and-design.md), [apps](02-architecture/applications-and-runtime.md) |
| Whole-system quality | [Performance and observability](02-architecture/observability-and-performance.md), [accessibility and internationalization](02-architecture/accessibility-and-internationalization.md), [power and lifecycle](02-architecture/power-audio-and-lifecycle.md) |
| Live changes | [Guarantees](03-live-update/model-and-guarantees.md), [transactions](03-live-update/protocol-and-transactions.md), [state](03-live-update/state-and-compatibility.md), [recovery](03-live-update/failure-recovery-and-security.md) |
| Interfaces | [Kernel objects and IPC](06-contracts/ipc-and-objects.md), [update format](06-contracts/update-wire-format.md), [scene protocol](06-contracts/ui-and-scene-protocol.md), [package manifest](06-contracts/package-manifest.md) |
| Execution | [Backlog](05-delivery/backlog.md), [build and release](05-delivery/build-and-release.md), [installer preparation](05-delivery/installer-preparation.md), [validation](05-delivery/validation.md), [risks and open questions](05-delivery/risks-and-open-questions.md) |
| Phase 1 stories | [Epic and ordered implementation gates](05-delivery/phase-1/README.md), [host module layout](05-delivery/phase-1/minimal-iso-runbook.md#host-harness-source-layout), [metadata runbook](05-delivery/phase-1/boot-metadata-runbook.md) |
| Phase 2 stories | [Hosted UI epic and thirteen ordered stories](05-delivery/phase-2/README.md): P2-01 through P2-13 accepted locally; M2 hosted subset passes; custom direction remains provisional until guest confirmation |
| Phase 3 stories | [Isolated runtime epic and fourteen ordered stories](05-delivery/phase-3/README.md): P3-01/P3-02/P3-03/P3-04/P3-05/P3-06/P3-07 Done; P3-08 Done; P3-09 Done; P3-10 Done; P3-11–P3-14 Not started; general supervision, independent SDK and final acceptance required |
| Later desktop | [Feature/release matrix](05-delivery/desktop-feature-matrix.md), [service responsibilities](06-contracts/desktop-services.md), [Windows study](01-research/windows-compatibility-feasibility.md), [browser study](01-research/browser-feasibility.md); no P4–P21 decomposition yet |

## Status and interpretation

The owner's [evidence cleanup](../evidence/README.md) leaves historical logs and
source copies local-only. Original reviews describe the pre-cleanup bundles;
current metadata checks do not prove unavailable raw records on a fresh checkout.

- **User requirement**: derived from the conversation. Only the user can change the core intent.
- **Verified finding**: supported by a primary source in the catalogue; not independently reproduced here.
- **Proposed default**: our engineering choice, used to make the project concrete. Validate at its gate.
- **Experiment**: a question with an owner, procedure, and pass/fail criterion in the milestone/backlog documents.
- **Measured**: requires build IDs, configurations, and raw evidence. Phase 0 has
  compile/link results; P1-02 adds minimal guest entry and emulator-test exits;
  P1-03 adds checked boot descriptors, correlated records and rejection evidence.
  Guest performance, full boot readiness and live updates remain unmeasured.

ADRs are proposed, rather than falsely marked accepted. Their experiments let an
implementation agent proceed with reversible work without repeatedly requesting
architectural permission. Incompatibility with a core user requirement needs an
explicit decision, not a silent shortcut.

Every source has a stable `Sxx` identifier. Links in specifications point to the
catalogue entries. Dates refer to the user's America/New_York calendar date.
Upstream `latest` and `master` URLs are research references, not dependency pins.

## Recommended first vertical slice

For the hosted UI workstream, compare Slint with a minimal custom UI core and
prove resource reload while a document remains open. The independent ISO path
has reached checked metadata, fatal diagnostics and P1-05 owned mappings.
P1-07 and the [Phase 2 prerequisite recheck](05-delivery/phase-2/01-hosted-prerequisites-and-profiles.md)
now pass. [P2-02 shared boundaries and fixtures](05-delivery/phase-2/02-shared-boundaries-and-reference-fixtures.md)
also pass [local acceptance](../evidence/phase-2/P2-02/2026-10-21/README.md).
P2-03's [document/action model](05-delivery/phase-2/03-document-state-and-typed-actions.md)
passes [local acceptance](../evidence/phase-2/P2-03/2026-10-21/README.md).
P2-04's [fonts and bounded text](05-delivery/phase-2/04-fonts-and-text-conformance.md)
pass [local acceptance](../evidence/phase-2/P2-04/2026-10-21/README.md).
P2-05's [custom candidate](05-delivery/phase-2/05-minimal-custom-ui-candidate.md)
passes [local acceptance](../evidence/phase-2/P2-05/2026-10-21/README.md), including
rendered scenes, shared action/semantic workflows, bounded text/list/cache checks
and native Windows presentation. P2-06's
[Slint candidate evidence](../evidence/phase-2/P2-06/2026-10-21/README.md) now passes
with explicit shared text/input/semantic adapters and native feature failures.
[P2-07 target linking](../evidence/phase-2/P2-07/2026-10-22/README.md) now closes
both real candidate paths with bounded probe allocation and explicit runtime
obligations. [P2-08's hosted comparison](../evidence/phase-2/P2-08/2026-10-22/README.md)
now records quality, frame/retention costs and direct resource/helper continuity.
Its conditional custom recommendation preserves both candidates' frame misses
and Slint lifetime/fidelity gaps. P2-09 now records the
[provisional custom Rust decision](05-delivery/phase-2/ui-selection-review.md),
license choices and mandatory completion plan. M2 completion and actual guest
execution remain later gates.
P2-10 now passes [selected hosted-core acceptance](../evidence/phase-2/P2-10/2026-10-23/README.md), including
bounded two-pass layout, input reconciliation, semantic virtual navigation, visual
grapheme affinity and interruptible motion. Use its
[selected-core runbook](05-delivery/phase-2/selected-ui-runbook.md). P2-11 now passes [same-window hosted resource reload](../evidence/phase-2/P2-11/2026-10-23/README.md); use its [runbook](05-delivery/phase-2/resource-reload-runbook.md). P2-12 now passes [hosted damage/idle/stability acceptance](../evidence/phase-2/P2-12/2026-10-24/README.md), retaining substantial frame/resource-latency misses; use its [runbook](05-delivery/phase-2/performance-stability-runbook.md). P2-13 now passes [final aggregate acceptance](../evidence/phase-2/P2-13/2026-10-24/README.md); use its [runbook](05-delivery/phase-2/acceptance-runbook.md). M3 has [fourteen Phase 3 stories](05-delivery/phase-3/README.md). P3-01 [passes local prerequisite/contract acceptance](../evidence/phase-3/P3-01/2026-10-25/README.md); P3-02 [passes local memory-backend acceptance](../evidence/phase-3/P3-02/2026-10-25/README.md); P3-03 [passes local staged-loader acceptance](../evidence/phase-3/P3-03/2026-10-25/README.md); use its [runbook](05-delivery/phase-3/static-loader-runbook.md). P3-04 [passes local user-entry/syscall/fault-containment acceptance](../evidence/phase-3/P3-04/2026-10-26/README.md); use its [runbook](05-delivery/phase-3/syscall-boundary-runbook.md). P3-08 is Done under local self-review; P3-09 is Done; P3-11–P3-14 remain Not started. P3-05 now passes the declared scheduler/context subset below; P3-06 adds the bounded handle/memory subset below; P3-08 now passes its wait/deadline follow-up below; P3-09 follows in the current-state review below.
Establish user-mode isolation and a small
authenticated development channel before native service replacement. This parallel
sequence describes independent workstreams, not permission to skip their dependencies.

The important demo is **edit source, update a running guest, preserve a window and
document, prove boot identity and uptime did not reset**. A screenshot alone does
not prove live updating.

Draft machine-readable schemas and synthetic fixtures are indexed in
[contracts](06-contracts/README.md). [Research audit](05-delivery/research-audit.md)
records documentation checks actually performed; it is separate from future OS tests.

P3-05 [passes local scheduler/context acceptance](../evidence/phase-3/P3-05/2026-10-27/README.md): equal-priority preemption, eager private x87/SSE state, 60 task runs and real missing-SSE2 CPU rejection with exact settlements and current M1 regressions. One owner thread per process and masked SSE exceptions are the admitted subset; native #XM and nonzero x87 debug metadata remain unverified. At the P3-05 gate public rights remained open. Blocking IPC, general supervision, SDK freeze and guest UI remain open; P3-06 adds the bounded handle/memory subset below; P3-08 now passes its wait/deadline follow-up below; P3-09 follows in the current-state review below.

P3-06 [passes local handle/memory-object acceptance](../evidence/phase-3/P3-06/2026-10-29/README.md): process-local typed rights, attenuating duplicates, retained immutable sharing after creator death, five hardware isolation faults, live-pin mutation denials and exact reference/frame settlements across 18 tasks. All 39 kernel host tests, ten prerequisites, complete M1 (33 rows/42 launches), memory/loader/user-boundary and scheduler regressions pass. This adds the R10/R14/R18 and T02/T03/T06/T11 object/mapping subset; public transfer, blocking IPC, general supervision, SDK freeze and guest UI remain open. Use the [runbook](05-delivery/phase-3/handles-memory-runbook.md); P3-08 now passes its wait/deadline follow-up below; P3-09 follows in the current-state review below.

P3-06 [passes local handle/memory-object acceptance](../evidence/phase-3/P3-06/2026-10-29/README.md): process-local typed rights, attenuating duplicates, retained immutable sharing after creator death, five hardware isolation faults, live-pin mutation denials and exact reference/frame settlements across 18 tasks. All 39 kernel host tests, ten prerequisites, complete M1 (33 rows/42 launches), memory/loader/user-boundary and scheduler regressions pass. This adds the R10/R14/R18 and T02/T03/T06/T11 object/mapping subset; public transfer, blocking IPC, general supervision, SDK freeze and guest UI remain open. Use the [runbook](05-delivery/phase-3/handles-memory-runbook.md); P3-08 now passes its wait/deadline follow-up below; P3-09 follows in the current-state review below.

P3-07 [passes local bounded-channel acceptance](../evidence/phase-3/P3-07/2026-10-29/README.md): atomic nonblocking bytes/authority transfer, trusted current-caller provenance after delegation, bounded queue/table/copy denials, queued peer death and exact settlement across six tasks in two waves. All 47 kernel host tests, ten prerequisites, complete M1 (33 rows/42 launches), memory/loader/boundary/scheduler and original capability regressions pass local self-review. 136 evidence mutations are rejected. This adds the R10/R14/R18 and T02/T03/T06/T11 messaging subset, with preparatory R19 provenance. Blocking IPC, current policy/session checks, SDK freeze and guest UI remain open; P3-08 now passes its wait/deadline follow-up below; P3-09 follows in the current-state review below. Use the [channel runbook](05-delivery/phase-3/channels-runbook.md).

## P3-08 wait/deadline subset

P3-08 passes local self-review for bounded waits, elapsed HPET deadlines, events/timers, cancellation and blocking channel helpers. The 18-task/six-wave guest proves registration-boundary notifications, immutable completion, pressured transfer ownership, bounded request cycles, parked-owner termination and exact settlements. All 64 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel regressions pass. General supervision, policy/session liveness, independent SDK freeze and guest UI remain open; P3-09 follows in the current-state review below. See the [acceptance evidence](../evidence/phase-3/P3-08/2026-10-30/README.md). This adds R10/R14/R18 and T03/T06, the timer subset of T04 and runtime T09 groundwork; it accepts no desktop latency or whole-guest idle-memory target.

## P3-09 bounded supervision subset

P3-09 passes local self-review for its bounded user-mode supervision profile: 29 dynamic admissions/reaps across four bootstrap epochs, exact lifecycle outcomes, prepaid ancestor quotas, authority/priority/inheritance denial, parked/orphan/supervisor teardown, two retry permits and quarantine. All 73 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel/wait regressions pass. Four epochs settle all object/frame/table/stack/pin/wait/tree inventories exactly. One thread/process, equal priority and eight lifetime units per epoch are the admitted subset; physical reclamation does not refund credits. Persistent supervision, current policy/session enforcement, independent SDK freeze and guest UI/reload remain open; P3-10 follows in the current-state review below. See the [acceptance evidence](../evidence/phase-3/P3-09/2026-10-30/README.md).


## P3-09 bounded supervision subset

P3-09 passes local self-review for bounded user-mode supervision: 29 dynamic admissions/reaps across four bootstrap epochs, exact lifecycle outcomes, prepaid ancestor quotas, authority/priority/inheritance denial, parked/orphan/supervisor teardown, two retry permits and quarantine. All 73 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel/wait regressions pass. Four epochs settle all object/frame/table/stack/pin/wait/tree inventories exactly. One thread/process, equal priority and eight lifetime units per epoch are the admitted subset; physical reclamation does not refund credits. Persistent supervision, current policy/session enforcement, independent SDK freeze and guest UI/reload remain open; P3-10 follows in the current-state review below. See the [acceptance evidence](../evidence/phase-3/P3-09/2026-10-30/README.md).


## P3-10 ephemeral scoped policy subset

P3-10 passes local self-review for its ephemeral scoped policy subset: 120 real IPC decisions, 67 expected denials, two broker faults/restarts, identical 160-byte replay across fresh process/session bindings and two exact resource settlements. All 82 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches) and separate memory/loader/boundary/scheduler/capability/channel/wait/supervisor regressions pass. The CPL3 broker/provider enforces exact scoped role assignments, child-grant attenuation, caller/service forwarding ceilings, finite sessions, use-time expiry/epoch/revision/context checks and synthetic privileged-scope separation. Queued grant and session expiry cross their actual elapsed fence; 136 focused evidence mutations reject. Trusted restoration is explicit and ordinary direct mappings survive broker death until owner teardown. No real accounts, credentials, durable RBAC, universal direct revocation, independently frozen SDK, guest UI/reload or desktop efficiency target is accepted; P3-11 is next. See the [acceptance evidence](../evidence/phase-3/P3-10/2026-10-30/README.md).
