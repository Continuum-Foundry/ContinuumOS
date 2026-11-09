# P4-03 device authority and DMA probe

Status: [passes local Codex self-review](../../../evidence/phase-4/P4-03/final-1794185451085334400/README.md) for the declared bounded primitives. This is not
a completed desktop, input service or security certification.

## Scope and commands

The temporary trusted kernel mediator uses pinned one-core Q35/TCG/OVMF optical
boot. Device-test adds modern PCI VirtIO keyboard at 00:04.0, tablet at 00:05.0
and serial controller at 00:06.0. The `org.continuum.dev` serial port terminates
at QEMU's null character backend. It is not an authenticated transport.
Diagnostic UART remains separate. Ordinary/historical flavors do not select
device-test or its experimental operations. No new SDK contract is frozen.

```powershell
pwsh -NoProfile -File tools/Test-P4-03.ps1 -Plan
pwsh -NoProfile -File tools/Test-P4-03.ps1
```

`config/p4-03-validation.json` records impact, consumers, omissions and the
14-launch ceiling. The adapter requires a fresh device probe, 17 separate metadata
builds and ordinary desktop boot plus selected affected regressions. The 30000-ms
guest deadline and inherited guest-local bounds remain. Run serially in the
approved signer/xorriso context; keep the host awake without changing settings.
Full runs/ISOs/source/context snapshots remain under ignored `build/m4/P4-03`,
`build/m1` and prerequisite directories. Failed/interrupted attempts never pass.
Resume requires unchanged implementation/context/artifacts and re-review.

## Resource and authority boundary

Root-bus discovery reads at most 256 function headers and follows at most 48
capabilities/function with cycle/alignment/extent checks. Selected modern IDs
are 1af4:1052 input and 1af4:1043 serial. IDs are discovery facts; trusted launch
bindings supply process incarnation and role entitlement. Apps receive no PCI
configuration ports or hardware page-table mappings.

The narrow adapter supports one 64-bit MMIO BAR within the checked 52-bit physical range containing
all four VirtIO capability windows, with power-of-two size 4–64 KiB. Decode and
bus mastering are disabled during BAR sizing. Both BAR words are restored;
only Command's halfword is written, avoiding W1C Status. BARs and capability
windows must not overlap. Every page passes existing device range/cache and
reservation checks before supervisor RW/NX/UC mapping. Boot RAM/modules/ACPI/
framebuffers are not reclaimed. Unknown layouts fail without legacy fallback.
No bridge, hotplug or MSI support is claimed.

The broker binds exact ranges and MAP/IRQ/DMA ceilings to a process incarnation.
Checks reject overflow, unsupported widths, unaligned/out-of-range access,
overlap, wrong owner, stale grants and over-rights. Experimental CPL3 operations
0x4000–0x4003 exist only in fixture registration and expose entitlement/range
validation and denials, not raw MMIO, CPU pointers or execution-state selection.
Role 2 receives input descriptions; role 5 the serial description. Other roles
deny even within the same synthetic session. These are not stable syscall IDs.

## Queues, IRQ and lifetime

The implementation follows [VirtIO 1.2](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html),
sections 2, 4.1 and 5.8. VERSION_1 alone is negotiated. Optional offered bits are
omitted; unsupported driver-required bits deny. No packed/indirect rings,
event-index, multiqueue or noncoherent architecture is supported.

Each input device has eight-slot event/status split queues, with four admitted
outstanding buffers per queue. One exclusive 32 KiB frame run contains both
queues. Keyboard LED entries exercise actual consumption only when NumLock is
advertised. The tablet's unsupported LED feedback is rejected locally; test-only
SYN_REPORT no-op sentinels exercise its status queue's byte consumption. The
selected QEMU device ignores these sentinels; this claims no tablet feedback
feature or general fallback workflow. Physical input delivery
and parsing remain P4-06. Serial proves discovery/negotiation/reset only;
port/control-queue delivery remains P4-12. Descriptors and addresses use explicit
little-endian fields. Checked chains fit admitted physical backing with valid
flags, lengths, direction/next indices and no cycles/overlap. Used IDs, duplicate
completions and returned lengths are checked before ownership bookkeeping.

