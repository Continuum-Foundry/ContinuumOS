# Security and trust model

Status: proposed threat model, not a security certification. R10, R12, R13, R16.

## Assets and adversaries

Protect kernel memory, app isolation, documents, credentials, update authority,
package integrity and host development files. Consider malformed app messages,
buggy native apps, malicious themes/fonts/packages, a compromised guest service,
replayed updates, malicious network peers and storage corruption. Initially trust
the local developer, firmware, chosen VM configuration and physical host; these
assumptions must be visible in release notes.

## Boundaries and authority

| Boundary | Control | Remaining risk |
| --- | --- | --- |
| App → kernel | Page isolation, checked syscalls, quotas, rights-bearing handles | Kernel/unsafe code bugs |
| App → services | Explicit grants, validated schemas, resource limits | Confused deputy or excessive permissions |
| Wasm → host | Validated module, limited imports, memory/fuel/stack quotas | Runtime bug or dangerous import |
| Driver → memory | CPU mappings plus mediated resources | DMA bypass until real IOMMU protection |
| Host → update agent | Dev authority, version/hash checks, bounded transport | Compromised host has authorized development power |
| Package → execution | Identity/integrity checks and declared permissions | Trusted signer can ship harmful code |
| Guest → host | Restricted transport surface and no automatic command execution | VM/device implementation bugs |

Rust helps enforce memory/lifetime rules inside safe code. It is not proof of
logical correctness, race freedom across external devices, isolation of arbitrary
unsafe code, or security of a whole desktop. seL4's documented assumptions are a
useful model for stating what assurance does and does not cover.
[S20](../01-research/sources.md#s20)

## Development versus distribution

Development images visibly enable update authority and diagnostics. Provision a
per-session credential over a trusted launch path; rotate/revoke it and bind requests
to boot/session IDs and sequence numbers. Never use a universal hard-coded secret.
Disable public listeners and automatic plugin execution. The raw ISO research does
not choose a production cryptographic protocol by inventing one.

Distribution images disable the development endpoint by default and use a distinct
update trust root. A later repository design should adopt maintained mechanisms
with role separation, signed metadata, expiry and anti-rollback state; TUF is the
reference. This is separate from permission to deliberately roll back to an
authorized known-good local generation. [S60](../01-research/sources.md#s60)

## Runtime hardening

Enforce W^X/NX, zero newly granted pages, scrub sensitive reused state, validate
ELF/archives/fonts/images and reject decompression bombs. Bound parser recursion,
message lengths and resource counts. Seed randomness from declared sources and
track readiness. Do not treat timestamps, CPU serial values or deterministic VM
seeds as cryptographically secure entropy. Strong randomness and trusted time are
gates for secure network protocols and signed metadata expiry checks.

Avoid storing secrets in ordinary checkpoints. Secret handles remain with a broker;
regrant after policy checks. Crash dumps and traces can expose documents/credentials,
so default logs contain metadata and explicit diagnostic export requires user action.

## Trusted computing bases

Memory integrity includes kernel, loader, unsafe HAL, DMA-capable components absent
IOMMU enforcement, and relevant firmware/VM. Update integrity additionally includes
coordinator, package verifier, persistence implementation and authority broker.
Visual privacy includes compositor, input/focus routing and screen capture policy.
Calling only the kernel “the TCB” would hide important dependencies.

QEMU security guarantees are configuration-dependent; TCG is not advertised by QEMU
as a guest-isolation guarantee. Use trusted development workloads and least host
exposure; do not market an experimental VM as a safe malware sandbox.
[S38](../01-research/sources.md#s38)

## Required security tests

Forge/attenuate/stale handles, race user mapping changes, overflow schemas, replay
updates, request extra permissions, corrupt payloads, exhaust queues, inject a
malicious font/image, and try guest-directed host file/command access. Add independent
review before public networking, installers or handling valuable credentials.
