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

M3 proves user/kernel isolation, process faults, bounded IPC and no global lock held
across user waits. If costs are excessive, optimize measured paths/shared bulk data
before broadening privilege. Any architecture reversal needs a new ADR.

Evidence: [S14](../01-research/sources.md#s14), [S19](../01-research/sources.md#s19),
[layers](../02-architecture/system-layers.md).
