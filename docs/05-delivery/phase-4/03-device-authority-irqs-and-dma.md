# P4-03 — Mediate device resources, IRQs and DMA lifetimes

Issue type: Story. Epic: P4. Status: Not started. Owner: Kernel/Platform.
Depends on: P4-02. Backlog: B14/B16 device foundation.
Requirements: R03/R10/R14/R17/R18. Test families: T02/T03/T04/T06/T11.
Labels: `phase-4`, `devices`, `authority`.

## User story

As a driver, I need narrowly granted device resources and correct IRQ/DMA lifetime
rules so display, input and transport do not bypass process isolation.

## Implementation scope

1. Implement bounded discovery and trusted resource descriptions for the selected
   VM devices. Validate BAR/MMIO/port ranges, widths/alignment and overflow; preserve
   boot reservations and ACPI/cache policy. Enumerated IDs never grant ownership.
2. Define device grants, checked mapping and IRQ wait/ack/mask semantics. Ordinary
   apps cannot map hardware, acknowledge another device or replace trusted input.
   Keep interrupt handlers bounded; perform parsing/rendering outside IRQ context.
3. Implement required VirtIO feature/queue negotiation and DMA buffer ownership,
   physical address/length checks, alignment and architecture-specific ordering.
   Distinguish CPU-private mappings from the trusted DMA device/driver boundary.
4. Implement stop/drain/reset/ack completion before reclaim. On timeout retain or
   quarantine device-owned buffers under a finite budget. Document temporary kernel
   mechanisms and extraction work instead of silently making all drivers kernel code.

## Acceptance criteria

- [ ] Only trusted role grants expose the exact selected device resources; forged,
  stale, overlapping and over-right mappings/IRQ operations deny in real guests.
- [ ] Supported features/queues work; unknown required features, malformed chains,
  invalid lengths and capacity exhaustion fail before unsafe device access.
- [ ] Lost/duplicate IRQ, cancellation, driver death and reset-under-load preserve
  ownership; no buffer is freed while device access remains possible.
- [ ] DMA/cache/ordering unsafe review and actual supported device matrix are
  recorded; absent IOMMU confinement is explicit.
- [ ] Existing memory, interrupt, syscall and M3 enforcement regressions pass in
  each affected flavor; serial recovery remains usable during device failure.

## Tests and evidence

Use optimized guest driver tests and deterministic bad-descriptor, queue-full,
missing-IRQ and reset hooks. Independently inspect grant ranges, DMA inventories,
ack/mask events and exact settlements/quarantine caps. Pure queue models alone
cannot accept actual device delivery or reset completion.

## Exclusions and handoff

This supplies device primitives, not completed input, display or development
transport. P4-04/P4-06/P4-12 own their actual device workflows. No IOMMU/GPU or
physical-driver claim. Read [driver design](../../02-architecture/drivers-and-hardware.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
