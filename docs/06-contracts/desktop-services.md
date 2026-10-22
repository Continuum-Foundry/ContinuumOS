# Draft responsibilities: desktop services and operations

Status: proposed responsibilities, not a frozen API, implemented service or wire
schema. [Milestones M9–M15](../05-delivery/milestones.md) and the
[feature matrix](../05-delivery/desktop-feature-matrix.md) define release scope.
Use [IPC](ipc-and-objects.md), [typed actions](actions-and-context.md) and
[packages/generations](package-manifest.md). Freeze versioned encodings only with
independent client/service tests at the relevant milestone; never serialize Rust layout.

## Shared boundary rules

Trusted principal/session provenance and explicit handles establish authority;
paths, app IDs, offers and role strings supplied as data do not. Owners enforce
scope, current revisions/epochs, quotas and mandatory policy checks at use time.
Long operations expose a bounded operation identity, status/progress, cancellation
request/result and authoritative outcome. Cancellation after an effect does not
undo it. Retry/idempotency and any partial completion are explicit per operation.
M3–M6 seed these interfaces, without importing M9–M15 implementation into their exits.

## Files and desktop exchange — M9

File operations accept granted roots/objects and explicit conflict choices. Define
create/copy/move/rename/delete/trash/restore and enumerate/stat semantics over the
real file service. Reject traversal, unauthorized targets and stale references.
Cross-volume moves may complete partially; do not promise atomicity without evidence.
An interrupted/cancelled copy reports which destination content exists and whether
it is complete. Collision handling never silently overwrites unrelated content.

Open/save pickers return narrowed grants tied to the user's selected objects,
not global filesystem access. Clipboard/drag offers declare types, producer,
audience, bounds and lifetime; consumption requires a grant and current validity.
Revoke/expire stale offers and clean up abandoned transfers. Association/default-app
settings select installed app identities and declared handlers; an association
does not bypass content or execution permissions. Notifications carry bounded
content, expiry and authorized typed actions, with keyboard/semantic focus rules.

Wall-clock values, time zones and locale/keymap settings have explicit semantics
separate from monotonic deadlines. Persist settings, declare the wall-time source
and uncertainty/readiness, and test adjustments. Locale formatting cannot rename
stored files or reinterpret an operation's authority.

## Applications and ordinary updates — M10

App management resolves an immutable package graph and supported SDK/ABI before
granting launch authority. Installation/update previews describe requested rights;
additional rights are not inherited merely because an older app was installed.
Uninstall stops/retires the instance safely, removes only its package references
and explicitly chosen app-owned data, and preserves shared/unrelated documents.
App-data retention and failed/partial cleanup outcomes are visible.

Ordinary software updates use a reviewed publisher/trust lifecycle and verifier
before shared durable activation. Declare trust/key transition, expiry/rollback
and authorized recovery behavior at the implementation gate. Offline and online
delivery feed the same verifier. Content digests alone are not publisher authority.
Kernel/boot changes report reboot required; native development hot reload remains
a distinct authorized path. No app-store service or cryptographic algorithm is
selected by this draft.

## Backup, administration and lifecycle — M11

Backup selects granted document roots and portable settings into a coherent export
view with versioned inventory, integrity checks and explicit failure/omission status.
Use a separate chosen destination. Bound space, recursion and processing; exclude
credentials, secrets and privileged account/policy state by default.
Settings export uses a reviewed portable-field allowlist rather than copying
arbitrary app state or hidden authentication fields. Selected user documents
may themselves be sensitive; export requires explicit scoped authority.

Restore verifies the complete inventory and supported schema before applying a
previewed conflict/ownership mapping into an authenticated account on a fresh OS.
Reject corrupt/path-escaping/unsupported input and unauthorized owner mappings.
Interrupted restore cannot claim complete success or silently replace unrelated
current files. This is user-data/settings recovery, not credential migration,
system rollback or preserving reinstall. Report the destination's actual tested
failure domain; two files on one host do not establish independent host recovery.

Administration exposes bounded CPU/memory/disk/service observations with declared
sampling/units, and scoped app termination/service restart requests. Deny unsafe
control of core/storage/update services during active ownership; roles are policy
inputs, not unrestricted kernel access. Diagnostic exports are bounded and exclude
credential/private content unless separately authorized. Shutdown stops new changes,
resolves writes/checkpoints under deadlines and reports failed flush/force outcomes.
Session restore reauthorizes logical resources and rejects incompatible or
crash-looping apps; it never restores privilege from a serialized checkpoint.

## Networking, audio and physical qualification

M12's network service grants scoped endpoints and bounded connections; connection
settings and trust/time policy need appropriate administration authority. The
online updater stages resumable content but cannot bypass M10 verification after
disconnects or metadata errors. Network availability never implies trusted time.

M13 audio specifies stream format/rate/channels, bounded buffers, ownership and
underrun/overrun/reset outcomes. Capture requires an explicit grant and trusted
indicator; revocation has a tested stop/cleanup boundary. Assistive consumers use
authorized semantic actions and a validated speech path, not screenshots as authority.

M15 binds these contracts to one recorded physical device profile. Driver behavior,
flush/power semantics and actual recovery need hardware evidence; a virtual driver
or VM shutdown result cannot stand in for it.
