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

## Gate and reconsideration

M1 cold-boots the ISO with host shares/network disabled and records readiness. M8
repeats from packaged outputs. Change to an own loader if the user explicitly requires
first-party boot ownership or loader constraints impede the tested platform.

Evidence: [S25](../01-research/sources.md#s25), [S27](../01-research/sources.md#s27),
[boot spec](../02-architecture/boot-and-platform.md).
