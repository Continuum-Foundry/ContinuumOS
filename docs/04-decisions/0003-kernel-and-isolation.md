# ADR-0003: Small original kernel with isolated services

Status: Proposed. Date: 2026-10-17.

## Context and decision

Live service replacement needs enforceable ownership/failure boundaries. Implement
an original kernel for memory, threads, scheduling, interrupts, handles and IPC;
move app, desktop, storage and networking policy to isolated user processes.
Track temporary kernel drivers as bootstrap exceptions with extraction work.

## Alternatives

A monolithic kernel is simpler initially but couples state and faults. An intralingual
single-address-space OS is interesting research but needs a different assurance
model. Existing seL4/Redox/Linux kernels would change the original-OS requirement.

## Consequences

More loader, IPC, routing and supervision work precedes rich apps. Isolation costs
must be measured; a microkernel label does not imply good performance or verification.
DMA-capable drivers remain trusted absent enforced IOMMU protection.

## Gate and reconsideration

P1-05 experiment (2026-10-19): the [memory implementation and acceptance status](../05-delivery/phase-1/memory-ownership-runbook.md)
use two fixed frame bitmaps covering physical addresses below 4 GiB and an
image-owned 2304-page arena for four-level 4 KiB tables. Usable memory beyond the
supported ceiling or metadata/table exhaustion fails explicitly. All loader
memory remains reserved; only eligible ordinary RAM has an RW/NX direct map.
Kernel text has no writable physical alias. Runtime and both IST stacks have
two absent end pages, with controlled #PF/#DF experiments. Static metadata avoids
introducing a heap to establish ownership; its 9 MiB arena is a bootstrap cost,
not a measured desktop footprint. Reconsider capacity and allocate tables from
owned frames only with a separate lifetime audit. ACPI/device mappings need
explicit validated range/cache policy in P1-06. This experiment does not pass
the user-mode isolation gate or change this ADR's Proposed status.

M3 proves user/kernel isolation, process faults, bounded IPC and no global lock held
across user waits. If costs are excessive, optimize measured paths/shared bulk data
before broadening privilege. Any architecture reversal needs a new ADR.
M19/B30 separately gates SMP context, scheduling, TLB/ordering and concurrent
update/recovery evidence. It is not a prerequisite of M8/M14, and a larger VM
vCPU count is not implementation evidence. M16's foreign-runtime investigation
must preserve native isolation and the original kernel rather than importing
Windows semantics as ambient kernel authority.

P1-06's bounded interrupt/timekeeping experiment is recorded in
[ADR-0017](0017-bootstrap-timekeeping.md) and its linked acceptance status. It
adds returning entry and delivered-tick uptime without scheduling, FP context,
SMP or a change to this ADR's Proposed status.

Evidence: [S14](../01-research/sources.md#s14), [S19](../01-research/sources.md#s19),
[layers](../02-architecture/system-layers.md).
