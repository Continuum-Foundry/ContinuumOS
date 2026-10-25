# P3-03 — Load static user images and bind trusted launch context

Issue type: Story. Epic: P3. Status: Done. Owner: Unassigned.
Depends on: P3-02 Done. Suggested labels: `phase-3`, `elf-loader`, `launch`.
Backlog: B11 loader/launch. Requirements: R03/R10/R14/R18. Test families: T02/T03/T11.

## User story

As the trusted launcher, I need validated static images and explicit process bindings
so executable bytes and caller-supplied identities cannot bypass launch authority.

## Implementation scope

1. Package real independently linked Rust user fixtures as immutable trusted boot
   inputs on the existing ISO path. Define image digest/size/entry/startup metadata
   and exact lifetime; this is boot-image trust, not a general package signature or
   install system. Audit the user target/code model separately from the kernel.
2. Parse static ELF with checked arithmetic: class/endian/machine/type, headers,
   file/memory sizes, offsets/alignment, segment overlap, user bounds, entry and
   permissions. Zero BSS. Reject dynamic linking, unsupported relocations/TLS and
   host dependencies explicitly rather than ignoring unresolved runtime obligations.
3. Build a staged process with owned image mappings, guarded aligned stack, bounded
   startup bytes and initial resource ceiling. Publish it only after complete
   validation; reclaim all partial state on image/admission failure.
4. Define trusted launch records distinguishing user/service principal, authority
   session, process instance/generation and permitted initial grants. The bootstrap
   launcher owns fixture bindings; ordinary child launch cannot select arbitrary
   principal/session/role or exceed delegated ceilings. Specify the later supervisor
   authorization hook now; exercise unprivileged launch through P3-09/P3-10.
5. Independently inspect loaded mappings and startup bytes against source ELF and
   digest. Reserve no privilege in user startup data merely because it contains an
   identifier; privileged authority remains in trusted kernel/broker state.

## Acceptance criteria

- [x] Real static fixture images load into separate owned spaces with correct RX/R/RW,
  zero BSS, guarded stack, entry and byte-for-byte audited contents.
- [x] Malformed/unsupported images fail before runnable publication with precise
  bounded reasons and no partial object, frame or trusted-context leak.
- [x] Trusted user A, user B and service launch records remain distinct across process
  reuse; stale identifiers and payload principal/session/role edits grant nothing.
- [x] Startup/entry/image lifetime and child authority ceilings are documented in
  draft contracts; no native Rust layout becomes an external startup ABI.
- [x] Actual ISO packaging and affected M1 metadata/boot regressions still pass;
  loaded-image validation is not reported as successful user-mode execution.

## Tests and evidence

Mutate each header/range/permission/entry class, truncated files, huge BSS, integer
overflow, overlap, WX/alias attempts, unsupported relocation and wrong digest.
Inject allocation failure during segment/stack/startup construction. Compare an
independent ELF reader with guest loaded-image records and mapping summaries.
Retain forged launch-context and stale process-generation fixtures for later syscalls.

## Exclusions and handoff

No dynamic linker, guest compiler, untrusted package admission or account database.
P3-04 receives validated staged processes and startup metadata for real ring-3
entry; P3-09/P3-10 own authorized child launch and policy enforcement. Sources:
[kernel design](../../02-architecture/kernel-and-execution.md),
[identity model](../../02-architecture/identity-and-authorization.md),
[build/release](../build-and-release.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

## Local acceptance — 2026-10-25

P3-03 passes [local Codex self-review](../../../evidence/phase-3/P3-03/2026-10-25/README.md),
not independent approval. Use the [loader runbook](static-loader-runbook.md).
Three separately linked Rust fixtures load 72 independently audited private pages,
including exact file/BSS/padding/startup/stack bytes and raw hardware tables.
All 28 kernel host tests pass; guest admission rejects 19 malformed/truncated
cases, every insufficient 0–23-page quota, wrong digest and 15 forged authority
requests, then rejects stale process generation after slot reuse and settles exactly.
The complete current M1 gate passes 33 rows/42 launches, and the separate P3-02
1000-cycle memory regression passes. Full raw artifacts remain local.

No user instruction executes. The return page deliberately contains UD2 until
P3-04 installs process_exit and IRETQ; public child launch/authorization remain
P3-09/P3-10. This accepts staged loading, not ring-3 containment, account identity,
public rights, a scheduler, general signed package admission or the M3 milestone.
