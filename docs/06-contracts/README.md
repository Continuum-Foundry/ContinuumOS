# Draft interface contracts

These are version 0.1 design drafts, not implemented or frozen ABIs.

P1-03's internal `BootInfoV1` and version-2 serial diagnostics are documented in the
[boot-metadata runbook](../05-delivery/phase-1/boot-metadata-runbook.md).
P1-04's [exception extension](../05-delivery/phase-1/exception-diagnostics-runbook.md)
updates the diagnostic records to version 3 with setup/fatal events and an emergency flag.
The owned Rust value is not a persisted or process ABI, and diagnostic records do
not implement the IPC, update or package contracts below. The host harness keeps
their assertions in [m1/records.rs](../../tools/xtask/src/m1/records.rs).

| Interface | Specification | Machine-readable material |
| --- | --- | --- |
| Kernel/service IPC | [Objects and IPC](ipc-and-objects.md) | Codec/ABI IDs frozen at M3, not supplied yet |
| Development updates | [Wire format](update-wire-format.md) | [Schema](schemas/update-manifest.schema.json), [synthetic fixture](examples/update-manifest.json) |
| UI/compositor | [Scene protocol](ui-and-scene-protocol.md) | Freeze encoding after renderer/IPC experiments |
| Packages/generations | [Package format](package-manifest.md) | [Schema](schemas/package-manifest.schema.json), [synthetic fixture](examples/package-manifest.json) |
| Typed actions/context | [Action contract](actions-and-context.md) | Draft semantic contract; wire IDs frozen after independent SDK tests |

Schemas use JSON Schema 2020-12. Structural validation is only one step: validate
cross-field epochs, base generations, package graph, path safety, permissions,
actual file digests/sizes and state compatibility separately. No authentication or
signature construction is supplied by a JSON schema.
[S73](../01-research/sources.md#s73)

Example digests and IDs are synthetic. There are no matching executable binaries.
They must not be used as real installation or update packages. The package schema
allows an absent entry for asset bundles; executable kinds require a matching
validated entry file through semantic policy. Compatibility's `min_minor` is a
minimum supported ABI minor, not the producer's runtime version.

During implementation, add independent encoders/decoders and negative semantic
fixtures before assigning permanent syscall/message numbers. Keep schemas,
examples, prose contracts and generated bindings synchronized.
