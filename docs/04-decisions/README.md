# Architecture decision records

Initial records dated **2026-10-17**; later records carry their own dates. Status **Proposed** means a documented working
default for implementation experiments, not user acceptance and not a verified result.
An agent may implement its reversible gate, then record evidence and change status.
Revision 2 records user approval of the [reconciled roadmap](../00-overview/reconciled-plan.md).
Technical capability claims and final library selections still require gate evidence.

[P1-03's acceptance review](../../evidence/phase-1/P1-03/2026-10-18/README.md)
adds checked boot metadata and identity evidence for the declared UEFI/QEMU profile.
The [host module-refactor regression](../../evidence/phase-1/m1-module-refactor/2026-10-18/README.md)
preserves that demonstrated behavior. Neither completes the broader ADR gates or
changes any record's Proposed status. P1-04 now adds
[fatal exception entry](../05-delivery/phase-1/exception-diagnostics-runbook.md);
P1-05 adds [owned mappings and guard integration](../05-delivery/phase-1/memory-ownership-runbook.md).
Check its story review; no whole-OS or user-mode isolation gate is implied.

| ADR | Decision | Evidence gate |
| --- | --- | --- |
| [0001](0001-system-language.md) | Rust foundation, narrow machine-specific boundary | Bare-metal compile, diagnostics and user ABI spike |
| [0002](0002-boot-and-iso.md) | UEFI/Limine, ISO separate from live updates | Cold boot with no host files |
| [0003](0003-kernel-and-isolation.md) | Small original kernel and isolated services | User-mode isolation and IPC failure tests |
| [0004](0004-versioned-boundaries.md) | Explicit versioned wire interfaces | Independent client/service compatibility fixtures |
| [0005](0005-live-update-transactions.md) | Transactional process/service replacement | Stateful reload plus fault injection |
| [0006](0006-app-runtime.md) | Native Rust baseline, optional Wasm interpreter | SDK process and bounded runtime experiment |
| [0007](0007-development-transport.md) | Local bounded dev channel before general networking | Auth/replay/disconnect tests |
| [0008](0008-rendering-path.md) | Shared CPU renderer before GPU stack | Same scenes hosted and in guest |
| [0009](0009-persistent-generations.md) | Immutable generations, distinct mutable data | Torn-write and recovery tests |
| [0010](0010-capability-security.md) | Least-authority handles and explicit trust model | Rights, stale-handle and escalation tests |
| [0011](0011-ui-authoring.md) | Provisional custom Rust after Slint comparison | Complete hosted core/reload; M4 guest confirmation |
| [0012](0012-network-and-crypto.md) | Isolated stack and maintained crypto | Stack/resource/time/entropy integration |
| [0013](0013-build-and-distribution.md) | Pinned build inputs and traceable artifacts | Two clean builds and boot evidence |
| [0014](0014-quality-and-scope.md) | Correctness gates and measured efficiency | Gate evidence and workload baseline |
| [0015](0015-typed-actions.md) | Shared typed actions and scoped context | UI/automation parity, rights/revision/reload tests |
| [0016](0016-optional-inference.md) | Optional isolated inference after baseline | Small-model quota/crash tests; no core release dependency |
| [0017](0017-bootstrap-timekeeping.md) | ACPI-discovered xAPIC/HPET bootstrap uptime | Real timer delivery, monotonic reads, masks, idle and failure diagnostics |
| [0018](0018-bounded-owned-memory.md) | Fixed transactional table banks and reclaiming owned page runs | Failure rollback, alias/pin audits, guest root switching and exact reclamation |
| [0019](0019-static-images-and-trusted-launch.md) | Coherent static boot images and staged trusted launch records | Independent ELF/page/startup audit, authority/reuse denial and complete M1 regression |
| [0020](0020-user-entry-and-fault-boundary.md) | Single-process SYSCALL/IRETQ and contained user failure | Real CPL3, checked copies, same-boot fault recovery and complete M1 regression |

## Lifecycle

Proposed → Accepted after gate evidence and project review; Accepted → Superseded
when replaced. Record who/what supplied approval or technical evidence. Do not erase
the historical reason for a choice. A failed gate may lead to a revised proposal.

New records include context, decision, alternatives, consequences, verification,
reconsideration triggers and links to sources/specifications. No ADR can override
the user's original-kernel/no-hidden-reboot requirements silently.

- [ADR-0021: Preempt owner threads and eagerly preserve baseline FP state](0021-preemptive-owner-threads-and-eager-fp.md) - Proposed; P3-05 experiment.

[ADR-0022: retained memory objects and handle ceilings](0022-retained-memory-objects-and-handle-ceilings.md) remains Proposed; P3-06 owns its experiment.

[ADR-0023: atomic queued authority and cycle collection](0023-atomic-queued-authority-and-cycle-collection.md) remains Proposed; P3-07 owns its experiment.

[ADR-0024: elapsed deadlines and bounded wait completion](0024-elapsed-clock-and-bounded-wait-completion.md) remains Proposed; P3-08 owns its experiment.

[ADR-0025: prepaid subtrees and quiescent process reaping](0025-prepaid-subtrees-and-quiescent-process-reaping.md) remains Proposed; P3-09 owns its experiment.

[ADR-0026: ephemeral scoped policy and exact forwarding tickets](0026-ephemeral-scoped-policy-and-exact-forwarding-tickets.md) remains Proposed; P3-10 owns its experiment.

[ADR-0027: ephemeral service routes and mandatory writer fences](0027-ephemeral-service-routes-and-mandatory-writer-fences.md) remains Proposed; P3-11 owns its synthetic guest gate.
