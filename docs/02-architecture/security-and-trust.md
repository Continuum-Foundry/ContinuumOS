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

## Installer and account boundaries

The [installer/accounts roadmap](../05-delivery/installer-preparation.md) is mandatory
for M8. M3 reserves separate disk-installation/account-administration authority;
M4 tests trusted focus and capture/diagnostic restrictions with synthetic sensitive
events. M6 reviews guest entropy and credential-storage feasibility. M7 must select
and validate the maintained credential mechanism, entropy source and authorized
recovery policy before real credentials are accepted. Host-only tests, timestamps
and deterministic VM IDs cannot satisfy this gate.

The unprivileged installer UI invokes a target-scoped installation service and an
account broker over checked versioned interfaces. Confirmation binds to the target,
operation and exact offline payload; revalidate before writes. Developer authority
does not automatically grant account administration. First installed boot remains
in restricted setup until durable owner/account/policy state is complete. No
default password, unauthenticated role promotion or rescue login bypass is permitted.
Credentials and privileged installation handles stay out of ordinary logs, traces,
inspectors and app checkpoints. M5 blocks replacement of an active installation
writer pending a separate proven handover. M7/M8 tests grant escalation, capture,
interrupted setup, account disable/session outcomes and repair/reinstall preservation.

## Later desktop and compatibility boundaries

M9/M10 file pickers, clipboard/drag offers, associations and app management use
narrow object grants, not path/app-name authority. Review permission increases
and data deletion independently. M10/B49 ordinary update verification/trust and
M12/B54 online delivery share one activation/verifier path; a reconnect cannot
weaken validation. M11/B50 export/restore excludes credential and privileged policy
state by default and requires explicit authenticated ownership mapping. System
management cannot bypass active writer/service safety. M13 capture permissions
include actual stream revocation and a trusted indicator.

M16/B59 evaluates a foreign compatibility boundary, not a trusted app exemption.
Map foreign handles to scoped native resources and retain process/owner quotas;
no ambient Windows-looking administrative API or inherited live-reload guarantee.
M17/B60 requires a credible browser sandbox and maintenance path; disabling the
sandbox or merely launching an EXE is not security evidence. Both are optional
research outside M8/M14, as are M18–M21. M15 needs actual profile-specific driver/
DMA/trust review; selecting one PC does not establish universal hardware isolation.
