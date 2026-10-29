# ADR-0022: Retained memory objects and immutable mapping ceilings

Status: Proposed. Date: 2026-10-27. Gate: P3-06, then P3-07/P3-12/P3-13.

## Decision and reason

Separate process-local handles, internal object identity and allocation lifetime.
Track handle and mapping refs independently; a memory object retains its backend
allocation across creator death until its last legal user is gone. Preserve the
original mapping's permission ceiling after handles close. Never infer authority
from a user principal, owner ID, virtual address or guessed object number.

Use fixed precharged registries and self-only process/thread grants for this
bounded gate. Share only trusted immutable R/NX snapshots. User-created backing
has an immutable READ, READ|WRITE or READ|EXECUTE ceiling, conservatively denying
RW-to-RX transitions. General mutable sharing and public transfers remain later
contracts. The backend's whole-allocation W^X checks remain defense in depth.

Each handle installation consumes a boot-wide nonzero u32 nonce; the low u32
is slot+1. This makes a copied foreign numeric token invalid in every other
table within the boot. Exhaustion fails all new installations until reboot.
Internal object u64 generation exhaustion retires its slot. This strengthens the
previous per-slot-wrap draft at the cost of finite boot-wide installation supply;
no unpredictability or cryptographic authority is claimed.

## Alternatives and consequences

Keeping creator teardown Busy on foreign references would block normal process
reaping. Eagerly freeing its backing would invalidate surviving grants. Universal
revocation would require mediated access or mapping invalidation protocols outside
this story. Instead detached backing remains visible in global resource accounts.
Close is narrow; direct duplicates and existing mappings deliberately survive.

Immutable backing ceilings simplify alias reasoning, while excluding JIT and
executable mutable buffers. A later executable-publication protocol must separately
validate staging, alias retirement and cache/CPU obligations. Finite global nonces
avoid per-process numeric collisions but can exhaust earlier than independent slot
counters. Expanding capacity or changing wrap policy requires reviewed tests/ABI
updates. Single-CPU completed syscalls and copy pins remain mandatory.

## Evidence gate and reconsideration

Use the [runbook](../05-delivery/phase-3/handles-memory-runbook.md) and story review.
Require actual guest denials, immutable shared reads after creator death, five
hardware faults, copy/close/unmap/removal interleavings and exact settlements.
Host tests force generation exhaustion and allocation/table admission failure.
P3-07 must preserve these refs through atomic transfer; P3-10 must explicitly
separate brokered invalidation from surviving direct authority. P3-12 owns ABI
freeze, P3-13 sustained adversarial resource review. This ADR remains Proposed.
