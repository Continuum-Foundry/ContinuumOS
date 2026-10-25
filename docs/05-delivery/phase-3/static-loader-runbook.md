# P3-03 static images and trusted launch bindings

This implements staged loading only. No instruction in the user fixtures is
executed. Read [the story](03-static-elf-loader-and-launch-context.md) and
[runtime contract](../../06-contracts/runtime-v0.md). Kernel/ISO and hosted UI
remain independently buildable.

## Commands

Use PowerShell 7, the existing local signer and the documented xorriso execution
context. No enrollment, certificate or host policy change is part of these commands.
Run serially without competing acceptance jobs; keep every failed run.

```powershell
pwsh -NoProfile -File tools/Test-P3-03.ps1
python tools/Review-P3-03.py --acceptance <new-acceptance.json> --destination evidence/phase-3/P3-03/<new-run-id>
```

The test runs all ten current prerequisite checks, the complete M1 33-row/42-launch
gate and its review, then the existing owned-memory case and the new
`cargo xtask test --suite m1 --case static-loader`. Offline Q35/TCG/OVMF, one vCPU,
2 GiB capacity and the unchanged 30-second per-launch deadline apply. No retry or
deadline extension occurs inside acceptance. A subsequent run after a finding is
a distinct experiment. The reviewer independently reconstructs raw user page
tables, expected pages from ELF bytes, all BSS/padding/stack/startup bytes via
SHA-256, private backing, supervisor alias permissions, identity/reuse, failure
settlement, extracted ISO payload and launch cleanup. It mutation-tests its own
rejection of missing or altered evidence. Full artifacts remain ignored locally.

## Image and startup lifetime

`experiments/user-images/fixture.rs` is linked three times directly by the pinned
workspace Rust compiler for `x86_64-unknown-none`, small code model, static
relocation, no red zone, panic abort, optimization 1. User A, user B and service
have distinct initialized data, RX text, R constants, RW data and 8193 bytes of BSS.
LLVM headers, sections, symbols, relocations, needed libraries and disassembly
are retained locally; undefined symbols fail the build. These are new binaries,
not the Phase 0 transform entry or the P2 arena. CPU/context execution qualification
remains P3-05; disassembly alone is not execution evidence.

The host build hashes each exact ELF and emits an explicit digest/size descriptor
compiled into the experimental kernel. Its bytes join build identity. The ISO
contains three Limine modules; extracted payload hashes must match staged bytes.
The pinned loader and coherent ISO are the trust root. There is no package
signature, credential, secure boot or untrusted installation claim. Module physical
ranges remain reserved for the whole boot. Only exact module extents are exposed
to the loader through supervisor RO/NX/WB windows established before private roots
inherit kernel entries; no module is reclaimed or mapped to a user.

Admission verifies exact size and SHA-256 before parsing. The no-heap parser admits
ELF64 LE ET_EXEC x86-64, version 1, zero ELF flags, 56-byte program headers
and 64-byte section headers. Limits are 1 MiB file, 16 program headers, 128 section
headers and eight LOADs. Stripped/extended section tables are unsupported in this
revision. LOADs have nonzero memory, filesz <= memsz, checked extents, power-of-two
alignment from 4 KiB through 2 MiB, congruent offset/address and paddr == vaddr.
Only R/RX/RW are admitted. Page-rounded overlaps reject, including W/X aliases.
Entry must be inside file-backed RX bytes. Program kinds other than NULL, LOAD
and empty nonexecutable GNU_STACK metadata reject. Dynamic/interpreter/TLS,
relocations, dynamic symbols, constructors/destructors, TLS sections and nonlocal
undefined symbols reject explicitly. There is no linker, TLS setup or hidden host
runtime. File offsets may refer to identical bytes, but each LOAD gets its own
zeroed physical backing; this cannot create a shared W/X alias.

The identity admits System-V (0) and GNU (3) OSABI tags with ABI version/padding
zero. LLVM's actual fixture headers use GNU because Rust's `#[used]` emitted
SHF_GNU_RETAIN on constants/data/BSS. That section retention flag carries no
runtime obligation. The complete segment/section/symbol audit, rather than the
OSABI label, closes the admitted static runtime subset.

Image pages must fit below `0x7ffd0000`; startup at `0x7ffd0000` is R/NX and the
return page at `0x7ffd1000` is RX. The P3-02 guarded 64 KiB RW/NX user stack stays
at `0x7ffe0000`. Every image page, BSS, rounding, stack and two startup/return pages
charges the 64 MiB commitment ceiling. Admission is fallible; no global heap is
added. Stack return bytes are written before mapping, with RSP = stack end - 8
(modulo 16 = 8). Startup is exactly 64 explicit LE bytes, revision/features/handle/
clock/reserved fields zero, magic `CNTM3RT0`, length 64 and page bytes 4096.

The current synthetic return points at **UD2**, a fail-closed placeholder. P3-04
must assign and install the process_exit stub before entering any fixture. There
is no valid syscall ID yet, so P3-03 does not pretend the exit syscall works.
RDI = startup address is a handoff requirement; registers, selectors and IRETQ
are not installed in this story. User code must never be entered from this test.
Loaded bytes are owned copies independent of module lifetime. No kernel pointer,
principal, session, role or usable bootstrap handle is encoded in startup.

## Publication, failure and authority

`kernel/src/user_image.rs` is shared by host and guest. Validation precedes any
space/context allocation. Construction then creates the guarded stack, copies
file bytes into eager zero-filled backing, maps each LOAD, startup and return page.
Only after all operations succeed does the fixed 16-slot trusted launch table
publish a **staged** record. Failure destroys all partial mappings/allocations/
space/stack state before returning its bounded field/reason. There is no runnable
publication, scheduler or public process object. Generation increments at successful
publication; slots retire at u64 exhaustion. Destruction verifies generation and
reclaims memory before removing trusted context. The fixed mutation scratch,
launch table, memory backend and guarded kernel stacks fit the existing 16 MiB
storage ceiling; these are kernel/test costs, not the desktop cold-idle target.

The bootstrap launcher alone chooses `Binding`. Its private table binds synthetic
user A (principal 101/session 1001), user B (202/2002), and service (303/3003),
process slot/generation, class and initial grant/commitment ceilings. Grant masks
1/1/3 are internal permitted ceilings, **not implemented public rights or handles**.
There is no user API that chooses Binding. Startup identifiers, forged handles or
future request role strings cannot change the trusted record. The child-context
hook validates a live parent, exact principal/session/class and subsets of grants
and commitment; it derives the result from the trusted parent. It does not create
a child or prove unprivileged enforcement. P3-09/10 must authorize the calling
supervisor and authenticate the caller, independently validate image delegation,
charge aggregate quotas and issue actual handles. Boot correlation is never an
authority session. Stale process generations fail lookup, destruction and delegation.

Host tests mutate all admitted header/range/permission/entry classes, overlap and
page aliases, truncation/overflow/huge BSS, TLS/relocations and external symbols.
They inject every allocation construction step and failing intermediate-table
capacity. Guest tests load all three images at once, read every mapped page under
the actual private CR3, dump complete user table entries, check zero guards and
RO/NX supervisor backing aliases, reject 19 malformed/truncated cases and every
insufficient page quota, deny 15 identity/class/grant/ceiling edits and wrong digest,
then reuse a process slot with fresh generation and settle all inventories exactly.
These checks cannot establish ring-3 fault containment; P3-04 owns that boundary.
