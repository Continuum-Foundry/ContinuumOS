# ADR-0017: ACPI-discovered xAPIC timer calibrated against HPET

Status: Proposed. Date: 2026-10-19.

## Context and decision

P1-06 needs real interrupt return, monotonic uptime and idle wakeup before M1's
final review. The pinned Q35 profile exposes APIC and HPET; availability still
requires validated guest discovery and calibration. Use bounded RSDP/XSDT/MADT/
HPET parsing, explicit RO table-copy windows and supervisor UC device mappings.
Mask PIC/IOAPIC and other LAPIC sources before unmasking only the periodic timer.

Count delivered events in a non-nested, allocation-free handler with full GPR
preservation and IRETQ. Calibrate divide-by-16 LAPIC counts over an observed HPET
interval. Initialize uptime once, use checked integer conversion, and report its
effective tick resolution. Masked/lost/coalesced interrupts mean this uptime can
lag elapsed time; no TSC-frequency, wall-clock or precision assumption is allowed.

## Alternatives and consequences

PIT calibration, TSC-deadline mode and HPET interrupt routing remain alternatives,
not silent fallbacks. Unsupported discovery/cache policy/calibration rejects and
requires a new documented experiment. Fixed storage and single-core ownership
avoid a heap, IRQ locks, scheduler and FP context changes. The limited ACPI topology
and PAT policy intentionally reject otherwise valid platforms beyond the baseline.

## Verification and reconsideration

See the [implementation runbook and acceptance status](../05-delivery/phase-1/interrupts-timekeeping-runbook.md)
for contracts, sources, unsafe review and exact cases. Gate: 100 real interrupts,
100 monotonic reads, fresh epochs, mask/unmask, register/DF sentinels, repeated halt
continuations, post-delivery fatal diagnostics and explicit malformed/unsupported/
calibration rejection. Reconsider for physical platforms, SMP, suspend, deadline
APIs or drift-sensitive consumers. This does not pass ADR-0003's M3 isolation gate,
complete B06 scheduling, or change existing ADRs' Proposed status.
