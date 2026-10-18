# Draft contract: packages and generations

Status: design draft, version 0.1. Schema: [package manifest](schemas/package-manifest.schema.json).
Fixture: [package example](examples/package-manifest.json).

## Package

A package is immutable content plus a strict versioned manifest. It declares package
ID/version, source revision, component kind, entry point where applicable, files
with size/digest, compatibility, authority requests, resource limits and update class.
Manifest schema is syntax validation only; policy and compatibility are separate checks.

An entry point must name a validated file in the package. File paths are relative,
normalized and reject empty/dot/dot-dot components, absolute paths, drive prefixes,
NULs, duplicate/case-alias conflicts on host extraction and symlink escapes. Do not
infer safe extraction from a JSON regex alone. Prefer flat content-addressed blobs
with a logical path table instead of general archive execution.

Content identity hashes canonical manifest bytes plus a defined file-digest table,
or uses a separately specified deterministic container. The package digest is not
stored inside the bytes it hashes. M0/M4 chooses and freezes canonical encoding;
no home-grown signature scheme is implied. Hashes provide integrity, not publisher
authorization or permission grants.

## Permissions

Required and optional capability names describe requests, not automatic entitlements.
The launcher/supervisor resolves them through policy. Example authorities include
own surfaces, selected directory access, scoped network endpoints, settings namespace
and diagnostics. A reload cannot increase authority unnoticed. Document per-resource
constraints rather than accepting one vague `system` permission.

## Compatibility

Record kernel ABI major/minimum minor, required CPU features and service protocol
requirements separately from app release version. A component's state schema is
explicit, including identical-schema migration support. An update class can be
upgraded in disruption level by the planner if dependencies require it.

## Generation graph

A generation manifest selects exact package digests, configuration digest, boot/kernel
identity and compatible dependency edges. Its digest is calculated outside the
manifest itself. No mutable `latest` pointer is a dependency. Detect dependency cycles,
missing objects and incompatible required interfaces before activation.

An ISO references a coherent generation. A development guest may overlay a newer
compatible generation on writable storage; the kernel identity still constrains it.
If a new generation requires a different kernel ABI, stage it for reboot instead
of attempting user-space activation.

## Garbage collection and recovery

Root the boot base, active generation, last good generations, staged decided
transactions, live handles and retained checkpoints. Collect only unreferenced
content after safe retirement. Do not delete the only binary able to read a user's
state schema. Bound retention/storage and fail staging with a useful error when
safe collection cannot create headroom.

## Release metadata

Publish package and image digests, source revision, tool versions, SPDX inventory,
provenance, support matrix and known limitations. Development images include visible
mode markers; distribution images use a separate trust root and disable dev updates.
The illustrative fixture uses synthetic digests and is not installable content.
