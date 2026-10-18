# Storage, files and persistent data

Status: proposed. R07, R11, R12, R16. Decision: [ADR-0009](../04-decisions/0009-persistent-generations.md).
See [boot recovery](boot-recovery.md) for boot attempts, health marks, known-good
selection and the later A/B alternative. Those mechanisms do not replace the
in-session live-update transaction or rewind user documents.

## Separate three lifecycles

1. **System content:** immutable packages addressed by digest and selected by a
   generation manifest. The ISO is one base generation.
2. **User data:** documents and settings, with their own schemas and save semantics.
3. **Operational metadata:** package staging, activation journal, checkpoints,
   health/rollback decisions and diagnostics.

Rolling back system packages must not restore an older copy of the user's documents
without an explicit data-recovery operation. Garbage collection must preserve any
generation/checkpoint still needed for recovery.

## Staging storage implementation

M1–M4 can use a read-only boot archive plus bounded in-memory files. Clearly label
changes ephemeral. M5 can prove in-memory replacement without claiming power-loss
durability. M6 adds a writable named VM disk through a block service.

Proposed first persistent mechanism: a small append-only, checksummed record store
with an immutable blob area and dual or alternating generation anchors. It supports
packages, checkpoints and activation records, not a claim of a complete POSIX
filesystem. This choice creates real storage engineering work and must pass fault
tests before storing valuable data.

Each record has format version, length, sequence, payload digest/checksum and commit
framing. A torn or uncommitted tail is ignored. Do not overwrite the last valid
anchor before the new referenced content is durable. The exact sector sizes,
maximum records, checksum algorithm and recovery scan limits are frozen during the
storage spike. Avoid relying on one-sector atomicity without proving that assumption.

An established filesystem/library is an alternative if its exact language, platform
and semantics fit. littlefs is a power-failure-oriented reference, not an automatic
desktop filesystem or a selected pure-Rust dependency.
[S56](../01-research/sources.md#s56)

## Durable write contract

Define `write`, `flush`, `commit` and `acknowledged durable` distinctly through the
file/block/device stack. Hash verification detects corruption; it does not force
data to storage. A filesystem/library using a flush API cannot be more durable than
the device implementation beneath it. SQLite's commit analysis is a useful warning
about these assumptions. [S55](../01-research/sources.md#s55)

Checkpoint and activation ordering: write immutable package/checkpoint → flush →
append prepare record → flush → publish activation record/anchor → flush → acknowledge
durable success. Exact record sequencing must match the update recovery table.
After restart, choose the latest fully valid activation with available dependencies;
never infer success solely from the presence of a partially copied package.

## File service API

Expose open-by-directory capability, read/write with offsets, metadata, enumeration,
flush, close and atomic replace only when their semantics exist. Directory handles
bound traversal; reject traversal outside granted roots. Use UTF-8 names but preserve
exact stored names initially; normalization/case folding must be an explicit policy.
Proposed default is case-sensitive and no silent normalization.

Application save: write new content, complete required flush, then replace the
document's authoritative reference. Provide meaningful unsaved/error states. An
autosave checkpoint is separate from an explicit user save. Keep multiple recent
checkpoint generations bounded by quota and retention policy.

## Recovery and testing

Inject interruption after every record write/flush/anchor step. Restart from the
same disk image and verify last acknowledged durable state, absence of references
to missing blobs, and preservation of unrelated user files. Use a faulting block
model for torn/reordered writes in addition to QEMU process termination; killing
a VM alone does not simulate every storage failure.

Encryption, disk quotas by user, rich attributes, remote filesystems, snapshots and
full desktop filesystem semantics are later decisions. A physical-disk installer
requires a separate target-selection and backup/recovery design.
