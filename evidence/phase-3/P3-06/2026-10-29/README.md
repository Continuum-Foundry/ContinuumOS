# P3-06 - Handle rights and retained memory objects pass locally

**P3-06 is Done under local Codex self-review for its declared experimental subset.**
This is not external approval or the M3 exit. Read the [criterion/unsafe review](review.json)
and [runbook](../../../../docs/05-delivery/phase-3/handles-memory-runbook.md).

One fresh offline optical boot executes 18 tasks in 6 waves.
Two apps of the same user receive separate grants; only an explicit cross-owner
immutable read succeeds. Forged/stale/foreign/wrong-type handles, amplification,
missing query/close/control rights, malformed map metadata, permission/range
escalation and full handle/mapping tables receive exact errors. The reviewer
independently replays 402 syscall records and 91
unique handle installations against the recorded reference inventories.

Creator death and source/duplicate close preserve surviving mappings. Orphan
backing remains charged until its last legal reference disappears. Six real
pin-held IRQ interleavings deny unmap/protect/close/removal before balanced unpin
and dispatch. Five actual CPL3 page faults reject shared write, shared execute,
kernel read, and another live task's private read/write. Healthy tasks continue
in the same boot. Every wave exactly restores object/handle/backing/frame/space/
table/stack/pin inventories. Newly allocated backing is zeroed after dirty-object
release; the current 1000-cycle backend regression separately verifies reuse.

## Exact artifacts

Final object run: `${WORKSPACE}\build/m1\capabilities-1793236560658965000-61740`.
Source base: `0275ae6ba88dbb4c3f33e94057400df4c08af70e` plus [dirty input hashes](source-provenance.json).
The tested snapshot includes the owner's three pre-existing Windows signing-policy
edits; these are outside the P3-06 staged change. No host policy was deployed.
Full binaries, ISOs, source patches/copies and disassembly remain local under build/work.

Build ID: `9011762bc0c21436219b4f79420fefd1976b6bf25835e172e07888c29c7ee907`.
Kernel SHA-256: `00866639f0e14af56145e1613e071431bbfecf0faf477ee2d05dba1df5fb3351`.
Exact launched ISO SHA-256: `183e5685fe7303ecd37a0d8c7273b8dfda5eb0de5a84e91483ba3e07c672e44d`.
Raw serial SHA-256: `0e0d1d82180effae68b3b995b752de3da2cea09af4c5ba306d8db2008e684de7`.

Pinned Q35/TCG/OVMF, one vCPU, 2 GiB capacity, unchanged 30000-ms deadline,
no NIC/shared host filesystem, fresh firmware variables and reaped children.
One kernel entry and exact build/session correlation are independently checked.
The conservative fixed runtime envelope is 9993752 bytes, including
precharged registry/table/task/stack capacity. This is not whole-guest idle memory.

## Gate and limits

All ten final-source prerequisites and 39 kernel host tests pass. Complete M1
passes 33 rows/42 launches, including two clean optical builds with matching
kernel bytes and only the permitted ISO session-field difference. Current
1000-cycle memory, staged-loader, four user-boundary (36 process runs), paired
scheduler (60 tasks) and real missing-SSE2 CPU-negative regressions pass raw audits.
The combined reviewer rejects 123 evidence mutations.
[Selected failed/interrupted development findings](development-findings.json)
remain recorded separately; they are not accepted runs. Full generated artifacts
remain local; this compact bundle follows the repository retention policy.

The profile admits 16 owners, 128 objects, 32 handles and 32 public mappings per
owner, one owning thread, and at most 64 KiB per memory-create call. Initial self
grants and immutable R/NX cross-owner grants come only from trusted admission.
Object query returns scalar kind/rights, with no object or kernel pointer.
Each installation consumes a boot-wide nonzero u32 nonce; exhaustion rejects
new installations until reboot. Object u64 exhaustion retires its slot. Host
tests force these boundaries and physical/object/table admission rollback.

Backing/mapping ceilings are immutable. RW backing cannot become executable.
The rights-bearing profile rejects provisional pointer allocation calls; old
P3-04/05 ABIs remain gated regression fixtures. Its clients exit with an explicit
self-process control handle. Ordinary callable-return and general allocator SDK
closure are not accepted for the new profile; P3-12 must close that interface.
No user pointer, copy pin or Rust borrow crosses dispatch. Static kernel LLVM
output alone has a documented 8-MiB local audit cap; guest limits and evidence
budgets are unchanged.

Close does not revoke duplicates or existing mappings. No universal revocation,
public transfer, blocking IPC, general child supervision, SMP/shared-process
threads, device/IRQ/DMA grant, frozen SDK, guest UI/reload or desktop performance
acceptance follows. P3-05's masked-SSE/emulator debug limits remain. ADRs remain
Proposed. P3-07 is next.
