# ADR-0002: UEFI bootstrap and independent ISO packaging

Status: Proposed. Date: 2026-10-17.

## Context and decision

We need a bootable original OS and rapid in-VM updates. Start with x86-64 QEMU Q35,
OVMF UEFI and pinned Limine/protocol releases. Package a standalone UEFI ISO. Live
updates enter the running guest separately; replacing the ISO file is not hot reload.

## Alternatives

Own Rust UEFI loader provides more first-party ownership but adds boot/file/ELF/
memory-map work before desktop features. Legacy BIOS increases initial complexity.
Direct emulator kernel loading is useful for debugging but does not satisfy ISO boot.
Rust OSDev bootloader is an explicit candidate; its disk-image workflow needs a
separate tested ISO recipe and an audited toolchain path. Changing an extension
does not pass this gate. [S81](../01-research/sources.md#s81)

## Consequences

Firmware/bootloader remain upstream trusted components and may include C. This is
consistent with the documented original-kernel interpretation, not strict first-party
code from reset. An ISO needs an explicit writable data image for durable changes.
The approved 2026-10-21 roadmap amendment additionally requires a complete VM
installer for M8. M6 selects and experimentally validates boot-readable layout,
early driver availability and firmware discovery; M7 B42–B44 implement and accept
installed-disk boot. Preserve the standalone optical ISO recipe independently.
No partition/filesystem choice or bootloader change is selected by this amendment.

## Gate and reconsideration

Phase 0 review (2026-10-18): Limine 12.9.3's upstream ISO recipe and Rust OSDev
bootloader 0.11.17's disk-image path were reviewed. Neither was boot-tested.
Native QEMU, OVMF and ISO tooling remain missing; [M1 runbook](../05-delivery/phase-0.md).
No loader/protocol integration is accepted by this review; status remains Proposed.

P1-01 review (2026-10-18): [native host inputs](../05-delivery/phase-1/prerequisites-runbook.md)
are now supplied and locked. Limine 0.6.5 Rust bindings require nightly and were
rejected; 0.5.0 passes the narrow stable base-revision-3 compile probe. The pinned
loader source supports that base revision. Native xorriso executes outside the
sandbox's MSYS2 named-object restriction. Firmware/ISO boot and actual handoff
compatibility remain unverified; this decision stays Proposed.

M1 cold-boots the ISO with host shares/network disabled and records readiness. M8
repeats from packaged outputs. Change to an own loader if the user explicitly requires
first-party boot ownership or loader constraints impede the tested platform.

P1-02 evidence (2026-10-18): the original stable-Rust kernel reaches serial entry
through the pinned Limine/OVMF UEFI optical route. Two fresh QEMU processes boot
the same hashed ISO and return the expected emulator-only test exit. See the
[runbook](../05-delivery/phase-1/minimal-iso-runbook.md) and
[retained evidence](../../evidence/phase-1/P1-02/2026-10-18/README.md).
This validates the minimal loader/ISO integration, not full boot metadata,
M1 readiness, BIOS, Secure Boot or release reproducibility. Status stays Proposed.

P1-03 implementation retains these pins and adds a bounded owned BootInfo boundary,
explicit four-level paging, guest session correlation and separately identified
metadata fixtures. See the [implementation runbook and acceptance status](../05-delivery/phase-1/boot-metadata-runbook.md).
The initial integration experiments exposed two distinctions: the raw ELF buffer
can occupy loader-reclaimable memory rather than loaded-segment storage, and
OVMF's extended ACPI locator is not necessarily 16-byte aligned. Preserve the
former as live backing and the latter as an undereferenced physical byte address.
Do not assume base-4 ACPI mappings while negotiating base 3. These changes do not
add owned page tables, ACPI interpretation, arbitrary-pointer recovery or full
M1 readiness. This ADR remains Proposed pending its broader gate.

P1-03's pre-completion evidence review additionally found an orphan GOT after the
initial linker image-end marker. The link script now places GOT inputs explicitly
inside the image, and every build cross-checks LLVM boundary symbols against all
ELF load-segment memory extents, including BSS. Keep the superseded suite and its
failed review distinct from the corrected acceptance evidence. No frames were
reclaimed in that experiment; the complete non-usable map remained reserved.

P1-03 review (2026-10-18): the corrected full acceptance and local evidence review
pass, with 18 host tests and 17 bounded offline launches covering positive boots,
metadata rejections and ISO regressions. See the
[retained review](../../evidence/phase-1/P1-03/2026-10-18/README.md).
The loader choice remains provisional beyond this profile; this ADR stays Proposed.

Evidence: [S25](../01-research/sources.md#s25), [S27](../01-research/sources.md#s27),
[boot spec](../02-architecture/boot-and-platform.md).

P1-05 retains this same loader/firmware/ISO route and boot-data lifetime policy,
then replaces loader mappings with owned four-level tables. Only eligible RAM
has a direct mapping; all loader memory is retained and ACPI is not dereferenced.
See the [memory implementation and acceptance status](../05-delivery/phase-1/memory-ownership-runbook.md).
The broader M1/M8 gates remain open and this ADR stays Proposed.
