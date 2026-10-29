# P3-04 user entry, syscalls and process-local failure

P3-04 [passes local Codex self-review](../../../evidence/phase-3/P3-04/2026-10-26/README.md). This slice is
single CPU, one executing process at a time and integer-only. It does not accept
preemption, public memory handles, general supervision, accounts or guest UI.

## Commands and evidence

Use the existing owner signer and pinned offline tools, serially:

```powershell
pwsh -NoProfile -File tools/Test-P3-04.ps1
python tools/Review-P3-04.py --acceptance <new-acceptance.json> --destination evidence/phase-3/P3-04/<new-run-id>
```

The runner sets process-local `RUST_TEST_THREADS=1` so independent host tests run serially; child/VM deadlines are unchanged. It rechecks all ten prerequisites including Phase 0, runs/reviews the full
M1 33-row/42-launch gate, then owned-memory, staged-loader and four user-boundary
launches. Each launch retains the locked 30000-ms deadline, one vCPU, offline
Q35/TCG/OVMF and 2 GiB capacity. No automatic retry or deadline extension. Failed
development runs are findings, never acceptance. Keep full output under ignored
build/work; retain only the compact reviewed bundle and its actual checksums.

## Experimental operation contract

`contracts/runtime-v0.json` generates the compiler-local integer constants for
kernel, independently linked fixtures and the exit trampoline on every build.
These IDs are experimental, not P3-12 ABI freeze. SYSCALL takes RAX operation,
RDI/RSI arguments, R9 revision (zero) and R8 reserved features (zero). RDX/R10
are currently unused arguments. Unsupported number/features return Unsupported;
nonzero revision returns IncompatibleProtocol. Return RAX is zero or the JSON
positive error ID. RDX is the declared value, otherwise zero; BufferTooSmall
returns required capacity. RCX/R11/RAX/RDX/memory are clobbered; all other integer
registers and user DF are preserved. No native structs/enums cross this boundary.

| ID / operation | Parameters and result |
| --- | --- |
| 0 abi_query | RDI output, RSI capacity; writes exactly 32 explicit LE bytes: revision u64, page bytes u64, trusted diagnostic principal u64, authority-session u64. Result 32. Capacity below 32 writes nothing and returns BufferTooSmall/value 32. |
| 1 diagnostic_log | RDI bytes, RSI length <=256. Only ASCII letters/digits/hyphen are admitted as a bounded synthetic diagnostic token. Zero length touches nothing, ignoring address. Result accepted length. Kernel adds trusted attribution and explicit offset/total fragments of at most 96 bytes, keeping every record within the 512-byte diagnostic limit. |
| 2 memory_allocate | RDI bytes, RSI power-of-two alignment 1..2 MiB. Zero size returns zero. Round eager RW/NX commitment up to pages. Result private base. Overflow/invalid alignment rejects; aggregate heap ceiling 32 MiB and allocation metadata ceilings return QuotaExceeded; physical exhaustion returns OutOfMemory. |
| 3 memory_free | RDI exact live base from this process's private allocation records; zero is a no-op. Interior, fabricated and double frees return InvalidArgument. Unmap/reload CR3 precedes physical release. |
| 4 thread_yield | Same process resumes; this experiment waits for one real timer IRQ on its owned kernel stack with no mapping borrow/pin held. No scheduler/fairness result. |
| 5 process_exit | RDI bounded numeric reason: fixtures use 0 normal, 1 panic, 2 allocation failure. No return. Supervisor restores the surviving kernel continuation before teardown. |

Allocation uses 32 fixed private run slots, each with a 32 MiB virtual stride,
starting at 0x10000000. Only the current process's trusted run record authorizes
free; a matching address in another address space grants nothing. This provisional
API maps only newly owned RW/NX memory. P3-06 replaces it with rights-bearing
memory objects and mapping lifetime contracts. No arbitrary map/protect is exposed.

