# P3-02 bounded allocation and owned address spaces

This implements the internal memory backend before a loader, public handles or
ring-3 execution. Read [the story](02-bounded-allocation-and-user-address-spaces.md),
[runtime contract](../../06-contracts/runtime-v0.md) and
[ADR-0018](../../04-decisions/0018-bounded-owned-memory.md).

## Run and review

Use PowerShell 7 and the already enrolled local signer, with the documented
certificate-store/xorriso execution context. No enrollment or policy change is
part of these commands. Run sequences serially, with no competing acceptance job.

```powershell
pwsh -NoProfile -File tools/Test-P3-02.ps1
# The command prints the three fresh artifact paths and a local acceptance JSON.
python tools/Review-P3-02.py --acceptance <new-acceptance.json> --destination evidence/phase-3/P3-02/<new-run-id>
```

The test reruns all ten prerequisite checks, the complete 33-row/42-launch M1
gate and its artifact/serial review, then `cargo xtask test --suite m1 --case
owned-memory`. The new case retains M1's offline Q35/TCG/OVMF one-vCPU/2-GiB
capacity and fixed 30-second deadline. It uses the M1 CPU pin, `qemu64-v1`, rather
than pretending that P3-01's proposed `+apic` CPU has been qualified for user FP.
The lock's existing APIC timer still passes; P3-05 owns the emitted user CPU gate.
No retry or deadline extension is performed. Each failed run remains local.

The independent Python reviewer hashes the actual launched ELF/ISO, parses ELF
load segments, reconstructs all dumped user page tables from raw entry words,
checks representative walks, supervisor RO/NX physical aliases, guarded stacks,
boot/session/sequence identity, exact settlement and QEMU cleanup. It also rejects
mutated entries, missing records, bad permissions and unsatisfied settlement.
Full logs, binaries, source patches and disassembly stay ignored under `build/`.
The retained bundle follows [evidence retention](../evidence-retention.md).

## Bounded storage and allocation

`kernel/src/owned_memory.rs` is the no-std backend shared by host tests and the
guest. `runtime_memory.rs` supplies physical byte access and CR3 publication;
`paging.rs` keeps the sealed M1 frame/reservation authority. No Rust global heap,
P2 arena, new dependency or host service is added.

Sixteen spaces, 256 allocation slots globally, 128 mappings per space and 32
copy pins are fixed metadata. Each space has two image-owned 64-page table banks.
The full Rust table objects (including alignment/tail padding), metadata, and
16 guarded 64-KiB kernel stack slots are precharged, even at zero live spaces.
The guest records actual `size_of` totals and rejects a build exceeding the
16-MiB kernel dynamic-storage ceiling. This fixed cost is separate from M1's
2304-table arena and frame bitmaps. It is not desktop cold-idle acceptance.

User runs are physically contiguous eager 4-KiB pages; first-fit fragmentation
can return `OutOfMemory` even when aggregate free RAM exists. Alignment accepts
powers of two from 1 through 2 MiB, rounds to at least a page, and never silently
weakens the requested alignment. Nonzero sizes round with checked arithmetic.
Zero-size allocation returns `None`; realloc-to-zero frees after validating the
owner/token/alignment. Free requires an exact live generation and owner. Generation
wrap retires the slot. Realloc precharges the full second run, copies only
`min(old_requested,new_requested)`, leaves all other bytes zero, and frees the old
run only after success. A temporary quota miss preserves the old allocation.

Each owner charges requested bytes and full page commitment (including the user
stack), up to 64 MiB. Foreign mappings additionally charge their full extent to
the receiving process; physical resident bytes count backing once. Same-owner
aliases consume mapping metadata but reuse the already charged backing. The
32-MiB eventual userspace heap policy remains a P3-04/SDK caller obligation.
Table/mapping/allocation exhaustion is a typed internal `quota` error; physical
exhaustion is `frames/exhausted`. Public error IDs do not exist yet. Kernel
invariant failures use M1's allocation-free fatal path; ordinary admission does
not panic or retry. Storage for unused banks remains reserved, not a leak.

## Tables, aliases and transaction boundary

User addresses are page-aligned extents inside `[0x10000,0x80000000)` with checked
end/offset arithmetic. Null, overflow, noncanonical, boundary-crossing, overlapping
and unaligned mappings reject. The fixed user stack is 64 KiB at `0x7ffe0000`,
with both guard holes reserved against every map. Each generation binds and clears
its own image-owned supervisor kernel stack; all 32 end pages remain unmapped.
The M1 runtime and two IST stacks retain their original six guards.

Create/map/unmap/protect/destroy are staged internal operations. A mutation
rebuilds the candidate user tree in the inactive bank from authoritative mapping
metadata, with U/S at every user level and W^X/NX leaves. Failure discards that
bank and preserves the active root, references and accounting. Publication has
no remaining fallible operation; active spaces reload CR3 before old translations
or backing can be reused. PCID/global user mappings/SMP are unsupported. Successful
destruction switches to the surviving M1 root before releasing a reusable slot.
Inactive tables can retain stale bytes but are unreachable and reset before reuse.

High-half roots copy only the trusted supervisor M1 entries. Their descendants
are shared kernel mappings; the user builder cannot mutate them. M1 reservations
for loader, kernel, firmware, ACPI, modules and devices remain unchanged. Allocated
data runs lose their writable direct-map aliases before user mapping. Staging
writes temporarily enable supervisor RW/NX only while the run has no mapping
references, then restore RO/NX with INVLPG. Full-allocation writer/executor counts
conservatively reject W/X aliases, including across spaces or disjoint subranges.
Executable pages therefore have no writable supervisor data alias. Released data
regains its ordinary RAM RW/NX alias only after all mappings have gone and CR3
publication has flushed old user translations; reuse zeroes the whole run.

## Pins, shared backing and teardown

An internal non-copyable generation pin checks the complete span, including
cross-mapping boundaries and write permission. Zero-length copies ignore the
address and touch no bytes. While pinned, address-space mutations, free and
destruction return `busy`. The caller must consume the pin with `unpin`; leaking
a pin retains authority/storage and cannot permit reuse. No pointer survives a
wait. IF is clear throughout the current boot experiment, and IRQ/NMI code cannot
mutate mappings. P3-04 must add bounded preemption masking and copy-fault recovery;
P3-06 must add public rights and race tests before exposing these primitives.

Trusted internal maps can reference another owner's backing. Mapping references
prevent free/realloc; the owner cannot die while foreign mappings survive and
gets `shared_busy`. Removing an owner mapping does not invalidate a foreign pin.
This conservative Busy policy is explicit; P3-06 must move backing to independent
memory-object ownership before public process death semantics can replace it.
Thread teardown must occur on a surviving stack; these kernel stacks are not
executed by P3-02. No scheduler or public process lifecycle is claimed.

The guest exercises private roots, actual CR3 switching, an active RW-to-RX
transition, cross-space alias denial, shared teardown denial, pins, every failing
stack-allocation quota from 0 through 15 pages, and 1000 poisoned create/map/
unmap/destroy cycles. Host tests additionally inject every page-zero failure,
each intermediate-table capacity failure, metadata exhaustion, invalid ranges,
realloc failures, stale/double-free/owner errors and generation retirement.
Ring-3 memory/fault enforcement remains P3-04/P3-06; the ELF loader remains P3-03.
