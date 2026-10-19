# P1-04 acceptance review — 2026-10-19

**Done for P1-04 on the declared Q35/TCG/OVMF profile. M1 is incomplete.**
Codex performed this local self-review against all five story criteria. This is
not independent approval, arbitrary-fault recovery, security certification or a
performance result. ADRs remain Proposed.

Accepted harness: `acceptance-p1-04-1792452228601`. All 15 commands in the
[acceptance record](acceptance/acceptance-results.json) returned zero.
[The artifact review](review.json) checks 21 distinct launches, exact ISO/ELF
digests, source hashes, correlated terminal records and generated entry code.
The [review procedure](acceptance/review-procedure.ps1) and
[retention procedure](acceptance/retention-procedure.ps1) are retained.

Source base: `2f46af9370a8be0fd7d0f483327f269d0f06cab7` on `codex/p1-04-exception-diagnostics`, dirty.
Every build has a complete source-input hash inventory. The [source snapshot](source/)
retains those final input bytes, checked against the working tree during review.
The unrelated pre-existing `.gitignore` edit is excluded from the implementation commit.

| Validation | Result |
| --- | --- |
| Host checks | 12 xtask + 7 kernel-library tests; formatting passes |
| Phase 0 | Clean probe links, provenance, source path with spaces, doctor overrides and bounded host cleanup pass |
| Dependency probes | Five libraries compile; allocation-dependent links retain their expected blockers |
| M1 prerequisites | Pinned inputs and rejection tests pass outside xorriso's documented sandbox restriction |
| Ordinary boots | Seven successful boots, each reaching exception setup then completion; includes a real module and serial-only fixture |
| Metadata rejections | Eight exact field/reason cases and exit 35; no metadata-success or exception-ready marker |
| Negative ISO cases | Two expected failures, pre-entry halt and missing configured executable; neither counts as a boot |
| New fatal cases | Real #UD, non-present read #PF, Rust panic and #UD with the logger held; exact diagnostic/exit 37, no boot-complete marker |
| Process lifecycle | All 21 fresh configured launches have distinct sessions and confirmed reaping; test exits are not physical shutdown |

## Story criteria review

| Criterion | Evidence and conclusion |
| --- | --- |
| Normal boot reaches exception readiness | [Ordinary observation](run/result.json) and [metadata matrix](boot-metadata/result.json) preserve prior identity/resource checkpoints; `m1.exceptions_ready` precedes completion |
| Distinct real exception context | [Invalid opcode](exception-invalid-opcode/result.json) reports vector 6; [page fault](exception-page-fault/result.json) reports vector 14, error 0 and CR2 `0x400000000000`. RIP matches exact ELF fault-site symbols, frame/RSP are inside the static owned runtime stack |
| Panic and emergency output before allocators | [Real Rust panic](panic/result.json) reports the symbolized reporting context and source line; [held-logger #UD](exception-emergency/result.json) uses `emergency=true` and exits without waiting for the logger |
| Reset/triple fault cannot pass | Exact identity, sequence, terminal event/context and process exit are mandatory; observer tests cover wrong exit, missing marker, timeout, log limits and cleanup |
| Assembly/unsafe review, no FP/SIMD | All 32 entry stubs and common alignment/CR2/DF logic are checked in retained LLVM disassembly. Build audits reject FP/SIMD, undefined symbols and image extents omitting BSS. The [CPU/unsafe review](../../../../docs/05-delivery/phase-1/exception-diagnostics-runbook.md) documents assumptions |

Normal build ID: `ea26a37ec4e55ba1a49b588d89629fcee89f228766631219620b3e371a74197a`.
Fault-case build ID: `8292d32448480d7b1b099fc1a93f26a71ec3252aa9881ef592f1bb224b643cf6`; all four named fault cases use identical
kernel bytes with separate configured ISOs/sessions. All full ELF, ISO, firmware,
configuration and input digests are in the per-run manifests and review.

## Failed development checks

- The first build rejected a named label in Rust inline assembly. The panic
  reporting context helper now lives in global assembly; no lint suppression or
  unstable ABI was introduced. See the retained
  [compiler failure](failed-experiments/build-1792452128073765700-26988/kernel-build.stderr.log).
- A host observer fixture still emitted version 2 after the protocol bump. The
  strict parser correctly rejected it; the fixture was updated and all 12 host
  tests subsequently passed. Its [run result](failed-experiments/marker-wrong-exit-1792452097885445800-3960/run-result.json)
  is retained. This was a host-test fixture failure, not a guest boot result.
- Initial sandbox doctor output reported xorriso's MSYS2 named-object access
  restriction. The accepted prerequisite run used per-command sandbox escalation;
  no global tool, system setting or Windows feature changed.

## Limits and next gate

The profile remains one vCPU, 2 GiB capacity, `pc-q35-11.1`, `qemu64-v1,+apic`,
TCG and pinned OVMF. There is no network, host share, writable user disk, snapshot
restore or direct-kernel shortcut. Every boot uses the packaged optical ISO.
Host session IDs are correlation labels, not credentials or sole reset detectors.

GDT/TSS/IDT and runtime/double-fault/NMI stacks are kernel-owned static storage
covered by the image reservation. Existing loader page tables provide mappings;
all loader-reclaimable memory stays retained and zero bytes are reclaimed. ACPI
remains an undereferenced locator. Interrupts stay disabled; no timer, allocator,
owned page tables, process isolation, rendering or live update is demonstrated.

Static stacks are unguarded. **Actual double-fault/IST entry and guard fault
integration remain mandatory P1-05 work.** A held logging gate does not prove
arbitrary stack corruption, NMI timing, byte-interleaved output or hardware recovery.
Panic RIP identifies the reporting context, not an unwound caller. Output failure,
malformed context and repeated reporting faults fail acceptance. The 256 MiB
desktop cold-idle target is unchanged and unmeasured.

[The evidence index](evidence-index.json) records raw/retained artifact hashes;
[SHA256SUMS.json](SHA256SUMS.json) freezes the retained bundle. Serial and source
snapshots are verbatim; other diagnostic text has personal path prefixes sanitized.
ISO/ELF binaries, firmware-variable copies and caches stay in ignored `build/m1/`.
Historical evidence is unchanged. Everything remains local.

Next: P1-05 memory ownership, preserving exception mappings and completing its
guard/double-fault integration gate before later M1 acceptance.
