# ADR-0019: Coherent boot images and staged trusted launch records

Status: Proposed. Date: 2026-10-25. P3-03 engineering experiment.

## Decision and boundary

Use independently linked static Rust ET_EXEC fixtures, packaged as immutable
Limine modules on the existing optical ISO route. Compile exact SHA-256/size
descriptors into the experimental kernel. Validate bytes and complete ELF runtime
obligations before allocation, copy into eager owned backing, then publish only
a staged launch record after every segment/stack/startup operation succeeds.
Trusted bootstrap bindings carry principal, authority session, service class,
process generation and initial grant/resource ceilings. User startup remains a
64-byte explicit LE non-authoritative record. Never expose Binding as a user API.
See the [loader runbook](../05-delivery/phase-3/static-loader-runbook.md).

## Alternatives and consequences

Embedding ELF bytes inside the kernel would simplify lifetime but would not
exercise actual boot-module packaging. A dynamic linker, general package signature
or account store would add unrelated runtime/trust dependencies. Boot descriptors
only check coherence against the kernel selected on this trusted ISO; an attacker
replacing the whole ISO can replace both. This is no authentication/security claim.
Conservative limits and page-rounded disjoint LOADs reject otherwise valid ELF
features. Fixed metadata and eager copies cost bounded RAM and simplify rollback.
No public launch, handle authority, ring-3 execution or scheduler is implemented.

## Gate and reconsideration

P3-03 requires three real RX/R/RW/BSS fixture loads, independent ELF/page/table/
startup audits, failure cleanup, forged context/stale generation rejection and
the complete current M1 gate. The synthetic return is UD2 until P3-04 assigns
process_exit; P3-04 must replace it before any user entry. P3-09/10 must authorize
supervisors, derive child bindings and enforce delegated ceilings. Reconsider
the ELF subset for independently built SDK clients at P3-12, and image authentication
at the signed delivery gate. Retain staged cleanup and no ambient authority.
This ADR remains Proposed pending those documented gates and project review.

Sources: [runtime contract](../06-contracts/runtime-v0.md),
[owned-memory ADR](0018-bounded-owned-memory.md),
[identity model](../02-architecture/identity-and-authorization.md).