## Entry and unsafe review obligations

`kernel/src/user-entry.S` is the sole user transition assembly. `user_enter` saves
the six SysV callee-saved registers and a surviving continuation RSP, records the
kernel CR3, switches to the owned user root and constructs a five-slot IRETQ frame.
RSP modulo 16 is 8 at entry; only RDI points at the non-authoritative startup.
All other GPRs, FS/GS bases and data selectors are zero. User CS=0x33, SS=0x3b,
RFLAGS=0x202, IF set, DF/IOPL clear. User descriptors follow the existing two-slot
TSS without moving M1's kernel descriptors. TSS RSP0 selects the current owned
64 KiB guarded kernel stack; double-fault/NMI IST stacks remain separate.

SYSCALL selects CS 8/SS 16 via STAR. LSTAR points at supervisor text. FMASK clears
IF/DF/TF/AC/NT before any Rust code or user-stack access. RIP-relative supervisor
storage captures user RSP without touching it, then installs the trusted stack.
Five return slots and fifteen GPR slots total 160 bytes; a 16-aligned stack top
therefore supplies SysV alignment before CALL. Rust's Frame offsets have compile
assertions. The kernel never uses SYSRET or accepts a user-provided continuation.

Return validation checks the restricted user IP/SP range, exact selectors, IF,
bit 1 and the arithmetic/DF flag allowlist, rejecting privileged/reserved flags.
Every level of the current hardware tree must allow user execution at IP and a
writable user byte at SP-1. Validation runs again after mapping operations and
immediately before IRETQ, with IRQs/preemption masked. There is no mutable mapping
access between validation and return. Bad state terminates on the trusted kernel
stack before IRETQ; no IRET fault is credited as containment. Noncanonical IP and
IOPL tests explicitly mutate a kernel-built frame in the test feature, since a
normal lower-range SYSCALL cannot manufacture a noncanonical saved RCX. Hostile
noncanonical/unmapped RSP and AC are supplied by actual user instructions.

The fatal exception wrapper now preserves all fifteen GPRs before Rust, keeping
the original normalized vector/error/RIP/CS/RFLAGS/RSP/SS diagnostic frame. Only
precisely authorized copy faults return through it. User synchronous faults with
exact user selectors and a live trusted current process are contained; kernel
faults, NMI, double fault and machine check retain fatal behavior. `user_leave`
clears IF/DF, restores kernel CR3 and the saved surviving RSP/callee registers,
then returns to its one waiting supervisor call. No Rust references, mapping pins
or structural borrows survive user entry. The per-process stack is reclaimed only
after that return. This continuation is not a general scheduler or user unwind.

## Copy and allocation lifetime review

Copies use a maximum 256-byte bounded kernel buffer. Length arithmetic, owned
mapping coverage, write rights and effective P/U/W permissions at all four table
levels are checked for the complete span before touching user bytes. A noncopyable
generation-checked pin rejects map/protect/free/destroy until explicit unpin. IRQs
are masked throughout validation/copy/unpin; no scheduler or second CPU exists.
IRQ/NMI handlers cannot mutate mappings. No pointer is retained across a wait;
there are no descriptor/input-output alias pairs in this subset.

The scalar assembly copy has exactly two fault-recovery sites. A kernel #PF is
recoverable only at one of those IPs while a bounded copy is active and CR2 lies
within that exact user span. All other kernel faults remain fatal. Validation
failure changes no output. An unexpected hardware fault can have copied a prefix;
the operation returns InvalidArgument and the entire output must be discarded.
No length or successful partial result is returned. An independent deliberate
unmapped read and write exercise both recovery sites separately from public
prevalidation denials. Real preemptive mapping races remain P3-05/P3-06 work.

