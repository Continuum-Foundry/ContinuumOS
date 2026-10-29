# Agent entry point

This workspace contains research/design, Phase 0 Windows build/link probes and
the minimal original Rust kernel ISO with checked metadata, identity, fatal
diagnostics, P1-05 memory ownership and P1-06 timer/uptime. P1-07's complete local
acceptance passes; see `evidence/phase-1/P1-07/2026-10-20-acceptance/README.md`.
Check the story's review before continuing. M1's experimental VM baseline and M2's declared hosted UI subset pass local acceptance; see
`evidence/phase-2/P2-13/2026-10-24/README.md`. Guest UI/reload and efficiency targets remain open;
this is not a usable OS. See
`docs/05-delivery/phase-1/interrupts-timekeeping-runbook.md` and retained evidence.
The user approved the reconciled revision-2 roadmap. See
`docs/00-overview/reconciled-plan.md`; individual library integrations still need validation.
P3-01's prerequisite/contract gate passes local self-review; see
`evidence/phase-3/P3-01/2026-10-25/README.md`. P3-02's owned-memory backend passes
local self-review; see `evidence/phase-3/P3-02/2026-10-25/README.md`. P3-03's staged
static-image loader and trusted launch bindings pass local self-review; see
`evidence/phase-3/P3-03/2026-10-25/README.md`. P3-04 user entry, checked syscalls
and process-local fault containment pass local self-review; see
`evidence/phase-3/P3-04/2026-10-26/README.md`. P3-05 preemption and eager
x87/SSE context pass local self-review; see `evidence/phase-3/P3-05/2026-10-27/README.md`.
Unmasked SSE exceptions, native x87 debug metadata and shared-process threads
remain unsupported/unverified as documented. P3-06 handle rights and retained memory-object lifetimes pass local self-review;
see `evidence/phase-3/P3-06/2026-10-29/README.md`. Next is P3-07; blocking IPC remains open.

Read `docs/README.md`, `docs/00-overview/requirements.md`, and
`docs/05-delivery/agent-handoff.md` before implementing anything. Claude and other
coding agents should use the same documents. `CLAUDE.md` points to this entry point.

## Development safeguards

- When instructed to work on `main`, inspect the working tree, switch safely and
  run `git pull --ff-only` before assuming it is current. Preserve unrelated edits;
  do not discard changes or force a divergent branch to match the remote.
- Keep test policy in the registered hooks in `kernel/src/user_fixture.rs`.
  Syscall operation arms share dispatch and return validation; switch capture and
  finish use the fixture-installed observer, never a hardcoded fixture address.
  Public operations must not select return IP, selectors or privileged flags.
- For shared boundary/scheduler changes, validate ordinary, `user-boundary-test`,
  `scheduler-test` and `capability-test` builds separately. An all-features build
  does not prove narrower subsets compile. Prefer early per-flavor compile checks
  where tooling supports them, then run the required guest regressions. The
  current xtask case commands combine building, packaging and guest execution;
  do not claim a compile-only matrix command exists.
- Run acceptance serially in the approved signer/xorriso execution context.
  Advise the owner to keep the host awake during long runs; do not change power or
  security settings. A host preflight timeout and a guest deadline failure are
  distinct failures. Preserve both logs, keep deadlines unchanged and never count
  an interrupted run as acceptance. Do not infer a root cause from a timeout alone;
  per-subprobe timestamps are a diagnostic improvement, not a proven fix.
- For an owner-exempted push, make the exact payload, remote destination, target
  branch and any one-time guard bypass explicit in the approved scope and record
  it in `EXPORT_CONTROL.md`. Use existing explicit authorization without asking
  again; clarify only missing scope required to proceed. An exemption is not a
  standing waiver. Keep unrelated edits and local artifacts out of the upload,
  preserve the pre-commit check and leave the push guard installed afterward.

Before retaining new evidence, follow `docs/05-delivery/evidence-retention.md`.
Keep full generated logs/source copies local; stage a compact reviewed bundle and
run `node audit/check-repository-hygiene.cjs`. Historical logs/source copies are
local-only after the owner's cleanup; see `evidence/README.md` before verifying them.

Preserve the user's central intent: an original desktop OS, modern languages,
excellent and efficient UI, live development without VM reboot for supported
changes, and a separately buildable bootable ISO. Do not replace the project with
a Linux distribution, a web desktop, or an existing kernel.

Architecture decisions are proposed engineering defaults until their documented
experiments pass. They are not claims of implementation or user approval. Work
through the milestone gates; update the relevant specification and ADR when
evidence changes a decision. Keep source-backed findings, proposals, and measured
results distinguishable. Never claim a VM boot, safe update, security guarantee,
or performance result without its evidence.

No guest compiler, GPU stack, third-party crate port, Linux application
compatibility, or universal kernel hot reload may be assumed to exist. First-party
kernel code should be Rust with narrowly documented architecture-specific assembly.
Use explicit protocol schemas across replaceable boundaries, not Rust memory layout.
Compare Slint with a minimal custom UI before selecting the toolkit. Typed actions
are core interfaces; inference is optional after the first state-preserving guest
reload. Keep the 256 MiB cold-idle target. Boot recovery and live state transfer
are separate; a booting disk image does not prove the required ISO boots.

Phase 0 implements compile/tooling feasibility; actual ISO boot starts M1 in
`docs/05-delivery/milestones.md`. Re-run the Phase 0 checks before extending it.
Rust is workspace-local under ignored `work/toolchains/`; existing MSVC/SDK were reused.
Pinned QEMU/OVMF/ISO tools are workspace-local under ignored `work/tools/`.
P1-02/P1-03 launch bounded offline QEMU processes; no WSL, system setting, physical
disk or global package was changed. xorriso needs execution outside the sandbox's
MSYS2 named-object restriction. Do not attempt the entire backlog at once.
