# P1-03 — Validate boot metadata and establish identity

Issue type: Story. Epic: P1. Status: Not started. Owner: Unassigned.
Depends on: P1-02 Done. Suggested labels: `phase-1`, `boot`, `diagnostics`.
Backlog: B04. Requirements: R01/R03/R17/R18; preparatory R14 interface discipline.
Test families: T01/T02; no process ABI or security-completeness claim.

## User story

As an OS developer, I need checked, owned boot descriptors and correlated diagnostics
so later memory code cannot mistake arbitrary loader metadata for usable resources.

## Implementation scope

1. Normalize loader data behind an internal versioned BootInfo boundary. Cover
   memory regions, kernel/boot-module ranges, direct-map information, framebuffer
   descriptors and the ACPI root locator. Bound counts/strings and check all range
   arithmetic before walking tables or dereferencing nested data.
2. Separate required responses from optional features: memory/loader prerequisites
   fail closed; an absent optional framebuffer yields an explicit serial-only mode.
   Reject malformed supplied descriptors even when the feature is optional.
3. Copy descriptors and bounded strings into kernel-owned fixed storage, retaining
   explicit reservations for backing memory still needed later. Store addresses as
   checked address values with ownership/lifetime rules; do not expose raw loader
   pointer structures as an IPC or persisted format.
4. Validate framebuffer pitch, dimensions, address/byte-length arithmetic and pixel
   format without rendering a desktop. Validate only the ACPI root locator needed
   for handoff here; table parsing/device discovery belongs to P1-06.
5. Add bounded serial records carrying event version, build ID, boot-session ID,
   sequence, stage and outcome. Use a host-generated per-launch session ID passed
   through configuration; label its origin. A kernel-generated secure random ID is
   not assumed. Only P1-06 introduces timekeeping.
6. Add `boot-metadata` and `boot-reject` cases with separately identified fault
   configuration. Fault injection must not alter normal acceptance artifacts silently.

## Acceptance criteria

- [ ] Normal ISO boots produce `m1.boot_info_validated` and checked resource summaries.
- [ ] Two fresh VM launches have distinct session IDs and the same build ID for the
  same kernel; each run keeps one immutable session ID across all checkpoints.
- [ ] Missing required responses, unsupported protocol revision, arithmetic overflow,
  excessive counts and truncated strings produce bounded rejection diagnostics.
- [ ] A missing optional framebuffer is reported accurately; invalid pitch/format/
  range fixtures cannot reach memory ownership or a false ready marker.
- [ ] Host manifest maps build/session identities to the final ISO hash; no circular
  self-hashing scheme or claimed kernel uptime is introduced.

## Tests and evidence

Use pure host tests for normalized descriptor validation, including zero lengths,
overlaps, overflow and capacity exhaustion. The adapter must establish its loader
trust bounds before accessing raw pointers; host fixtures cannot prove safe arbitrary
pointer dereference. Add guest injection cases after safe handoff to demonstrate
rejection logs and no later-stage marker. Archive fixtures and the injection identity.

Retain successful and rejected guest logs, identity comparisons, normalized metadata
summaries and reservation/lifetime documentation. Do not include host secrets in logs.

## Exclusions and handoff

No paging ownership, memory reuse, compositor, clock or complete ACPI interpreter.
P1-04 receives bounded metadata and serial output; P1-05 will validate disjoint
allocation ownership and preserve every live reservation.
Sources: [boot/platform contract](../../02-architecture/boot-and-platform.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation revision: Not implemented. Evidence: None. Review: Pending.