The no-std user runtime implements a reclaiming GlobalAlloc over these page runs,
including allocate-first realloc, preserved old allocation on failure and zero
growth/reuse. Its panic/OOM paths emit a fixed bounded token then process_exit;
no formatting allocation, unwinding, terminal spin or VM exit. Runtime intrinsics
are scalar and independently linked. CR0.EM is cleared, CR0.MP/TS and CR4.OSFXSR/OSXMMEXCPT are set after feature checks. CR0.TS prevents accidental FP/SIMD execution;
the single explicit forbidden PXOR fixture must #NM. All other emitted user code
and all kernel code must pass the integer-only codegen audit. P3-05 owns FP state.

## Required guest review

Review four bounded guest launches, each with nine ordered process runs: healthy
first, four offenders each immediately followed by a new healthy process. The
combined matrix has 36 runs, twenty healthy and sixteen offenders. Cases cover null/end/overflow/kernel/foreign/
RO/mixed-page buffers, real crossing copies, fabricated frees/identity bytes,
bad operation/revision/features, register/DF/IF sentinels, nested kernel and actual
user timer IRQs, page/stack-guard/UD2/CLI/port/RO/FP-trap/IDT faults, panic, quota
and injected physical-allocator exhaustion, hostile RSP/AC and injected bad return IP/IOPL. Require exact
outcomes, no kernel-fatal event, one boot/build/session per launch, monotonic delivered ticks,
and complete inventory/frame restoration after each run. Capacity and delivered
ticks are not desktop idle usage, elapsed deadline accuracy or performance results.

The assembly, exception and copy reviews are local Codex self-review, not external
approval or a security guarantee. Sources: [Intel manuals](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html),
[runtime draft](../../06-contracts/runtime-v0.md), [ADR-0020](../../04-decisions/0020-user-entry-and-fault-boundary.md).

Case 16 is a trusted supervisor fault injection: while the physical allocator is empty, constrain it to both exact staged image plans, the two fixture/control pages and 256 additional pages. One 1 MiB allocation succeeds; the next returns the real allocator Exhausted-to-OutOfMemory path. Restore the unlimited test quota after reap has returned the physical inventory to empty, before the healthy replacement. This does not exhaust all 2 GiB of guest RAM and is not a natural whole-VM OOM claim.

Image staging uses a checked `Store::write_bytes` operation. The guest adapter admits only an exclusive unmapped allocation, makes one physical page’s supervisor alias RW/NX, copies that page’s bounded portion, restores RO/NX and invalidates before moving to the next page. It never writes a published/executable allocation. Default host stores retain scalar semantics. This removes per-byte alias transitions without changing admission, pins, lifetime or rollback; the staged-loader independent page/hash/alias audit remains required. Static GOT entries are placed before final NOLOAD BSS in the user linker script.

## Shared syscall path and fixture policy

`user_entry::dispatch` checks the entry invariant, invokes one explicit entry
seam, validates return state, decodes and handles the operation, then validates
again before the return hook/assembly. `user_fixture.rs` holds the registered
flavor hooks for frame injection, FP checks, operation overrides and return
scheduling. Registration occurs before user execution on the single CPU.
The hook table is copied by value; no structural borrow or pin crosses a switch.
The return hook may capture an already validated frame or clear TS; it cannot
change mappings or the return frame. SYSCALL itself does not switch RSP.

Cases 12 and 13 are kernel-side negative tests in `inject_bad_return`, selected
by the trusted supervisor case. The supervisor separately stages the guest case
word at 0x70000000; the executable reads it and calls yield. No public operation
selects IP, CS, SS or flags. Both injections retain `m1.user_bad_return` and result
0x200. The 160-byte frame and IP offset assertion are unchanged.

The boundary fixture keeps its integer-only TS/#NM behavior and nested-IRQ yield.
The scheduler flavor registers quota/yield/exit policy, eager FP checks and
masked-body accounting. The capability flavor checks rights before that policy.
Scheduler returns, including quota denials and yields, pass the same final
return validator. Event names, stages, results, fixtures and feature dependencies
are unchanged; no xtask contract update is required.
