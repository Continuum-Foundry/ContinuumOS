# P3-02 — Build bounded allocation and owned user address spaces

Issue type: Story. Epic: P3. Status: Not started. Owner: Unassigned.
Depends on: P3-01 Done. Suggested labels: `phase-3`, `memory`, `allocation`.
Backlog: B05 follow-up, B11/B13 memory foundation. Requirements: R03/R10/R18;
preparatory R09/R14. Test families: T02/T03; T09 accounting groundwork.

## User story

As a process owner, I need zeroed private memory with bounded allocation and cleanup
so an image can be loaded without exposing kernel/other-process data or leaking frames.

## Implementation scope

1. Extend accepted frame/page ownership with fallible bounded kernel metadata and
   a reclaiming process-memory backend. Account requested/committed/resident bytes,
   allocator metadata, page tables and stack guards explicitly. Choose the smallest
   allocator sufficient for M3; record alignment, deallocation and realloc rules.
2. Construct independent address spaces with supervisor-only kernel mappings,
   eager zero-filled user pages, explicit user ranges and guarded user/kernel stacks.
   Enforce W^X/NX and checked ranges at every mapping/permission transition, including
   aliases. Keep loader, firmware, ACPI, MMIO and other M1 reservations intact.
3. Implement internal create/map/unmap/protect/destroy primitives and staged ownership
   before the P3-06 public handle API. Stabilize mappings during future user copies;
   document lifetime pins or equivalent synchronization and TLB invalidation on the
   single CPU. No unmap/reuse may invalidate an active kernel copy.
4. Precharge bounded memory ceilings and roll back partial address-space/allocator
   construction on each allocation failure. Keep an emergency diagnostic path that
   does not allocate. Define process-local allocation failure versus kernel emergency.
5. Add reclamation/inventory assertions and a memory runbook. The user allocator
   uses this backend once P3-04 supplies syscalls; test the backend directly now.

## Acceptance criteria

- [ ] Private address spaces, user/supervisor permissions, W^X/NX and guard mappings
  pass independent page-table/ELF audits; kernel aliases do not expose user writes.
- [ ] Allocation/free/realloc honors alignment and limits, zeroes newly exposed bytes
  and reclaims storage; P2's bump-arena behavior is absent from the runtime path.
- [ ] Failed mapping, permission change and partial construction preserve the prior
  valid state with no frame/reference/accounting leak.
- [ ] Mapping stabilization and teardown rules cover active copies and shared backing;
  internal deterministic interleavings pass before public syscall races are added.
- [ ] Existing actual M1 guard/RO/NX/double-fault and reservation regressions pass;
  hardware cross-process enforcement is explicitly handed to P3-04/P3-06.

## Tests and evidence

Exercise zero-length/overflow/noncanonical/unaligned ranges, cross-boundary pages,
conflicting aliases, double-free/stale ownership and failures at every allocation
stage. Repeatedly create/destroy spaces and reuse poisoned pages, verifying zeroing,
fixed allocator growth bounds and exact frame/page-table inventories after settlement.
Retain representative mapping walks, failure rollback traces and affected M1 results.

## Exclusions and handoff

No user-mode execution, public capability API, demand pager, swap, DMA allocator or
desktop memory acceptance. P3-03 receives an owned loader destination; P3-04/P3-06
must exercise the real ring-3 copy/mapping enforcement and public rights boundary.
Sources: [memory runbook](../phase-1/memory-ownership-runbook.md),
[kernel design](../../02-architecture/kernel-and-execution.md),
[objects contract](../../06-contracts/ipc-and-objects.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
