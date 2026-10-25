# ADR-0018: Fixed table banks and reclaiming owned page runs

Status: Proposed. Date: 2026-10-25. P3-02 engineering experiment.

## Context and decision

M1 owns reserved frames and supervisor mappings but has no process allocation
backend. P2's bump arena cannot satisfy reclamation or transactional mapping.
Use bounded image-owned metadata and two 64-page hardware-table banks per space,
plus reclaiming, zeroed contiguous physical page runs from sealed M1 ordinary RAM.
Precharge complete storage within P3's 16-MiB kernel dynamic ceiling, independent
of VM capacity. Rebuild in the inactive bank and publish with CR3 invalidation.
See the [memory runbook](../05-delivery/phase-3/owned-memory-runbook.md) for exact
limits, accounting, alignment, ownership, pins, aliases and failure rules.

## Alternatives and consequences

Allocating every table from physical frames would reduce idle reserved storage
but needs a more complex rollback journal and table-reference lifetime audit.
In-place mutation with an undo log risks exposing partial permissions. A general
heap, demand paging and a subpage kernel allocator exceed this story's needs.
The selected banks spend bounded fixed RAM for simple atomic admission. Sparse
maps can exhaust 64 tables before 128 mapping slots or 64-MiB data commitment.
Contiguous runs may fail under fragmentation; realloc temporarily needs both
runs. Rebuild/CR3 and per-byte staging are correctness-first, unmeasured costs.

Shared mapping references conservatively keep the original owner alive with Busy;
P3-06 must introduce independently owned memory objects for public process death.
Cross-alias W^X applies to the entire backing allocation. Kernel data aliases are
RO/NX while published; staging writes require zero mapping references.

## Gate and reconsideration

P3-02 requires host failure/interleaving tests, independent ELF/table audits,
actual guest root/permission publication, 1000 exact settlement cycles and the
complete M1 regression gate. Record the resulting evidence in the story before
Done. This ADR remains Proposed: ring-3 faults, public rights, scheduling and M3
interop are later gates. Reconsider contiguous allocation and fixed banks if
P3-03's loader or measured M3 workloads cannot fit, fragmentation becomes material,
or M4 demonstrates unacceptable memory/copy/mutation costs. Changing table
allocation must retain staged failure and copy-lifetime guarantees.

Sources: accepted [M1 memory design](../05-delivery/phase-1/memory-ownership-runbook.md),
[runtime draft](../06-contracts/runtime-v0.md),
[original kernel ADR](0003-kernel-and-isolation.md). No third-party port is selected.
