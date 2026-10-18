# Drivers and hardware support

Status: proposed. R02, R06, R10. Sources: [VirtIO](../01-research/sources.md#s32),
[driver HAL](../01-research/sources.md#s33), [DMA trust limits](../01-research/sources.md#s20).

## Staged device matrix

| Device | Bootstrap | Intended service form | Live replacement class |
| --- | --- | --- | --- |
| Serial UART | Kernel logging | Stable diagnostic core | Reboot initially |
| Interrupt controllers and timer | Kernel HAL | Kernel | Reboot |
| PCI configuration | Trusted resource enumerator | Mediated service after bootstrap | Restart only if resources remain valid |
| Framebuffer / display | Direct mapped boot framebuffer | Stable display service | Shell reload first; display restart later |
| VirtIO console/serial | Development transport candidate | Stable dev-channel driver | Do not replace over its sole active channel |
| VirtIO input | Keyboard/pointer | Isolated input driver | Drain, reinitialize, synthesize all-keys-up |
| VirtIO block | Virtual data disk | Isolated block driver | Only after requests/flush/DMA are drained |
| VirtIO network | Later development/general network | Isolated network driver | Reconnect sockets or expose interruption |
| VirtIO GPU 2D | Display-resource transport | Isolated display path | Surface reconstruction, no initial seamless promise |
| Audio | Deferred | Audio service/driver | Drain stream, recover with explicit discontinuity |

Each device has a tested configuration/feature list. Guest driver code must negotiate
supported VirtIO features and reject required unknowns. Start with one queue/device
and correctness before multiqueue optimizations. Do not silently fall back to a
different device ABI when negotiation fails.

## Resource model

The supervisor grants only a device's MMIO ranges, IRQ handles and DMA buffers.
The DMA API returns CPU/device addresses, size/alignment, direction, mapping lifetime
and synchronization obligations. Volatile access alone does not provide DMA ordering
or cache coherence. Validate descriptor chains and bounds even in a trusted VM.

Ordinary user page tables do not stop an authorized DMA device from accessing other
physical memory. Until IOMMU enforcement is implemented and tested, DMA-capable
drivers/devices remain within the memory-integrity trust boundary. Describe them
as isolated from CPU memory accesses, not fully confined against malicious DMA.

## Reset and lifetime

Before replacing a driver: stop new submissions, drain or cancel requests under a
deadline, ensure the device can no longer DMA into retired buffers, acknowledge
interrupts, reset using the device's actual completion rule, then release mappings.
Never free buffers while the device may still own descriptors. Timeouts retain or
quarantine resources and fail safely rather than guessing completion.

For block I/O, a completed write is not necessarily durable; distinguish submission,
device completion and flush/barrier completion. For input, preserve physical key
state in a stable input layer and release stuck keys after reset. For network/audio,
uninterrupted external stream continuity is not assumed.

## Portability contract

Drivers target narrow abstract interfaces, but platform-specific discovery and
memory operations stay visible. Real PCs introduce UEFI variation, ACPI/AML,
USB host controllers, storage controllers, GPU initialization and power management.
Supporting QEMU VirtIO does not imply supporting a laptop.

## Required tests

Negotiate unsupported features, submit malformed/oversized descriptors, lose an
interrupt, reset under load, kill a driver, exhaust DMA memory, and detach where the
VM supports it. Trace object/buffer/IRQ cleanup. Test queues with compiler optimization
enabled; timing and memory-order bugs can disappear in debug builds.

M0 determines which libraries may be reused. Implementing a driver HAL is trusted
unsafe code, not boilerplate. Track temporary kernel drivers and extraction tasks
so the architecture cannot quietly become permanently monolithic.
