# ADR-0009: Immutable system generations and separate mutable data

Status: Proposed. Date: 2026-10-17.

## Context and decision

ISO content is read-only while live updates and documents change. Use immutable
hashed packages and coherent generation manifests. Keep documents/settings and
activation/checkpoint metadata on a separate optional writable image. Begin
ephemeral; gate durable updates on a tested append-only record store or a justified
filesystem alternative.
Keep boot-attempt/known-good recovery independent from in-session replacement.
A/B slots are a later packaging option, not a live state-transfer mechanism;
signed distribution needs its own trust/recovery design.

## Alternatives

Overwriting files in place risks mixed versions. Whole-disk snapshot rollback can
silently discard user data. A mature filesystem/library reduces design work only
if its platform, flush and licensing assumptions can be met.

## Consequences

Staging, garbage collection, crash consistency and rescue selection require code.
Content hashes verify bytes, not authority or durability. Keeping old packages does
not preserve compatibility with new user-data schemas.

## Gate and reconsideration

M6 tests torn writes, flush failure, full disk and recovery at every decision boundary.
Reconsider the storage mechanism if recovery complexity exceeds a portable library's
integration cost. Never claim durable success before a real flush path exists.
Test repeated failed boots, health marks, fallback and current user-schema
compatibility. See [boot recovery](../02-architecture/boot-recovery.md).

Evidence: [S55](../01-research/sources.md#s55), [storage spec](../02-architecture/storage-and-data.md).
