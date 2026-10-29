# P3-06 handles and retained memory objects

Experimental, unfrozen, single CPU and one owning thread per process. The new
rights-bearing fixture uses the original pinned offline Q35/TCG/OVMF optical
route, one vCPU, 2 GiB capacity and the unchanged 30000 ms deadline. No UI runs.
Use the existing development signer; no host policy deployment is needed.

```powershell
pwsh -NoProfile -File tools/Test-P3-06.ps1
python tools/Review-P3-06.py --acceptance build/m3/P3-06/acceptance-<run>/acceptance.json
```

Focused bring-up after dot-sourcing `tools/Enter-Dev.ps1`:
`cargo xtask test --suite m1 --case capabilities`. Full generated artifacts stay
under ignored build/work; the compact review bundle follows evidence retention.

## Authority and draft scalar ABI

The trusted launcher installs a self-process handle (QUERY, DUPLICATE, CLOSE,
CREATE, MAP, CONTROL) and a self-thread handle (QUERY, DUPLICATE, CLOSE, CONTROL).
The test's launch configuration conveys these numeric values before entry; it
contains no kernel pointers. Editing this user page cannot install a grant.
The startup channel field stays zero: no channel exists until P3-07. The third
task has the first task's principal/session but its own address space and grants.
Knowing that owner's IDs or handle value gives neither app authority.

The draft JSON contract generates operation/error constants. New IDs 6–12 are
object_info, handle_close, handle_duplicate, memory_create, memory_map,
memory_unmap and memory_protect. R9 revision and R8 features remain zero. All
results use RAX status/RDX scalar; errors return value zero. The exact arguments
are in `objects_boundary.arguments` in `contracts/runtime-v0.json`.

- QUERY=1, DUPLICATE=2, CLOSE=4, CREATE=8, MAP=16, READ=32, WRITE=64,
  EXECUTE=128, CONTROL=256. Object kinds are Process=1, Thread=2, Memory=3.
- object_info(handle) requires QUERY and returns `(kind << 32) | rights` in RDX.
  It reveals no object ID, principal ID or address. Duplicate requires DUPLICATE
  and a subset of that handle's rights. Close requires CLOSE. A zero-right
  duplicate is legal and is reclaimed on death, since it cannot close itself.
- memory_create(self_process, bytes, backing_rights) requires CREATE, positive
  page-multiple bytes at most 64 KiB, and READ, READ|WRITE or READ|EXECUTE.
  It returns a zero-filled memory handle with basic rights, MAP and that ceiling.
  This conservative backing ceiling is immutable; RW objects never become RX.
- memory_map(self_process, memory, request_ptr, 32) requires process MAP plus
  memory MAP and requested access rights. Copy four explicit little-endian u64s:
  base, object offset, bytes, access. Access is Read=0, Write=1, Execute=2.
  Copy the entire 32 bytes once before borrowing the registry. There is no native
  struct cast or output alias. Overflow, guard/kernel/unaligned addresses,
  conflicting maps and invalid offsets/extents fail transactionally.
- memory_unmap(self_process, exact_base) and memory_protect(self_process,
  exact_base, access) affect only mappings made through this interface. Loader
  image/stack/config mappings cannot be selected by guessing their addresses.
  Protect also checks the original mapping's retained handle/backing ceiling.
- thread_yield(self_thread) and process_exit(reason, self_process) require the
  matching type and CONTROL. Public foreign-process mutation, termination,
  process/thread creation and joins remain unsupported. No supervisor handle
  is granted to these clients. Normal exit retains the supervisor/scheduler
  ownership until reaping; closing self handles does not destroy a live task.

Type validation precedes rights validation after handle lookup: malformed,
stale and foreign tokens give BadHandle; valid wrong kinds give WrongType;
missing rights give AccessDenied. Alignment/range/flags/conflicts give
InvalidArgument, resource metadata admission gives QuotaExceeded, backing frame
failure gives OutOfMemory, and a live copy pin gives WouldBlock. Unsupported
families and the provisional memory_allocate/free operations give Unsupported
in this profile. P3-04/05 regression fixtures remain explicitly gated historical
ABIs with their previous allocator conventions; no production process launcher
exposes that test API. Unknown extra scalar arguments are ignored; R8/R9 and
the map request length retain their explicit checks.

## Identity, references and exhaustion

