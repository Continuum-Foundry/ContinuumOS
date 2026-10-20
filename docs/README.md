# Original desktop OS: research and engineering foundation

Research date: **2026-10-17**. Project name: **ContinuumOS (working name)**.
Implementation update: **2026-10-20 — original Rust kernel ISO with checked owned
boot metadata, host session identity, fatal diagnostics and timer/uptime demonstrated
in QEMU**. See the [exception runbook](05-delivery/phase-1/exception-diagnostics-runbook.md) and
[P1-04 acceptance review](../evidence/phase-1/P1-04/2026-10-19/README.md). This is not completed M1
or a usable desktop OS. Phase 0's compile-only evidence remains separate.
Host maintenance: the [M1 harness module split](../evidence/phase-1/m1-module-refactor/2026-10-18/README.md)
passed the current regression suite. Build, package, launch and observe now have
separate modules; commands and evidence formats are preserved. P1-04 now adds
[fatal exception diagnostics](05-delivery/phase-1/exception-diagnostics-runbook.md).
Its local acceptance review passes. P1-05 now implements
[memory ownership and guarded mappings](05-delivery/phase-1/memory-ownership-runbook.md);
its [local acceptance review passes](../evidence/phase-1/P1-05/2026-10-19/README.md).
P1-06 now adds [interrupts and monotonic uptime](05-delivery/phase-1/interrupts-timekeeping-runbook.md); its [local acceptance passes](../evidence/phase-1/P1-06/2026-10-20/README.md).
P1-07 implements the [full acceptance command](05-delivery/phase-1/acceptance-runbook.md),
but its [initial review was blocked](../evidence/phase-1/P1-07/2026-10-20/README.md) by Windows
Smart App Control refusing the rebuilt host-test executable. M1 remains incomplete.
The owner authorized [local development signing and an App Control development policy](05-delivery/windows-development-signing.md)
on this Windows installation to address recurring host execution blocks. The
replacement base is enforcing, but the pinned-tool supplement awaits Windows
elevation; compiler execution and full acceptance remain blocked.
Plan revision: **2 — reconciled direction approved by the user; technical gates pending**.
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
| Persistent services | [Storage](02-architecture/storage-and-data.md), [networking](02-architecture/networking.md), [security](02-architecture/security-and-trust.md) |
| Desktop | [Graphics](02-architecture/graphics-and-compositor.md), [UI and design](02-architecture/ui-framework-and-design.md), [apps](02-architecture/applications-and-runtime.md) |
| Whole-system quality | [Performance and observability](02-architecture/observability-and-performance.md), [accessibility and internationalization](02-architecture/accessibility-and-internationalization.md), [power and lifecycle](02-architecture/power-audio-and-lifecycle.md) |
| Live changes | [Guarantees](03-live-update/model-and-guarantees.md), [transactions](03-live-update/protocol-and-transactions.md), [state](03-live-update/state-and-compatibility.md), [recovery](03-live-update/failure-recovery-and-security.md) |
| Interfaces | [Kernel objects and IPC](06-contracts/ipc-and-objects.md), [update format](06-contracts/update-wire-format.md), [scene protocol](06-contracts/ui-and-scene-protocol.md), [package manifest](06-contracts/package-manifest.md) |
| Execution | [Backlog](05-delivery/backlog.md), [build and release](05-delivery/build-and-release.md), [validation](05-delivery/validation.md), [risks and open questions](05-delivery/risks-and-open-questions.md) |
| Phase 1 stories | [Epic and ordered implementation gates](05-delivery/phase-1/README.md), [host module layout](05-delivery/phase-1/minimal-iso-runbook.md#host-harness-source-layout), [metadata runbook](05-delivery/phase-1/boot-metadata-runbook.md) |

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
Complete P1-07 before bringing UI primitives
to the guest framebuffer.
Establish user-mode isolation and a small
authenticated development channel before native service replacement. This parallel
sequence describes independent workstreams, not permission to skip their dependencies.

The important demo is **edit source, update a running guest, preserve a window and
document, prove boot identity and uptime did not reset**. A screenshot alone does
not prove live updating.

Draft machine-readable schemas and synthetic fixtures are indexed in
[contracts](06-contracts/README.md). [Research audit](05-delivery/research-audit.md)
records documentation checks actually performed; it is separate from future OS tests.
