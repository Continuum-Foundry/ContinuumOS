# P1-05 — Own physical frames and page tables

Issue type: Story. Epic: P1. Status: Not started. Owner: Unassigned.
Depends on: P1-04 Done. Suggested labels: `phase-1`, `kernel`, `memory`.
Backlog: memory/guard subset of B05. Requirements: R01/R03; early R10/R17 support.
Test families: T02/T03; no userspace isolation claim.

## User story

As an OS developer, I need explicit ownership of usable frames and mappings so
the kernel can allocate pages without overwriting firmware, its own state or devices.

## Implementation scope

1. Build a checked region/reservation model from P1-03's descriptors. Reserve kernel
   segments, live bootstrap/page-table data, modules, metadata, all active/emergency
   stacks, framebuffer and firmware/MMIO ranges before making frames allocatable.
2. Implement a bounded 4 KiB physical-frame allocator with fixed bootstrap metadata,
   explicit supported capacity, eager zeroing and honest exhaustion results. Align
   eligible ranges inward; do not reclaim partial pages or silently ignore overlaps.
   Reclaim loader memory only after its full ownership/lifetime audit; retain it
   reserved for M1 if that audit has not passed.
3. Create owned page tables preserving only required boot mappings during transition.
   Apply supervisor-only permissions, read-only executable text, writable non-executable
   data/stacks and NX where the declared CPU supports it. Remove writable aliases
   of executable kernel pages; explicitly audit any retained direct mapping.
4. Switch to owned page tables and guarded kernel/emergency stacks without losing
   instruction access, serial output, metadata or the exception tables. Enable and
   verify the CPU's required permission enforcement rather than checking PTE bits alone.
5. Add checked typed address/range helpers, mapping conflict detection and the
   necessary single-core translation invalidation. Bound every allocation/mapping
   operation; no general heap, DMA allocator or memory overcommit is introduced.
6. Add `memory-ownership`, `memory-exhaustion`, `guard-fault`, `write-protect-fault`,
   `execute-disable-fault` and `emergency-stack` cases. Keep fault injection explicit.

## Acceptance criteria

- [ ] Normal boot reaches `m1.memory_ready` on owned page tables and stacks.
- [ ] No allocated frame overlaps a reservation or another live allocation; each
  newly supplied frame is zeroed and correctly aligned.
- [ ] Invalid/double frees, out-of-range addresses, overlapping maps, range overflow,
  metadata capacity exhaustion and allocation exhaustion have explicit outcomes.
- [ ] Guest mapping inspection and actual protection-fault tests establish text/data
  permissions, NX behavior and stack guards; host PTE fixtures alone do not pass.
- [ ] Stack-guard/emergency-stack fault tests produce bounded P1-04 diagnostics.
  The controlled double-fault case uses its emergency stack and does not reset QEMU.
- [ ] Exceptions and prior ISO/identity checkpoints still work after the mapping switch.

## Tests and evidence

Use independently constructed host memory maps covering unsorted/overlapping entries,
unaligned edges, holes, live boot data, address wrap, all-reserved memory and finite
exhaustion. Guest tests allocate/release a bounded batch and verify zeroing/ownership.
Use a declared test-only capacity limit for exhaustion rather than filling the host.

Record reservation tables, accounting totals, allocator configuration, owned mapping
summaries, symbolized protection faults and the before/after page-table switch log.
Test-only instrumentation must not silently replace the normal memory policy.
An expected protection fault is not a successful normal boot or safe recovery result.

## Exclusions and handoff

No userspace, demand paging, swap, general heap, SMP shootdowns, IOMMU or DMA safety
claim. A working frame allocator does not satisfy the allocation-dependent UI probes.
P1-06 inherits mapped interrupt/timer data and fault-safe stacks. B05 completes only
when both this story and P1-04, including their integration tests, pass.
Sources: [kernel contract](../../02-architecture/kernel-and-execution.md),
[boot reservations](../../02-architecture/boot-and-platform.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation revision: Not implemented. Evidence: None. Review: Pending.