A handle encodes slot+1 in its low 32 bits and a nonzero boot-wide monotonic
installation nonce in its high 32 bits. Every install, including a duplicate,
gets a new nonce; no table or process reuse repeats it within that boot. After
installing u32::MAX, all further installations fail QuotaExceeded until reboot.
There is no wrap or process-death reset of this allocator. This deliberately
stronger exhaustion policy replaces the earlier per-slot draft. Object IDs have
separate internal u64 generations; exhausted vacant slots retire permanently.
The forced boundaries are host-tested through the same registry implementation.
These values are not credentials across boots and must not be checkpointed.

Each object counts handle and mapping references separately. A memory object
holds one independent backend reference. Closing the source handle does not
invalidate a duplicate; closing all handles does not unmap memory. The mapping
keeps its own object reference and immutable permission ceiling. Creator teardown
detaches surviving held backing, preserving allocation generation and content.
Orphan backing remains charged in global committed/requested/resident inventory;
each foreign mapping is charged separately to its process. Last handle AND last
mapping release the backend hold and frames. Allocation reuse is zero-filled.
Internal legacy non-object sharing retains its original shared-busy behavior.

Only trusted launch code grants immutable R/NX backing to another process;
it cannot grant READ|WRITE backing as an immutable snapshot. P3-07 owns public
atomic transfer. There is no general revoke: surviving duplicate handles and
mappings remain valid after issuer close/death. A mapping cannot recover rights
from a later stronger handle. Brokered policy/session revocation remains P3-10.

## Bounds and unsafe review obligations

The locked `objects_execution_subset` is 16 owners, 128 objects, 32 handles and
32 public mappings per owner, with 32 MiB created/mapped ceilings. Backend image,
stack and public maps share its 128-map/64-MiB commitment ceiling. The smaller
admitted subset does not claim the planned M3 capacities are implemented.
Fixed registry, table banks, task/FX state and guarded stacks are charged in full
against the existing 16-MiB fixed-runtime envelope. All metadata is precharged;
failed handle/object/backing/map admission leaves the recorded live inventory
unchanged. A single request admits at most 64 KiB. No cache is excluded.

Structural methods run with IF clear. The IRQ never borrows registry/memory;
kernel IRQs only mark pending. No borrowed user pointer, registry/memory borrow
or copy pin crosses a dispatch. A pin blocks mapping mutation and teardown;
close/unmap/removal conservatively require all registered spaces unpinned before
collecting objects. There is no spinning on another task. Removal unmaps only
admitted mappings (a page-table subset needing no frames), drops handle refs,
collects unreachable objects, then the launcher destroys the space on the
surviving supervisor root/stack. Active CR3 publication follows P3-05.

The test delivers real timer IRQs with a shared-copy pin held, attempts
unmap/protect/close/termination, checks WouldBlock and unchanged inventories,
then unpins and dispatches at syscall return. Other copies use the existing
bounded recovery instructions and effective page permission checks. No SMP,
shared-process threads or suspended kernel continuation is admitted.

Full kernel disassembly now exceeds the old 2-MiB ordinary command-log bound.
Only static LLVM kernel audits opt into a bounded 8-MiB local output reader;
ordinary commands, guest serial caps, child cleanup and VM deadlines are unchanged.
Retained excerpts still obey the 256-KiB file/evidence-bundle limits.

## Workloads and completion gate

Six three-task waves exercise issuer close/death, explicit cross-owner read,
same-user app denial, stale/forged/wrong-type/attenuated handles, non-amplifying
duplicates, handle/mapping quota failure and zero-on-reuse. Five receiver faults
exercise shared write, shared execute, kernel read, and another live task's
private read/write. Healthy owners continue in the same boot. Each wave must
restore every object/handle/backing/frame/space/table/stack/pin inventory exactly.
The forced generation boundaries, object exhaustion, physical allocation and
page-table failure rollbacks use host tests through the actual implementation.

Final acceptance also reruns ten prerequisites, complete M1 (33 rows/42 launches),
1000-cycle owned memory, staged loader, four user-boundary groups, and paired
scheduler/CPU-negative boots. The independent reviewer parses serial and actual
ELF/ISO bytes, correlates sources, checks lifecycle cleanup and rejects evidence
mutations. Review is local Codex self-review, not independent external approval.
No DMA/IRQ/device families, public thread/child supervision, policy broker,
blocking IPC, frozen SDK, guest UI/reload or desktop performance is accepted.

The [2026-10-29 local acceptance review](../../../evidence/phase-3/P3-06/2026-10-29/README.md) passes this bounded subset. Failed development runs remain recorded separately.

The new profile exercises explicit self-handle exit. Ordinary callable-return and general allocator SDK closure remain unaccepted for this profile and must be completed at P3-12; the earlier regression fixtures retain their original return/allocator conventions.