CPU stores precede release/sequence fences before avail publication/notify;
used reads precede acquire fences. Coherent x86 RAM is an explicit assumption;
volatile access alone is not the ordering argument. Common 64-bit addresses use
two 32-bit accesses while queues are disabled, before DRIVER_OK/publication.

The checked Q35 host/LPC IDs and root slots 4–6 select GSIs 20–22 using the emulator's fixed PIRQ[E-H] swizzle. The legacy PCI Interrupt Line byte is not used as an APIC route. These GSIs use active-low level IOAPIC routing to vector 66.
The returning entry preserves GPRs/DF and the existing FP trap policy. IRQ work
is bounded to three ISR byte reads, scalar atomics and LAPIC EOI. There is no
allocation/logging/parsing/rendering in IRQ context. Per-function INTx disable
permits missing-IRQ tests while used polling progresses. Broker IRQ latches
coalesce duplicate notifications; masks retain pending state and exact-sequence
ACK denies duplicate/stale/wrong-owner requests.

Stop/cancel/death disables new submissions. Neither a used entry nor a timeout
makes live ring memory reclaimable. The mediator masks INTx, writes status zero,
waits at most 100 ms for observed zero, drains ISR and removes bus mastering
before release. A hidden-completion hook tests retention when success cannot be
observed; recovery requires subsequent actual reset observation. Admission prepays
worst-case quarantine: at most three devices/16 MiB, including rings/payloads.

The input-role process holds a live kernel-mediated lease with pending event
buffers. Its actual process exit invokes device reset and grant revocation before
process backing is destroyed. Unknown reset completion retains DMA and fails the
probe. Authority revocation alone never releases device-owned physical memory.

## Unsafe review and extraction

PCI accesses use fixed kernel-only CF8/CFC ports and bounded selectors. MMIO
pointers come from checked UC supervisor mappings; references to device storage
do not escape. DMA pages come from the existing zeroing frame allocator. CPU
aliases are supervisor RO/NX outside bounded staging writes; descriptor addresses
never become arbitrary CPU read pointers. Registration is single-CPU/IF-clear;
the nonnested IRQ handler touches atomics/MMIO only. No broker/memory borrow crosses
user dispatch. Teardown clears ISR routing before unmapping; quarantine is never
returned to the allocator without reset completion.

No IOMMU confines DMA. Devices and the temporary kernel driver remain trusted
for memory integrity; CPU isolation is not malicious-DMA confinement. P4-04 owns
framebuffer presentation, P4-06 input parsing/routing and driver extraction,
P4-12 serial framing/control/transport. Toolkit, shell, parsers and publication
belong in user processes. This is not a general driver framework or a permanent
decision to put all drivers in the kernel. GPU/physical hardware remain unsupported.

## Independent evidence

The raw reviewer checks actual IDs/BARs/IRQs, queue address arithmetic, used IDs/
lengths, IRQ and polling phases, reset-before-release order, quarantine inventories,
trusted CPL3 launch/call identities and final physical settlement. It reopens
actual kernel/user ELF and ISO bytes and rejects missing criterion records.
A host queue model cannot accept hardware delivery/reset. Retain compact reviewed
records, source/artifact hashes, outcomes and failures before marking P4-03 Done.

Settled `frames=0` means the tested dynamic frame pool returned to its baseline.
Kernel images, static metadata/table banks, firmware and device reservations remain
separately owned. It is not zero whole-OS memory use or a 256 MiB cold-idle pass.

Routing reference: [QEMU ICH9 root-slot routing](https://github.com/qemu/qemu/blob/master/hw/isa/lpc_ich9.c), `ich9_cc_init` and `ich9_pirq_to_gsi`. This is a selected-board adapter, not general ACPI AML routing.
