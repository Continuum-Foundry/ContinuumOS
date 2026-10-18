# Boot, firmware and platform contract

Status: proposed. R01, R02, R03, R16. Decision: [ADR-0002](../04-decisions/0002-boot-and-iso.md).

## Initial boot chain

QEMU Q35 → pinned OVMF → pinned Limine UEFI loader → original kernel ELF →
initial supervisor and immutable boot modules. Keep the bootloader boundary small:
memory map, framebuffer, module descriptors, ACPI pointer and explicitly requested
CPU/firmware metadata. Validate every optional response before use.

Limine is reused infrastructure, not our kernel. Its release, boot protocol and
bindings must match. Firmware and loader are part of the trust assumptions.
[S25](../01-research/sources.md#s25), [S26](../01-research/sources.md#s26)

Revision 2 also evaluates Rust OSDev bootloader with exact-target/toolchain evidence.
Its documented output is a disk image; the chosen route must additionally cold-boot
the actual ISO. Normalize loader-specific data behind our BootInfo boundary.
[S81](../01-research/sources.md#s81)

An own-loader alternative can be authored in Rust as a UEFI application. It must
load/verify ELF segments, collect the final memory map, handle ExitBootServices
correctly, establish the handoff and stop using boot-only functions. That is a
separate milestone if strict first-party boot ownership becomes a requirement.
[S27](../01-research/sources.md#s27)

## Boot information

Normalize loader structures into our own versioned `BootInfo` after checking
counts, ranges and descriptor lengths. Never persist external raw pointers.
Include build ID, image generation, boot arguments, framebuffer pitch/format/size,
memory ranges and reserved modules. Record boot ID once; uptime uses a monotonic
clock initialized by the kernel, not the desktop process start time.

P1-03 implements an internal, allocation-free `BootInfoV1` boundary and version-2
serial record contract; see the [runbook and current evidence status](../05-delivery/phase-1/boot-metadata-runbook.md).
The demonstrated loader remains pinned at base revision 3 with four-level paging.
Required handoff responses fail closed; optional framebuffer/module/ACPI absence
is explicit. Owned copies and retained backing reservations are separate: no
loader stack, GDT, page table or other loader-reclaimable frame is released here.
Raw ELF backing may be loader-reclaimable; loaded kernel segments use their
executable-address response and linker bounds, including BSS. Non-usable memory
descriptors may overlap; usable and loader-reclaimable descriptors may not.

Bootstrap response/map readability remains a trusted loader/firmware prerequisite.
Bounds and canonical-address checks do not prove arbitrary pointer mappings.
The base-3 ACPI locator is physical and has no assumed alignment; recording it
does not establish that the table is mapped or its checksum valid. P1-06 owns
table mapping/parsing. Session IDs originate on the host and are copied once;
they provide correlation, not entropy, uptime, security identity or reset proof.

Framebuffer pitch may exceed width × bytes-per-pixel. Pixel channel ordering and
memory cache policy are explicit. Reserve mapped framebuffer/MMIO ranges. After
firmware handoff, use memory access and later our display driver, not firmware
graphics calls. [S28](../01-research/sources.md#s28)

## Platform discovery

Early serial output is independent of the graphical path. Validate ACPI table
checksums/lengths before extracting topology/interrupt/timer information. Use the
smallest required table subset first; a general AML interpreter, battery support
and suspend/resume are later projects. PCI discovery checks configuration access,
BAR sizes/types and resource conflicts. Never map a BAR merely because a device
reports a plausible address.

## ISO contents and persistence

The release ISO contains EFI boot files, kernel, init package/archive, package
manifest, default configuration, licenses, generation metadata and recovery tools.
Use a pinned El Torito UEFI recipe tested with the selected loader. Checksums and
build inputs live beside the ISO. An ISO filename extension alone proves nothing.

A live session can have a RAM-only writable layer or an explicitly attached data
image. Label the mode in the UI. Persistent activation state must reference packages
available after reboot; RAM-only updates cannot be advertised as durable. Rescue
boot should permit choosing the immutable base or a known good persisted generation.

Do not treat an optical ISO as a generally bootable USB disk. A hybrid image or
disk installer needs its own tested layout. No automated writing to physical disks
belongs in the initial development workflow.

## Host feasibility

M0 chooses actual QEMU/OVMF/xorriso/toolchain paths and records versions. On Windows,
WHPX may accelerate the CPU; TCG is a portability fallback for trusted tests. Neither
guarantees accelerated graphics. Optional WSL/container build use is a host tooling
choice, not a Linux kernel dependency in the guest.

Validate both cold boot from ISO and boot without a host shared folder or network.
Record launch config, serial transcript, screen evidence, generation and explicit
readiness markers. Secure Boot remains unsupported until a complete signing and
provisioning/recovery plan is implemented.
