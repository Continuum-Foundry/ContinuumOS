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
| M3 runtime startup/syscalls | [Implementation draft](runtime-v0.md) | [Revision-0 source](../../contracts/runtime-v0.json), generated offsets/names only; no ABI IDs frozen |
| Development updates | [Wire format](update-wire-format.md) | [Schema](schemas/update-manifest.schema.json), [synthetic fixture](examples/update-manifest.json) |
| UI/compositor | [Scene protocol](ui-and-scene-protocol.md) | Freeze encoding after renderer/IPC experiments |
| Hosted UI comparison | [Shared boundaries/corpus](hosted-ui-fixtures.md) | Version-1 [authored fixtures and lock](../../fixtures/m2/ui-v1/README.md); seven `hosted-*.schema.json` drafts |
| Hosted resource reload | [Experimental version-1 file boundary](hosted-resource-reload.md) | P2-11 bounded JSON builder/parser and negative fixtures in the [runbook](../05-delivery/phase-2/resource-reload-runbook.md); no process ABI |
| Hosted text | [Bounded pipeline and composition](hosted-text.md) | [Font/data/corpus locks](../../fixtures/m2/text-v1/README.md); no process ABI |
| Packages/generations | [Package format](package-manifest.md) | [Schema](schemas/package-manifest.schema.json), [synthetic fixture](examples/package-manifest.json) |
| Installation/account setup | [Installer responsibilities](../05-delivery/installer-preparation.md#service-boundary-and-draft-interfaces), [identity](../02-architecture/identity-and-authorization.md) | Draft service boundary only; M6/M7 define tested versioned encodings before freezing; no executable command or credential protocol supplied |
| Later desktop services | [Files/exchange/apps/updates/backup/admin responsibilities](desktop-services.md) | Draft ownership, progress/cancellation and grant responsibilities; no new runtime encoding frozen |
| Typed actions/context | [Action contract](actions-and-context.md), [hosted model](hosted-editor-model.md) | P2-03 [action schema](schemas/hosted-action.schema.json), [logical fixtures](../../fixtures/m2/actions-v1/README.md); process wire IDs remain unfrozen |

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

P3-10 adds the [experimental scoped policy protocol](../../contracts/policy-v0.json)
and its [use-time/restart runbook](../05-delivery/phase-3/policy-runbook.md).
The synthetic controller fields confer no policy authority; independent SDK
freeze remains P3-12.
