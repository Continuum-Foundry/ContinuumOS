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

M6 also seeds [VM installer preparation](../05-delivery/installer-preparation.md):
target validation, boot/system/data separation, offline payload manifests and
interrupted first-generation recovery. Resolve how the firmware/loader reaches
boot-readable content separately from the proposed record store. A non-writing
installation planner is preparation; installed-disk boot and installer writes
require M7 B42–B44, mandatory for M8. M6 must select and experimentally validate
the installed layout and offline payload contract before M7 implementation.
Installation, updates and repair share the generation/activation mechanism; preserve
current user/account/ownership data through reinstall. No partition layout or
filesystem is selected by this documentation amendment.
M10/B49 uses the same activation machinery for ordinary verified software delivery;
M11/B50 separately backs up documents/portable settings to independent storage and
restores through explicit authenticated ownership mapping. Exclude credential/
privileged policy migration by default. Generation fallback and preserving
reinstall do not establish an independent backup or a full-machine restore.

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
Include incomplete first initialization, where there is no known-good generation
to fall back to. Review layout/boot dependencies at M6 and require ISO-detached
cold boots plus interruption/rescue evidence before declaring an installable variant.

Evidence: [S55](../01-research/sources.md#s55), [storage spec](../02-architecture/storage-and-data.md).
